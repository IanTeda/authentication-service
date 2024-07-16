//-- ./src/utils

//! Utility modules that don't fit into other places

mod mock_uuid;
#[cfg(test)]
pub use mock_uuid::mock_uuid;
