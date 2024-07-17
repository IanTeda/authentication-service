// #![allow(unused)] // For beginning only.

use secrecy::Secret;
use sqlx::{Pool, Postgres};

use personal_ledger_backend::{domain, rpc::ledger::{
    authentication_client::AuthenticationClient, LoginRequest, UpdatePasswordRequest,
}};
use uuid::Uuid;

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn returns_access_refresh_access(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password_original = helpers::mocks::password()?;
    let mut random_user = helpers::mocks::user_model(&random_password_original)?;
    random_user.is_active = true;
    random_user.is_verified = true;
    let _database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(database).await?;

    // Build Tonic Authentication Client
    let mut authentication_client =
        AuthenticationClient::new(tonic_server.clone().client_channel().await?);

    // Build tonic login request
    let login_request = tonic::Request::new(LoginRequest {
        email: random_user.email.to_string(),
        password: random_password_original.to_string(),
    });

    // Send tonic client login request to server
    let response = authentication_client
        .login(login_request)
        .await?
        .into_inner();
    // println!("Response: {}", response.access_token);

    //-- Execute Test (Act)
    // Generate a new random password
    let random_password_update = helpers::mocks::password()?;

    // Build Update Password Request
    let mut request = tonic::Request::new(UpdatePasswordRequest {
        email: random_user.email.to_string(),
        password_original: random_password_original.to_string(),
        password_new: random_password_update.to_string(),
    });

    // Append access token to request
    request
        .metadata_mut()
        .append("access-token", response.access_token.parse().unwrap());

    // Send update password request to server
    let response = authentication_client
        .update_password(request)
        .await?
        .into_inner();

    //-- Checks (Assertions)
    // Get Token Claim Secret before Tonic Client takes ownership of the server instance
    let token_secret = &tonic_server.clone().config.application.token_secret;
    let token_secret = Secret::new(token_secret.to_owned());

    // Build Token Claims from token responses
    let access_token_claim =
        domain::TokenClaim::from_token(&response.access_token, &token_secret)
            .await?;

    let refresh_token_claim =
        domain::TokenClaim::from_token(&response.refresh_token, &token_secret)
            .await?;

    // Confirm User IDs (uuids) are the same
    assert_eq!(
        Uuid::parse_str(&access_token_claim.sub)?,
        random_user.id
    );
    assert_eq!(
        Uuid::parse_str(&refresh_token_claim.sub)?,
        random_user.id
    );

    // Confirm Token Claims
    assert_eq!(&access_token_claim.jty, "Access");
    assert_eq!(&refresh_token_claim.jty, "Refresh");

    Ok(())
}
