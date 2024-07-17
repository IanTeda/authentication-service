use personal_ledger_backend::{domain, rpc::ledger::{authentication_client::AuthenticationClient, LoginRequest, RefreshRequest}};
use secrecy::Secret;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn returns_access_refresh_access(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password = helpers::mocks::password()?;
    let random_user = helpers::mocks::user_model(&random_password)?;
    let _database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    let token_secret = &tonic_server.clone().config.application.token_secret;

    // Build Tonic user client, with authentication intercept
    let mut authentication_client = AuthenticationClient::new(
        tonic_server.client_channel().await?
    );

    // Build tonic request
    let request = tonic::Request::new(LoginRequest {
        email: random_user.email.to_string(),
        password: random_password.to_string(),
    });

    // Send tonic client request to server
    let response = authentication_client
        .login(request)
        .await?
        .into_inner();

    //-- Execute Test (Act)
    // Build tonic request
    let request = tonic::Request::new(RefreshRequest {
        refresh_token: response.refresh_token,
    });

    // Send tonic client request to server
    let response = authentication_client.refresh(request).await?.into_inner();
    // println!("{response:#?}");

    //-- Checks (Assertions)
    // Get Token Claim Secret
    let token_secret = Secret::new(token_secret.to_owned());

    // Build Token Claims
    let access_token_claim =
        domain::TokenClaim::from_token(&response.access_token, &token_secret)?;

    let refresh_token_claim =
        domain::TokenClaim::from_token(&response.refresh_token, &token_secret)?;

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

    //TODO: Check database revokes all others

    Ok(())
}