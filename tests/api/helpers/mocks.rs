// #![allow(unused)] // For beginning only.

use authentication_service::AuthenticationError;
use chrono::{DateTime, SubsecRound, Utc};
use fake::faker::boolean::en::Boolean;
use fake::faker::chrono::en::DateTime;
use fake::faker::chrono::en::DateTimeAfter;
use fake::faker::name::en::Name;
use fake::{faker::internet::en::SafeEmail, Fake};
use secrecy::SecretString;
use std::net::Ipv4Addr;
use uuid::Uuid;

use authentication_service::{database, domain};

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

pub fn password() -> Result<String, AuthenticationError> {
    // Get a random count to repeat minimum password requirements
    let random_count = (5..30).fake::<i64>() as usize;
    // Password must have a lower and upper case plus a number and special character
    let password = "aB1%".repeat(random_count);

    Ok(password)
}

pub fn users(password: &String) -> Result<database::Users, AuthenticationError> {
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
    let password = SecretString::from(password.to_owned());
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

pub fn sessions(
    user: &database::Users,
    refresh_token: &domain::RefreshToken,
) -> Result<database::Sessions, AuthenticationError> {
    use chrono::SubsecRound;
    use std::time;
    use fake::faker::boolean::en::Boolean;
    use fake::faker::chrono::en::DateTime;
    use fake::faker::internet::en::IPv4;
    use fake::Fake;

    // Generate random Uuid V7
    let random_id = uuid_v7();

    // Take ownership of the user_id
    let user_id = user.id.to_owned();

    // Generate random login time
    let random_login_on: DateTime<Utc> = DateTime().fake();
    let random_logged_in_at = random_login_on.round_subsecs(0);

    // Generate random IPV4 address, with 25% chance of being None
    let random_ip: Ipv4Addr = IPv4().fake();
    // Convert IPV4 to an i32 to be consistent with Postgres INT type
    let random_ip = u32::from(random_ip) as i32;
    let random_login_ip = if Boolean(4).fake() {
        Some(random_ip)
    } else {
        None
    };

    // Generate a random session expiration date between 1 and 30 days.
    let random_duration_days = (1..30).fake::<u64>();
    let duration = time::Duration::from_secs(random_duration_days * 24 * 60 * 60);
    let random_expires_on = random_logged_in_at + duration;

    // Generate random boolean value
    let random_is_active: bool = Boolean(4).fake();

    // Generate random login time
    let random_logout = random_logged_in_at.round_subsecs(0);
    let random_logged_out_at = if Boolean(4).fake() {
        Some(random_logout)
    } else {
        None
    };

    // Generate random IPV4 address, with 25% chance of being None
    let random_ip: Ipv4Addr = IPv4().fake();
    // Convert IPV4 to an i32 to be consistent with Postgres INT type
    let random_ip = u32::from(random_ip) as i32;
    let random_logout_ip = if Boolean(4).fake() {
        Some(random_ip)
    } else {
        None
    };

    // Build the mock session instance
    let mock_session = database::Sessions {
        id: random_id,
        user_id,
        logged_in_at: random_logged_in_at,
        login_ip: random_login_ip,
        expires_on: random_expires_on,
        refresh_token: refresh_token.to_owned(),
        is_active: random_is_active,
        logged_out_at: random_logged_out_at,
        logout_ip: random_logout_ip,
    };

    Ok(mock_session)
}
