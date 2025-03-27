// #![allow(unused)] // For development only

use cookie::Cookie;
use http::header::COOKIE;
use http::HeaderMap;
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
async fn returns_access_refresh_access(database: Pool<Postgres>) -> Result<()> {
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
    let refresh_response = tonic_client
        .authentication()
        .refresh(request)
        .await?
        .into_inner();
    // println!("{response:#?}");

    //-- 3. Checks (Assertions)
    // Get token secret form server configuration
    let token_secret = &tonic_server.config.application.token_secret;
    let token_secret = token_secret.to_owned();

    // Get JWT issuer from server configuration
    let issuer = &tonic_server.config.application.ip_address;

    // Build Access Token Claims from token responses
    let access_token_claim = domain::TokenClaim::parse(
        &refresh_response.access_token,
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
    let logins =
        database::Logins::index_user(&random_user.id, &10, &0, &database).await?;
    assert_eq!(random_user.id, logins[0].user_id);

    // Confirm Session is in the database
    let sessions =
        database::Sessions::index_from_user_id(&random_user.id, &10, &0, &database)
            .await?;
    assert_eq!(random_user.id, sessions[0].user_id);

    //-- 4. Return Ok

    //TODO: Check database revokes all others

    Ok(())
}
