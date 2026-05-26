pub mod error;

pub use error::{ClientError, ClientStatus};
pub use ytmusic_service_proto::ytmusic::v2;

use tonic::transport::{Channel, Endpoint};

#[derive(Debug, Clone)]
pub struct YtMusicServiceClient {
    music: v2::yt_music_client::YtMusicClient<Channel>,
    cipher: v2::yt_cipher_client::YtCipherClient<Channel>,
    status: v2::service_status_client::ServiceStatusClient<Channel>,
}

impl YtMusicServiceClient {
    pub async fn connect(endpoint: impl Into<String>) -> Result<Self, ClientError> {
        let channel = Endpoint::from_shared(endpoint.into())?.connect().await?;
        Ok(Self::from_channel(channel))
    }

    pub fn from_channel(channel: Channel) -> Self {
        Self {
            music: v2::yt_music_client::YtMusicClient::new(channel.clone()),
            cipher: v2::yt_cipher_client::YtCipherClient::new(channel.clone()),
            status: v2::service_status_client::ServiceStatusClient::new(channel),
        }
    }

    pub fn music(&mut self) -> MusicApi<'_> {
        MusicApi {
            inner: &mut self.music,
        }
    }

    pub fn cipher(&mut self) -> CipherApi<'_> {
        CipherApi {
            inner: &mut self.cipher,
        }
    }

    pub fn status(&mut self) -> StatusApi<'_> {
        StatusApi {
            inner: &mut self.status,
        }
    }
}

pub struct MusicApi<'a> {
    inner: &'a mut v2::yt_music_client::YtMusicClient<Channel>,
}

impl<'a> MusicApi<'a> {
    pub fn inner_mut(&mut self) -> &mut v2::yt_music_client::YtMusicClient<Channel> {
        self.inner
    }
}

pub struct CipherApi<'a> {
    inner: &'a mut v2::yt_cipher_client::YtCipherClient<Channel>,
}

impl<'a> CipherApi<'a> {
    pub fn inner_mut(&mut self) -> &mut v2::yt_cipher_client::YtCipherClient<Channel> {
        self.inner
    }
}

pub struct StatusApi<'a> {
    inner: &'a mut v2::service_status_client::ServiceStatusClient<Channel>,
}

impl<'a> StatusApi<'a> {
    pub async fn get_status(&mut self) -> Result<v2::GetStatusResponse, ClientStatus> {
        self.inner
            .get_status(v2::GetStatusRequest {})
            .await
            .map(|response| response.into_inner())
            .map_err(ClientStatus::from)
    }

    pub fn inner_mut(&mut self) -> &mut v2::service_status_client::ServiceStatusClient<Channel> {
        self.inner
    }
}

#[test]
fn client_crate_reexports_v2_proto() {
    let _ = std::any::TypeId::of::<v2::GetStatusRequest>();
}
