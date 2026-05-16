fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let public_descriptor_path = out_dir.join("ytmusic_public_descriptor.bin");
    let admin_descriptor_path = out_dir.join("ytmusic_admin_descriptor.bin");

    tonic_build::configure()
        .file_descriptor_set_path(&public_descriptor_path)
        .compile_protos(&["proto/ytmusic/v1/public.proto"], &["proto"])?;

    tonic_build::configure()
        .file_descriptor_set_path(&admin_descriptor_path)
        .compile_protos(&["proto/ytmusic/v1/admin.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
