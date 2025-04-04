//-- ./tests/integration/authentication/login.rs

// #![allow(unused)] // For beginning only.

//! Module for integration testing the authentication endpoints
//!
//! Endpoints include
//!
//! * `authenticate`: Authenticate user email and password
//! * `refresh_authentication`: Refresh the bearer token after it expires
//! * `update_password`: Update my password
//! * `reset_password`: Reset my forgotten password
//! * `logout`: Log me out

use chrono::{DateTime, Utc};
use cookie::Cookie;
use fake::{faker::internet::en::SafeEmail, Fake};
use secrecy::Secret;
use sqlx::{Pool, Postgres};
use tonic::Code;
use uuid::Uuid;

use authentication_service::{database, domain, rpc::proto::LoginRequest};

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn returns_access_refresh_tokens(database: Pool<Postgres>) -> Result<()> {
    //-- 1. Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password = helpers::mocks::password()?;
    let mut random_user = helpers::mocks::users(&random_password)?;
    random_user.is_active = true; // Set to true for login testing
    random_user.is_verified = true; // Set to true for login testing
    let _database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    //-- 2. Execute Test (Act)
    let request_message = LoginRequest {
        email: random_user.email.to_string(),
        password: random_password.to_string(),
    };

    // Build tonic request
    let request = tonic::Request::new(request_message);

    // Send tonic client authentication request to server
    let (response_metadata, response_message, _response_extensions) = tonic_client
        .authentication()
        .login(request)
        .await?
        .into_parts();

    //-- 3. Checks (Assertions)
    // Get token secret form server configuration
    let token_secret = &tonic_server.config.application.token_secret;
    let token_secret = token_secret.to_owned();

    // Get JWT issuer from server configuration
    let issuer = &tonic_server.config.application.get_issuer();

    // Build Access Token Claims from token responses
    let access_token_claim = domain::TokenClaim::parse(
        &response_message.access_token,
        &token_secret,
        &issuer,
    )?;
    // println!("access_token_claim: {access_token_claim:#?}");

    // Get the refresh token from the response header (metadata)
    // Cannot use the RefreshToken from_header() method because the response use "set-cookie" key not
    // "cookie" that the browser sends in a request.
    // We are using get, not get_all because we know there will be only one cookie in the response header
    let set_cookie = response_metadata.get("set-cookie").unwrap().to_str()?;

    // Parse the cookie string into a Cookie object
    let cookie = Cookie::parse(set_cookie)?;

    // Get the refresh token string value from the cookie
    let refresh_token = cookie.value().to_string();

    // Decode the refresh token into a Token Claim for asserting
    let refresh_token_claim =
        domain::TokenClaim::parse(&refresh_token, &token_secret, &issuer)?;

    // Confirm User IDs (uuids) are the same
    assert_eq!(Uuid::parse_str(&access_token_claim.sub)?, random_user.id);
    assert_eq!(Uuid::parse_str(&refresh_token_claim.sub)?, random_user.id);

    // Confirm Token Claims
    assert_eq!(&access_token_claim.jty, "Access");
    assert_eq!(&refresh_token_claim.jty, "Refresh");

    // Confirm Login is in database
    // let logins = database::Logins::index_user(&random_user.id, &10, &0, &database).await?;
    let session = database::Sessions::from_token(&refresh_token, &database).await?;
    assert_eq!(random_user.id, session.user_id);

    // Confirm Session is in the database
    let sessions =
        database::Sessions::index_from_user_id(&random_user.id, &10, &0, &database)
            .await?;
    assert_eq!(random_user.id, sessions[0].user_id);

    //-- 4. Return Ok
    Ok(())
}

