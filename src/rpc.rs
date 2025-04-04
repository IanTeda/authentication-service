pub mod proto {
    // The string specified here must match the proto package name
    tonic::include_proto!("authentication");

    #[allow(dead_code)]
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("authentication_descriptor");
}

