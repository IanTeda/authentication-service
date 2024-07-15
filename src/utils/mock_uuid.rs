//-- ./src/utils/mock_uuid.rs

//! Utility modules that don't fit into other places
//!
//! https://www.reddit.com/r/rust/comments/ny6k3f/cfgtest_doesnt_take_affect_when_running/
 #![allow(unused)]
#[cfg(feature = "mocks")]
pub fn mock_uuid() -> uuid::Uuid {
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
    let uuid = uuid::Uuid::new_v7(random_uuid_timestamp);

    uuid
}
