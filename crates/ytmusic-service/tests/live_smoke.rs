use std::{io, net::SocketAddr, path::PathBuf, time::Duration};

use tokio::{task::JoinHandle, time::sleep};
use ytmusic_service::config::ServiceConfig;
use ytmusic_service_client::{YtMusicServiceClient, v2 as pb};

const DEFAULT_QUERY: &str = "Miles Davis";

struct LiveServer {
    local_addr: SocketAddr,
    task: JoinHandle<Result<(), ytmusic_service::error::ServiceError>>,
}

impl LiveServer {
    async fn start(browser_auth_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let local_addr = reserve_local_addr()?;
        let config = ServiceConfig::from_parts(&local_addr.to_string(), browser_auth_path)?;
        let task = tokio::spawn(async move { ytmusic_service::run(config).await });

        wait_for_server(local_addr).await?;

        Ok(Self { local_addr, task })
    }

    fn endpoint(&self) -> String {
        format!("http://{}", self.local_addr)
    }
}

impl Drop for LiveServer {
    fn drop(&mut self) {
        self.task.abort();
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

async fn wait_for_server(local_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!("http://{local_addr}");
    let mut last_error = String::from("server did not become ready");

    for _ in 0..60 {
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

#[tokio::test]
#[ignore = "requires live YouTube Music credentials and network access"]
async fn live_v2_stack_serves_music_cipher_and_status() -> Result<(), Box<dyn std::error::Error>> {
    let browser_auth_path = PathBuf::from(required_env("YTMUSIC_SERVICE_LIVE_BROWSER_JSON")?);
    let query =
        std::env::var("YTMUSIC_SERVICE_LIVE_QUERY").unwrap_or_else(|_| DEFAULT_QUERY.to_owned());
    let video_id = required_env("YTMUSIC_SERVICE_LIVE_VIDEO_ID")?;

    let server = LiveServer::start(browser_auth_path).await?;
    let mut client = YtMusicServiceClient::connect(server.endpoint()).await?;

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
