#![allow(unused)] // For beginning only.

use chrono::prelude::*;
use fake::Fake;
use fake::faker::chrono::en::DateTimeAfter;
use uuid::Uuid;

use personal_ledger_backend::error;

pub fn uuid_v7() -> Uuid {
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
    Uuid::new_v7(random_uuid_timestamp)
}

pub fn password() -> Result<String, error::BackendError> {
    // Get a random count to repeat minimum password requirements
    let random_count = (5..30).fake::<i64>() as usize;
    // Password must have a lower and upper case plus a number and special character
    let password = "aB1%".repeat(random_count);

    Ok(password)
}

// pub fn user_model(
//     password: &String,
// ) -> Result<database::UserModel, error::BackendError> {
//     //-- Generate a random id (Uuid V7) by first generating a random timestamp
//     // Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
//     let random_datetime: DateTime<Utc> =
//         DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();

//     // Convert datetime to a UUID timestamp
//     let random_uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
//         uuid::NoContext,
//         random_datetime.timestamp() as u64,
//         random_datetime.timestamp_nanos_opt().unwrap() as u32,
//     );
//     // Generate Uuid V7
//     let id: Uuid = Uuid::new_v7(random_uuid_timestamp);

//     // Generate random safe email address
//     let random_email: String = SafeEmail().fake();
//     let email = domains::EmailAddress::parse(random_email)?;

//     // Generate random name
//     let random_name: String = Name().fake();
//     let user_name = domains::UserName::parse(random_name)?;

//     // Generate random password hash
//     let password = Secret::new(password.to_owned());
//     let password_hash = domains::PasswordHash::parse(password)?;

//     // Generate random boolean value
//     let is_active: bool = Boolean(4).fake();

//     // Generate random DateTime
//     let created_on: DateTime<Utc> = DateTime().fake();

//     let random_user = database::UserModel {
//         id,
//         email,
//         user_name,
//         password_hash,
//         is_active,
//         created_on,
//     };

//     Ok(random_user)
// }

// pub async fn access_token(
//     user_id: &Uuid,
//     token_secret: &Secret<String>
// ) -> Result<domains::AccessToken, error::BackendError> {
//     // Build an Access Token
//     let access_token =
//         domains::AccessToken::new(token_secret, user_id).await?;

//     Ok(access_token)
// }
