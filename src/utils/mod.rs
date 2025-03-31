//-- ./src/utils

//! Utility modules that don't fit into other places
#[cfg(test)]
mod mock_uuid;

pub mod metadata;

#[cfg(test)]
pub use mock_uuid::mock_uuid;
