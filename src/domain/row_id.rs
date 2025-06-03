//-- ./src/domain/row_id.rs

// #![allow(unused)] // For development only

//! # RowID Domain Type
//!
//! This module defines the `RowID` newtype, a strongly-typed wrapper around `uuid::Uuid`
//! for uniquely identifying database rows in the authentication service. By default, it generates
//! UUID version 7 identifiers, which are time-ordered and suitable for use as database primary keys.
//!
//! ## Features
//! - `RowID::new()` generates a new UUID v7 for production use.
//! - `RowID::mock()` (for tests) generates a random, realistic RowID with a time-ordered UUID v7.
//! - Implements common traits for serialization, comparison, and debugging.
//! - Provides conversions from `uuid::Uuid` and from string parsing.
//!
//! Use `RowID` instead of raw UUIDs to enforce type safety, domain clarity, and to leverage
//! time-ordered UUIDs for efficient indexing and sharding.

/// A newtype wrapper around `uuid::Uuid` representing a unique, time-ordered row identifier.
///
/// `RowID` is used throughout the authentication service as a strongly-typed primary key for database rows.
/// By default, it uses UUID version 7, which is time-ordered and suitable for efficient indexing and sharding.
///
/// Use `RowID` in place of raw UUIDs to improve type safety and domain clarity.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RowID(uuid::Uuid);

impl From<uuid::Uuid> for RowID {
    /// Converts a `uuid::Uuid` into a `RowID`.
    ///
    /// This allows you to create a `RowID` from an existing UUID, for example when loading from a database or external source.
    ///
    /// # Example
    /// ```
    /// let uuid = uuid::Uuid::now_v7();
    /// let row_id = RowID::from(uuid);
    /// ```
    fn from(uuid: uuid::Uuid) -> Self {
        RowID(uuid)
    }
}

impl std::str::FromStr for RowID {
    type Err = uuid::Error;

    /// Attempts to parse a `RowID` from a string representation of a UUID.
    ///
    /// # Example
    /// ```
    /// let row_id: RowID = "01890b7c-6b7a-7e4b-bb1a-6e2f5e8c7e2a".parse().unwrap();
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(s).map(RowID)
    }
}

impl std::fmt::Display for RowID {
  /// Formats the `RowID` as a hyphenated UUID string.
  ///
  /// # Example
  /// ```
  /// let row_id = RowID::new();
  /// println!("{}", row_id); // prints a UUID string
  /// ```
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{}", self.0)
  }
}

impl RowID {
    /// Creates a new `RowID` using a UUID version 7 (time-ordered).
    ///
    /// # Returns
    /// A `RowID` instance containing a freshly generated UUID v7, suitable for use as a database primary key.
    ///
    /// # Example
    /// ```
    /// let row_id = RowID::new();
    /// ```
    pub fn new() -> Self {
        let row_id = uuid::Uuid::now_v7();
        Self(row_id)
    }

    /// Returns the inner `uuid::Uuid` value from this `RowID`.
    ///
    /// # Example
    /// ```
    /// let row_id = RowID::new();
    /// let uuid: uuid::Uuid = row_id.into_uuid();
    /// ```
    pub fn into_uuid(self) -> uuid::Uuid {
      self.0
  }

    /// Generates a mock `RowID` with a random, realistic UUID v7 timestamp.
    ///
    /// This method is intended for use in tests and development. It produces a `RowID`
    /// with a UUID v7 value based on a randomly generated timestamp after the Unix epoch,
    /// simulating real-world, time-ordered IDs.
    ///
    /// # Example
    /// ```
    /// let mock_id = RowID::mock();
    /// ```
    #[cfg(test)]
    pub fn mock() -> Self {
        use chrono::{DateTime, Utc};
        use fake::faker::chrono::en::DateTimeAfter;
        use fake::Fake;

        // Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
        let random_datetime: DateTime<Utc> =
            DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();

        // Convert datetime to a UUID timestamp
        let random_uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
            uuid::NoContext,
            random_datetime.timestamp() as u64,
            random_datetime.timestamp_nanos_opt().unwrap() as u32,
        );

        // Generate Uuid V7
        let row_id = uuid::Uuid::new_v7(random_uuid_timestamp);

        Self(row_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_rowid_new_creates_uuid_v7() {
        let row_id = RowID::new();
        // UUID v7 has version 7
        assert_eq!(row_id.0.get_version_num(), 7);
    }

    #[test]
    fn test_rowid_mock_creates_uuid_v7() {
        let row_id = RowID::mock();
        assert_eq!(row_id.0.get_version_num(), 7);
    }

    #[test]
    fn test_rowid_equality() {
        let row_id1 = RowID::new();
        let row_id2 = RowID(row_id1.0);
        assert_eq!(row_id1, row_id2);
    }

    #[test]
    fn test_rowid_serialization() {
        let row_id = RowID::new();
        let serialized = serde_json::to_string(&row_id).unwrap();
        let deserialized: RowID = serde_json::from_str(&serialized).unwrap();
        assert_eq!(row_id, deserialized);
    }

    #[test]
    fn test_from_uuid_for_rowid() {
        let uuid = uuid::Uuid::now_v7();
        let row_id = RowID::from(uuid);
        assert_eq!(row_id.0, uuid);
    }

    #[test]
    fn test_from_str_for_rowid_valid() {
        let uuid = uuid::Uuid::now_v7();
        let uuid_str = uuid.to_string();
        let row_id = RowID::from_str(&uuid_str).unwrap();
        assert_eq!(row_id.0, uuid);
    }

    #[test]
    fn test_from_str_for_rowid_invalid() {
        let invalid_str = "not-a-uuid";
        let result = RowID::from_str(invalid_str);
        assert!(result.is_err());
    }
}
