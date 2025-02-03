use tonic_reflection::server::{ServerReflection, ServerReflectionServer};

use crate::rpc::proto::FILE_DESCRIPTOR_SET as REFLECTIONS_DESCRIPTOR_SET;

pub struct ReflectionsService {}

impl ReflectionsService {
    pub fn new() -> ServerReflectionServer<impl ServerReflection> {
        let service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(REFLECTIONS_DESCRIPTOR_SET)
            .build_v1()
            .expect("ERROR: Building gRPC reflection service");

        service
    }
}