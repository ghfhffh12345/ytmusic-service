pub mod ytmusic {
    pub mod v1 {
        tonic::include_proto!("ytmusic.v1");
        pub const FILE_DESCRIPTOR_SET: &[u8] =
            tonic::include_file_descriptor_set!("ytmusic_descriptor");

        pub mod admin {
            tonic::include_proto!("ytmusic.v1.admin");
        }
    }
}
