use std::any::TypeId;

use tonic::transport::Channel;
use ytmusic_service_proto::ytmusic::v1::{
    PUBLIC_FILE_DESCRIPTOR_SET, SearchRequest, admin::ADMIN_FILE_DESCRIPTOR_SET,
    admin::ReloadBrowserAuthRequest, admin::yt_music_admin_client::YtMusicAdminClient,
    yt_music_public_client::YtMusicPublicClient,
};

#[test]
fn generated_public_and_admin_exports_are_available() {
    let _ = SearchRequest::default();
    let _ = ReloadBrowserAuthRequest::default();
    let _ = TypeId::of::<YtMusicPublicClient<Channel>>();
    let _ = TypeId::of::<YtMusicAdminClient<Channel>>();

    assert!(!PUBLIC_FILE_DESCRIPTOR_SET.is_empty());
    assert!(!ADMIN_FILE_DESCRIPTOR_SET.is_empty());
}
