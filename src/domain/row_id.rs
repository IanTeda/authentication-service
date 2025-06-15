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
#[derive(
    Debug, Copy, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize,
)]
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

impl PartialOrd for RowID {
    /// Compares two `RowID` instances for ordering.
    ///
    /// Since `RowID` uses UUID v7 which is time-ordered, this comparison
    /// provides chronological ordering based on creation time.
    ///
    /// # Returns
    /// - `Some(Ordering::Less)` if this RowID was created before the other
    /// - `Some(Ordering::Greater)` if this RowID was created after the other
    /// - `Some(Ordering::Equal)` if the RowIDs are identical
    /// - `None` should never occur since UUIDs are always comparable
    ///
    /// # Example
    /// ```rust
    /// let id1 = RowID::new();
    /// std::thread::sleep(std::time::Duration::from_millis(1));
    /// let id2 = RowID::new();
    ///
    /// assert!(id1 < id2); // id1 was created first
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RowID {
    /// Provides total ordering for `RowID` instances.
    ///
    /// UUID v7 values are designed to be sortable by creation time,
    /// making this ordering both deterministic and chronologically meaningful.
    /// This enables efficient range queries and time-based operations.
    ///
    /// # Example
    /// ```rust
    /// let mut ids = vec![RowID::new(), RowID::new(), RowID::new()];
    /// ids.sort();
    /// // ids are now sorted by creation time
    /// ```
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl Eq for RowID {}

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

    /// Sorts a slice of `RowID` values in ascending chronological order.
    ///
    /// This is a convenience method that sorts RowIDs by their creation time,
    /// with the earliest created IDs appearing first.
    ///
    /// # Arguments
    /// * `ids` - A mutable slice of `RowID` values to sort
    ///
    /// # Example
    /// ```rust
    /// let mut ids = vec![
    ///     RowID::new(),
    ///     RowID::new(),
    ///     RowID::new()
    /// ];
    /// RowID::sort_ascending(&mut ids);
    /// // ids[0] is the earliest created, ids[2] is the latest
    /// ```
    pub fn sort_ascending(ids: &mut [RowID]) {
        ids.sort();
    }

    /// Sorts a slice of `RowID` values in descending chronological order.
    ///
    /// This sorts RowIDs with the most recently created IDs appearing first,
    /// useful for displaying recent records at the top.
    ///
    /// # Arguments
    /// * `ids` - A mutable slice of `RowID` values to sort
    ///
    /// # Example
    /// ```rust
    /// let mut ids = vec![
    ///     RowID::new(),
    ///     RowID::new(),
    ///     RowID::new()
    /// ];
    /// RowID::sort_descending(&mut ids);
    /// // ids[0] is the latest created, ids[2] is the earliest
    /// ```
    pub fn sort_descending(ids: &mut [RowID]) {
        ids.sort_by(|a, b| b.cmp(a));
    }

    /// Returns a sorted vector of `RowID` values in ascending order.
    ///
    /// This creates a new vector without modifying the original collection.
    ///
    /// # Arguments
    /// * `ids` - An iterator of `RowID` values to sort
    ///
    /// # Returns
    /// A new `Vec<RowID>` sorted in ascending chronological order
    ///
    /// # Example
    /// ```rust
    /// let ids = vec![RowID::new(), RowID::new(), RowID::new()];
    /// let sorted = RowID::sorted_ascending(ids.iter().cloned());
    /// ```
    pub fn sorted_ascending<I>(ids: I) -> Vec<RowID>
    where
        I: IntoIterator<Item = RowID>,
    {
        let mut result: Vec<RowID> = ids.into_iter().collect();
        result.sort();
        result
    }

    /// Returns a sorted vector of `RowID` values in descending order.
    ///
    /// This creates a new vector without modifying the original collection.
    ///
    /// # Arguments
    /// * `ids` - An iterator of `RowID` values to sort
    ///
    /// # Returns
    /// A new `Vec<RowID>` sorted in descending chronological order
    ///
    /// # Example
    /// ```rust
    /// let ids = vec![RowID::new(), RowID::new(), RowID::new()];
    /// let sorted = RowID::sorted_descending(ids.iter().cloned());
    /// ```
    pub fn sorted_descending<I>(ids: I) -> Vec<RowID>
    where
        I: IntoIterator<Item = RowID>,
    {
        let mut result: Vec<RowID> = ids.into_iter().collect();
        result.sort_by(|a, b| b.cmp(a));
        result
    }

    /// Finds the minimum (earliest created) `RowID` in a collection.
    ///
    /// # Arguments
    /// * `ids` - An iterator of `RowID` values
    ///
    /// # Returns
    /// * `Some(RowID)` - The earliest created RowID
    /// * `None` - If the collection is empty
    ///
    /// # Example
    /// ```rust
    /// let ids = vec![RowID::new(), RowID::new(), RowID::new()];
    /// let earliest = RowID::min(ids.iter().cloned()).unwrap();
    /// ```
    pub fn min<I>(ids: I) -> Option<RowID>
    where
        I: IntoIterator<Item = RowID>,
    {
        ids.into_iter().min()
    }

    /// Finds the maximum (latest created) `RowID` in a collection.
    ///
    /// # Arguments
    /// * `ids` - An iterator of `RowID` values
    ///
    /// # Returns
    /// * `Some(RowID)` - The latest created RowID
    /// * `None` - If the collection is empty
    ///
    /// # Example
    /// ```rust
    /// let ids = vec![RowID::new(), RowID::new(), RowID::new()];
    /// let latest = RowID::max(ids.iter().cloned()).unwrap();
    /// ```
    pub fn max<I>(ids: I) -> Option<RowID>
    where
        I: IntoIterator<Item = RowID>,
    {
        ids.into_iter().max()
    }

    /// Checks if this `RowID` was created before another `RowID`.
    ///
    /// # Example
    /// ```rust
    /// let id1 = RowID::new();
    /// std::thread::sleep(std::time::Duration::from_millis(1));
    /// let id2 = RowID::new();
    ///
    /// assert!(id1.is_before(&id2));
    /// assert!(!id2.is_before(&id1));
    /// ```
    pub fn is_before(&self, other: &RowID) -> bool {
        self < other
    }

    /// Checks if this `RowID` was created after another `RowID`.
    ///
    /// # Example
    /// ```rust
    /// let id1 = RowID::new();
    /// std::thread::sleep(std::time::Duration::from_millis(1));
    /// let id2 = RowID::new();
    ///
    /// assert!(id2.is_after(&id1));
    /// assert!(!id1.is_after(&id2));
    /// ```
    pub fn is_after(&self, other: &RowID) -> bool {
        self > other
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
    use std::{str::FromStr, thread, time::Duration};

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

    #[test]
    fn test_rowid_ordering() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        assert!(id1 < id2);
        assert!(id2 < id3);
        assert!(id1 < id3);
    }

    #[test]
    fn test_sort_ascending() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        let mut ids = vec![id3, id1, id2];
        RowID::sort_ascending(&mut ids);

        assert_eq!(ids, vec![id1, id2, id3]);
    }

