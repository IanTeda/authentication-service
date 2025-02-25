use sqlx::{Pool, Postgres};
use uuid::Uuid;

use authentication_service::domain;
use authentication_service::rpc::proto::{AuthenticationRequest, RefreshRequest};

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn returns_access_refresh_access(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password = helpers::mocks::password()?;
    let random_user = helpers::mocks::users(&random_password)?;
    let _database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Build tonic request
    let request = tonic::Request::new(AuthenticationRequest {
        email: random_user.email.to_string(),
        password: random_password.to_string(),
    });

    // Send tonic client request to server
    let response = tonic_client.authentication()
        .authentication(request)
        .await?
        .into_inner();

    //-- Execute Test (Act)
    // Build tonic request
    let request = tonic::Request::new(RefreshRequest {
        refresh_token: response.refresh_token,
    });

    // Send tonic client request to server
    let response = tonic_client.authentication().refresh(request).await?.into_inner();
    // println!("{response:#?}");

    //-- Checks (Assertions)
    // Get token secret
    let token_secret = &tonic_server.config.application.token_secret;
    let token_secret = token_secret.to_owned();

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