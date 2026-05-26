pub mod error;

pub use ytmusic_service_proto::ytmusic::v2;

pub struct YtMusicServiceClient;

#[test]
fn client_crate_reexports_v2_proto() {
    let _ = std::any::TypeId::of::<v2::GetStatusRequest>();
}
