pub mod ytmusic {
    pub mod v2 {
        tonic::include_proto!("ytmusic.v2");
        pub const FILE_DESCRIPTOR_SET: &[u8] =
            tonic::include_file_descriptor_set!("ytmusic_v2_descriptor");
    }
}
