use std::{
    io,
    net::SocketAddr,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    time::Duration,
};

use tokio::{task::JoinHandle, time::sleep};
use ytmusic_service::config::ServiceConfig;
use ytmusic_service_client::{YtMusicServiceClient, v2 as pb};

const DEFAULT_QUERY: &str = "Miles Davis";

struct LiveServer {
    local_addr: SocketAddr,
    process: LiveServerProcess,
}

enum LiveServerProcess {
    InProcess(Option<JoinHandle<Result<(), ytmusic_service::error::ServiceError>>>),
    Child(Option<Child>),
}

impl LiveServer {
    async fn start(browser_auth_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let local_addr = reserve_local_addr()?;
        let process = match std::env::var_os("YTMUSIC_SERVICE_LIVE_BINARY") {
            Some(binary_path) => LiveServerProcess::spawn_release_binary(
                local_addr,
                Path::new(&binary_path),
                &browser_auth_path,
            )?,
            None => LiveServerProcess::spawn_in_process(local_addr, browser_auth_path)?,
        };
        let mut server = Self {
            local_addr,
            process,
        };

        server.wait_for_server().await?;

        Ok(server)
    }

    fn endpoint(&self) -> String {
        format!("http://{}", self.local_addr)
    }

    async fn wait_for_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = self.endpoint();
        let mut last_error = String::from("server did not become ready");

        for _ in 0..60 {
            self.process.check_early_exit().await?;

            match YtMusicServiceClient::connect(endpoint.clone()).await {
                Ok(mut client) => match client.status().get_status().await {
                    Ok(status) if status.lifecycle == "serving" => return Ok(()),
                    Ok(status) => {
                        last_error = format!("unexpected lifecycle {}", status.lifecycle);
                    }
                    Err(error) => {
                        last_error = error.to_string();
                    }
                },
                Err(error) => {
                    last_error = error.to_string();
                }
            }

            sleep(Duration::from_millis(500)).await;
        }

        Err(io::Error::new(io::ErrorKind::TimedOut, last_error).into())
    }
}

impl Drop for LiveServer {
    fn drop(&mut self) {
        self.process.shutdown();
    }
}

fn reserve_local_addr() -> io::Result<SocketAddr> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    let local_addr = listener.local_addr()?;
    drop(listener);
    Ok(local_addr)
}

fn required_env(name: &'static str) -> Result<String, Box<dyn std::error::Error>> {
    std::env::var(name).map_err(|source| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("missing required env var {name}: {source}"),
        )
        .into()
    })
}

impl LiveServerProcess {
    fn spawn_in_process(
        local_addr: SocketAddr,
        browser_auth_path: PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = ServiceConfig::from_parts(&local_addr.to_string(), browser_auth_path)?;
        let task = tokio::spawn(async move { ytmusic_service::run(config).await });
        Ok(Self::InProcess(Some(task)))
    }

    fn spawn_release_binary(
        local_addr: SocketAddr,
        binary_path: &Path,
        browser_auth_path: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut command = Command::new(binary_path);
        command
            .env("YTMUSIC_SERVICE_ADDR", local_addr.to_string())
            .env("YTMUSIC_SERVICE_BROWSER_JSON", browser_auth_path)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if let Some(timeout_ms) = std::env::var_os("YTMUSIC_SERVICE_RPC_TIMEOUT_MS") {
            command.env("YTMUSIC_SERVICE_RPC_TIMEOUT_MS", timeout_ms);
        }

        let child = command.spawn().map_err(|source| {
            io::Error::new(
                source.kind(),
                format!(
                    "failed to spawn live smoke binary {}: {source}",
                    binary_path.display()
                ),
            )
        })?;

        Ok(Self::Child(Some(child)))
    }

