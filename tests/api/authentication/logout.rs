#![allow(unused)] // For development only

use chrono::Duration;

use cookie::Cookie;
use fake::faker::company::en::CompanyName;
use fake::Fake;
use http::header::COOKIE;
use http::HeaderMap;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use secrecy::Secret;
use sqlx::{Pool, Postgres};
use tonic::metadata::MetadataMap;
use tonic::Request;
use uuid::Uuid;

use authentication_service::database;
use authentication_service::domain;
use authentication_service::rpc::proto::{AuthenticationRequest, Empty};

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn returns_logout_true(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user and insert into database for testing
    let random_password = helpers::mocks::password()?;
    let mut random_user = helpers::mocks::users(&random_password)?;
    random_user.is_active = true;
    random_user.is_verified = true;
    let _database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    //-- 2. Execute Test (Act)
    let auth_message = AuthenticationRequest {
        email: random_user.email.to_string(),
        password: random_password.to_string(),
    };

    // Build tonic request
    let auth_request = tonic::Request::new(auth_message);

    // Request authenticaoitn of random user email and password
    let (response_metadata, _response_message, _response_extensions) = tonic_client
        .authentication()
        .authentication(auth_request)
        .await?
        .into_parts();

    // Get the refresh cookie from the tonic response header
    let refresh_cookie = response_metadata.get("set-cookie").unwrap().to_str()?;

    // Parse the cookie header string into a Cookie object
    let refresh_cookie = Cookie::parse(refresh_cookie)?;

    // Strip out additoinal cookie detail and convert to key=vaule string
    let refresh_cookie = refresh_cookie.stripped().to_string();

    // Build tonic request message
    let request_message = Empty {};

    // Build a tonic request
    let mut request = Request::new(request_message);

    // Create a new http header map
    let mut http_header = HeaderMap::new();

    // Add refresh cookie to the http header map
    http_header.insert(COOKIE, refresh_cookie.parse().unwrap());

    // Add the http header to the rpc response
    *request.metadata_mut() = MetadataMap::from_headers(http_header);

    // Send token refresh request to server
    let logout_response_message: authentication_service::rpc::proto::LogoutResponse = tonic_client
        .authentication()
        .logout(request)
        .await?
        .into_inner();
    // println!("{logout_response_message:#?}");

    //-- 3. Checks (Assertions)
    assert_eq!(logout_response_message.success, true);

    assert_eq!(logout_response_message.message, "You are logged out");

    Ok(())
}


#[sqlx::test]
async fn incorrect_refresh_token_is_unauthorised(database: Pool<Postgres>) -> Result<()> {
    //-- 1. Setup and Fixtures (Arrange)
    // Generate random user and insert into database for testing
    let random_password = helpers::mocks::password()?;
    let mut random_user = helpers::mocks::users(&random_password)?;
    random_user.is_active = true;
    random_user.is_verified = true;
    let _database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Generate random secret string
    let secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 60);
    let secret = Secret::new(secret);

    //-- 2. Execute Test (Act)

    // Geneate a new random user to make a Refresh Token that is not in the database
    let new_random_user = helpers::mocks::users(&random_password)?;

    // Generate a random issuer for the incorrect Refresh Token
    let random_issuer = CompanyName().fake::<String>();

    // Generate a random duration or the incorrect Refresh Token
    let random_duration =
        std::time::Duration::from_secs(Duration::days(30).num_seconds() as u64);

    // Generate a new refresh token not in the database so authentication fails
    let incorrect_refresh_token = domain::RefreshToken::new(
        &secret,
        &random_issuer,
        &random_duration,
        &new_random_user,
    )?;

    // Build the incorrect Refresh Token cookie for fail authentication
    let incorrect_refresh_cookie = incorrect_refresh_token.build_cookie(&tonic_server.address, &random_duration);

    // Build tonic request message
    let request_message = Empty {};

    // Build a tonic request
    let mut request = Request::new(request_message);

    // Create a new http header map
    let mut http_header = HeaderMap::new();

    // Add refresh cookie to the http header map
    http_header.insert(COOKIE, incorrect_refresh_cookie.to_string().parse().unwrap());

    // Add the http header to the rpc response
    *request.metadata_mut() = MetadataMap::from_headers(http_header);

    // Send token refresh request to server
    let refresh_response = tonic_client
        .authentication()
        .logout(request)
        .await
        .unwrap_err();
    // println!("{refresh_response:#?}");

    //-- Checks (Assertions)
    // Confirm Tonic response status code
    assert_eq!(refresh_response.code(), tonic::Code::Unauthenticated);

    // Confirm Tonic response message
    assert_eq!(refresh_response.message(), "Authentication Failed!");


    Ok(())
}