#[sqlx::test]
async fn default_user_login(database: Pool<Postgres>) -> Result<()> {
    //-- 1. Setup and Fixtures (Arrange)
    // I am add with the database migrations and should be updated on first load
    let default_password = "S3cret-Admin-Pas$word!".to_string();
    let default_user = database::Users {
        id: Uuid::parse_str("019071c5-a31c-7a0e-befa-594702122e75")?,
        email: domain::EmailAddress::parse("default_ams@teda.id.au")?,
        name: domain::UserName::parse("Admin")?,
        password_hash: domain::PasswordHash::parse(Secret::new(
            default_password.clone(),
        ))?,
        role: domain::UserRole::Admin,
        is_active: true,
        is_verified: true,
        created_on: DateTime::parse_from_rfc3339("2019-10-17T00:00:00.000000Z")?
            .with_timezone(&Utc),
    };

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    //-- 2. Execute Test (Act)
    let request_message = LoginRequest {
        email: default_user.email.to_string(),
        password: default_password,
    };

    // Build tonic request
    let request = tonic::Request::new(request_message);

    // Send tonic client login request to server
    let (response_metadata, response_message, _response_extensions) = tonic_client
        .authentication()
        .login(request)
        .await?
        .into_parts();

    //-- 3. Checks (Assertions)
    // Get token secret
    let token_secret = &tonic_server.config.application.token_secret;
    let token_secret = token_secret.to_owned();

    // Get JWT issuer from config
    let issuer = &tonic_server.config.application.get_issuer();

    // Build Token Claims from token responses
    let access_token_claim = domain::TokenClaim::parse(
        &response_message.access_token,
        &token_secret,
        &issuer,
    )?;

    // Get the refresh token from the response header (metadata)
    // Cannot use the RefreshToken from_header() method because the response use "set-cookie" key not
    // "cookie" that the browser sends in a request.
    // We are using get, not get_all because we know there will be only one cookie in the response header
    let set_cookie = response_metadata.get("set-cookie").unwrap().to_str()?;

    // Parse the cookie string into a Cookie object
    let cookie = Cookie::parse(set_cookie)?;

    // Get the refresh token string value from the cookie
    let refresh_token = cookie.value().to_string();

    // Decode the refresh token into a Token Claim for asserting
    let refresh_token_claim =
        domain::TokenClaim::parse(&refresh_token, &token_secret, &issuer)?;

    // Confirm User IDs (uuids) are the same
    assert_eq!(Uuid::parse_str(&access_token_claim.sub)?, default_user.id);
    assert_eq!(Uuid::parse_str(&refresh_token_claim.sub)?, default_user.id);

    // Confirm Token Claims
    assert_eq!(&access_token_claim.jty, "Access");
    assert_eq!(&refresh_token_claim.jty, "Refresh");

    // Confirm Login is in database
    // let logins = database::Logins::index_user(&default_user.id, &10, &0, &database).await?;
    let session = database::Sessions::from_token(&refresh_token, &database).await?;
    assert_eq!(default_user.id, session.user_id);

    // Confirm Session is in the database
    let sessions =
        database::Sessions::index_from_user_id(&default_user.id, &10, &0, &database)
            .await?;
    assert_eq!(default_user.id, sessions[0].user_id);

    //-- 4. Return
    Ok(())
}

#[sqlx::test]
async fn incorrect_password_returns_error(database: Pool<Postgres>) -> Result<()> {
    //-- 1. Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password = helpers::mocks::password()?;
    let random_user = helpers::mocks::users(&random_password)?;
    let _database_record = random_user.insert(&database).await?;

    // Generate an incorrect password
    let incorrect_password = helpers::mocks::password()?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    //-- 2. Execute Test (Act)
    // Build tonic request message
    let request_message = LoginRequest {
        email: random_user.email.to_string(),
        password: incorrect_password,
    };

    // Build tonic request
    let request = tonic::Request::new(request_message);

    // Send tonic client request to server
    let response = tonic_client
        .authentication()
        .login(request)
        .await
        .unwrap_err();
    // println!("{response:#?}");

    //-- 3. Checks (Assertions)
    // Confirm Tonic response status code
    assert_eq!(response.code(), Code::Unauthenticated);

    // Confirm Tonic response message
    assert_eq!(response.message(), "Authentication Failed!");

    //-- 4. Return
    Ok(())
}

#[sqlx::test]
async fn incorrect_email_returns_error(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password = helpers::mocks::password()?;
    let random_user = helpers::mocks::users(&random_password)?;
    let _database_record = random_user.insert(&database).await?;

    // Generate an incorrect password
    let incorrect_email = SafeEmail().fake();

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    //-- Execute Test (Act)
    // Build tonic request message
    let request_message = LoginRequest {
        email: incorrect_email,
        password: random_password,
    };

    // Build tonic request
    let request = tonic::Request::new(request_message);

    // Send tonic client request to server
    let response = tonic_client
        .authentication()
        .login(request)
        .await
        .unwrap_err();
    // println!("{response:#?}");

    //-- Checks (Assertions)
    // Confirm Tonic response status code
    assert_eq!(response.code(), Code::Unauthenticated);

    // Confirm Tonic response message
    assert_eq!(response.message(), "Authentication Failed!");

    Ok(())
}
