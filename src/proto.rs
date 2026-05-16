pub mod ytmusic {
    pub mod v1 {
        tonic::include_proto!("ytmusic.v1");
        pub const PUBLIC_FILE_DESCRIPTOR_SET: &[u8] =
            tonic::include_file_descriptor_set!("ytmusic_public_descriptor");

        pub mod admin {
            tonic::include_proto!("ytmusic.v1.admin");
            pub const ADMIN_FILE_DESCRIPTOR_SET: &[u8] =
                tonic::include_file_descriptor_set!("ytmusic_admin_descriptor");
        }
    }
}
