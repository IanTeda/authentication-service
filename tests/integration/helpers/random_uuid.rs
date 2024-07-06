//-- ./tests/helpers/random_uuid.rs

//! Generate a random Uuid V7, returning a Uuid.
//! 
//! The function first generates a random DatTime which is converted to a timestamp.
//! The random timestamp is then used to generate the Uuid V7

use chrono::{DateTime, Utc};
use fake::{faker::chrono::en::DateTimeAfter, Fake};
use uuid::Uuid;

#[allow(dead_code)]
pub fn generate_random_uuid() -> Uuid {
    	// Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
	let random_datetime: DateTime<Utc> = DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();
	// Convert datetime to a UUID timestamp
	let random_uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
		uuid::NoContext,
		random_datetime.timestamp() as u64,
		random_datetime.timestamp_nanos_opt().unwrap() as u32,
	);

	// Generate Uuid V7
	Uuid::new_v7(random_uuid_timestamp)
}