    async fn check_early_exit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::InProcess(task) => {
                if task.as_ref().is_some_and(JoinHandle::is_finished) {
                    let task = task.take().expect("finished task must exist");
                    let result = task.await.map_err(|source| {
                        io::Error::other(format!("live smoke task join failed: {source}"))
                    })?;
                    result.map_err(|source| {
                        io::Error::other(format!("live smoke task exited early: {source}"))
                    })?;
                }
            }
            Self::Child(child) => {
                if let Some(status) = child
                    .as_mut()
                    .expect("child process must exist")
                    .try_wait()?
                {
                    return Err(early_exit_error("live smoke binary", status).into());
                }
            }
        }

        Ok(())
    }

    fn shutdown(&mut self) {
        match self {
            Self::InProcess(task) => {
                if let Some(task) = task.take() {
                    task.abort();
                }
            }
            Self::Child(child) => {
                if let Some(mut child) = child.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
    }
}

fn early_exit_error(subject: &str, status: ExitStatus) -> io::Error {
    io::Error::other(format!("{subject} exited before readiness: {status}"))
}

fn path_from_env(name: &'static str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    std::env::var_os(name).map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("missing required env var {name}"),
        )
        .into()
    })
}

fn validate_browser_auth_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let browser_auth_path = path.to_path_buf();
    if !browser_auth_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "YTMUSIC_SERVICE_LIVE_BROWSER_JSON path does not exist: {}",
                browser_auth_path.display()
            ),
        )
        .into());
    }
    if !browser_auth_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "YTMUSIC_SERVICE_LIVE_BROWSER_JSON path is not a file: {}",
                browser_auth_path.display()
            ),
        )
        .into());
    }
    Ok(())
}

fn endpoint_from_env(name: &'static str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

#[tokio::test]
#[ignore = "requires live YouTube Music credentials and network access"]
async fn live_v2_stack_serves_music_cipher_and_status() -> Result<(), Box<dyn std::error::Error>> {
    let query =
        std::env::var("YTMUSIC_SERVICE_LIVE_QUERY").unwrap_or_else(|_| DEFAULT_QUERY.to_owned());
    let video_id = required_env("YTMUSIC_SERVICE_LIVE_VIDEO_ID")?;
    let _server;
    let endpoint = if let Some(endpoint) = endpoint_from_env("YTMUSIC_SERVICE_LIVE_ENDPOINT") {
        endpoint
    } else {
        let browser_auth_path = path_from_env("YTMUSIC_SERVICE_LIVE_BROWSER_JSON")?;
        validate_browser_auth_path(&browser_auth_path)?;
        _server = LiveServer::start(browser_auth_path).await?;
        _server.endpoint()
    };
    let mut client = YtMusicServiceClient::connect(endpoint).await?;

    let status = client.status().get_status().await?;
    assert_eq!(status.lifecycle, "serving");
    assert!(status.ytmusic_ready);
    assert!(status.cipher_ready);

    let search = client
        .music()
        .inner_mut()
        .search(pb::SearchRequest {
            query,
            filter: Some(pb::SearchFilter::Songs as i32),
            ignore_spelling: false,
        })
        .await?
        .into_inner();
    assert!(!search.items.is_empty());

    let song = client
        .music()
        .inner_mut()
        .get_song(pb::GetSongRequest {
            video_id: video_id.clone(),
        })
        .await?
        .into_inner();
    assert_eq!(
        song.video_details
            .as_ref()
            .map(|details| details.video_id.as_str()),
        Some(video_id.as_str())
    );

    let account = client
        .music()
        .inner_mut()
        .get_account_info(pb::Empty {})
        .await?
        .into_inner();
    assert!(!account.account_name.trim().is_empty());

    let signature_timestamp = client
        .cipher()
        .inner_mut()
        .get_signature_timestamp(pb::Empty {})
        .await?
        .into_inner();
    assert!(signature_timestamp.signature_timestamp > 0);

    client.cipher().inner_mut().refresh(pb::Empty {}).await?;

    Ok(())
}