    #[test]
    fn test_sort_descending() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        let mut ids = vec![id1, id3, id2];
        RowID::sort_descending(&mut ids);

        assert_eq!(ids, vec![id3, id2, id1]);
    }

    #[test]
    fn test_sorted_ascending() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        let ids = vec![id3, id1, id2];
        let sorted = RowID::sorted_ascending(ids.iter().cloned());

        assert_eq!(sorted, vec![id1, id2, id3]);
    }

    #[test]
    fn test_sorted_descending() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        let ids = vec![id1, id3, id2];
        let sorted = RowID::sorted_descending(ids.iter().cloned());

        assert_eq!(sorted, vec![id3, id2, id1]);
    }

    #[test]
    fn test_min_max() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id3 = RowID::new();

        let ids = vec![id2, id1, id3];

        assert_eq!(RowID::min(ids.iter().cloned()).unwrap(), id1);
        assert_eq!(RowID::max(ids.iter().cloned()).unwrap(), id3);
    }

    #[test]
    fn test_min_max_empty() {
        let ids: Vec<RowID> = vec![];
        assert!(RowID::min(ids.iter().cloned()).is_none());
        assert!(RowID::max(ids.iter().cloned()).is_none());
    }

    #[test]
    fn test_is_before_after() {
        let id1 = RowID::new();
        thread::sleep(Duration::from_millis(1));
        let id2 = RowID::new();

        assert!(id1.is_before(&id2));
        assert!(!id2.is_before(&id1));
        assert!(id2.is_after(&id1));
        assert!(!id1.is_after(&id2));
    }

    #[test]
    fn test_equality() {
        let id = RowID::new();
        let same_id = RowID(id.0);

        assert!(!(id.is_before(&same_id)));
        assert!(!(id.is_after(&same_id)));
        assert_eq!(id, same_id);
    }

    #[test]
    fn test_mock_ids_are_sortable() {
        let mut mock_ids = vec![RowID::mock(), RowID::mock(), RowID::mock()];
        
        // Should not panic when sorting mock IDs
        RowID::sort_ascending(&mut mock_ids);
        
        // All should be different (very high probability with UUIDs)
        assert_ne!(mock_ids[0], mock_ids[1]);
        assert_ne!(mock_ids[1], mock_ids[2]);
        assert_ne!(mock_ids[0], mock_ids[2]);
    }

    #[test]
    fn test_large_collection_sorting() {
        let mut ids: Vec<RowID> = (0..1000).map(|_| RowID::new()).collect();
        let original_ids = ids.clone();
        
        RowID::sort_descending(&mut ids);
        
        // Should be in reverse chronological order
        for i in 0..ids.len()-1 {
            assert!(ids[i] >= ids[i+1]);
        }
        
        // All original IDs should still be present
        let mut sorted_original = original_ids;
        sorted_original.sort();
        ids.sort();
        assert_eq!(ids, sorted_original);
    }
}
