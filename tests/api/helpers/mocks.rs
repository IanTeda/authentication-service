// #![allow(unused)] // For beginning only.

use chrono::{DateTime, SubsecRound, Utc};
// use chrono::prelude::*;
use fake::faker::boolean::en::Boolean;
use fake::faker::chrono::en::DateTime;
use fake::faker::chrono::en::DateTimeAfter;
use fake::faker::name::en::Name;
use fake::{faker::internet::en::SafeEmail, Fake};
use secrecy::Secret;
use uuid::Uuid;

use personal_ledger_backend::{database, domain, error};

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

pub fn users(password: &String) -> Result<database::Users, error::BackendError> {
    //-- Generate a random id (Uuid V7) by first generating a random timestamp
    // Generate Uuid V7
    let random_id: Uuid = uuid_v7();

    // Generate random safe email address
    let random_email: String = SafeEmail().fake();
    let random_email = domain::EmailAddress::parse(random_email)?;

    // Generate random name
    let random_name: String = Name().fake();
    let random_name = domain::UserName::parse(random_name)?;

    // Generate random password hash
    let password = Secret::new(password.to_owned());
    let password_hash = domain::PasswordHash::parse(password)?;

    let random_role: domain::UserRole = rand::random();

    // Generate random boolean value
    let random_is_active: bool = Boolean(4).fake();

    let random_is_verified: bool = Boolean(4).fake();

    // Generate random DateTime
    let random_created_on: DateTime<Utc> = DateTime().fake();
    let random_created_on = random_created_on.round_subsecs(0);

    let random_user = database::Users {
        id: random_id,
        email: random_email,
        name: random_name,
        password_hash,
        role: random_role,
        is_active: random_is_active,
        is_verified: random_is_verified,
        created_on: random_created_on,
    };

    Ok(random_user)
}

pub fn refresh_tokens(
    user: &database::Users,
) -> Result<database::RefreshTokens, error::BackendError> {
    use chrono::SubsecRound;
    use fake::faker::boolean::en::Boolean;
    use fake::faker::chrono::en::DateTime;
    use fake::Fake;
    use rand::distributions::DistString;
    use secrecy::Secret;

    let random_id = uuid_v7();
    let user_id = user.id.to_owned();
    let random_secret =
        rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 60);
    let random_secret = Secret::new(random_secret);

    let random_token = domain::RefreshToken::new(&random_secret, &user)?;

    // Generate random boolean value
    let random_is_active: bool = Boolean(4).fake();

    // Generate random DateTime
    let random_created_on: DateTime<Utc> = DateTime().fake();
    let random_created_on = random_created_on.round_subsecs(0);

    let random_refresh_token = database::RefreshTokens {
        id: random_id,
        user_id,
        token: random_token,
        is_active: random_is_active,
        created_on: random_created_on,
    };

    Ok(random_refresh_token)
}
