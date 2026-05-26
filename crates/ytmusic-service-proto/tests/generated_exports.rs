#[test]
fn generated_v2_exports_are_available() {
    use ytmusic_service_proto::ytmusic::v2::{
        FILE_DESCRIPTOR_SET, GetStatusRequest, service_status_client::ServiceStatusClient,
        yt_cipher_client::YtCipherClient, yt_music_client::YtMusicClient,
    };

    let _ = std::any::TypeId::of::<YtMusicClient<tonic::transport::Channel>>();
    let _ = std::any::TypeId::of::<YtCipherClient<tonic::transport::Channel>>();
    let _ = std::any::TypeId::of::<ServiceStatusClient<tonic::transport::Channel>>();
    let _ = GetStatusRequest {};
    assert!(!FILE_DESCRIPTOR_SET.is_empty());
}
