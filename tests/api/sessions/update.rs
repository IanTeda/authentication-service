//-- ./tests/api/refresh_tokens/update.rs

// #![allow(unused)] // For beginning only.

use fake::Fake;
use sqlx::{Pool, Postgres};

use authentication_service::rpc::proto::{Empty, SessionsRevokeRequest, SessionsRevokeUserRequest};

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn revoke_returns_int(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Generate a random password string
    let random_password = helpers::mocks::password()?;

    // Generate a random user for testing passing in the random password string
    let random_user = helpers::mocks::users(&random_password)?;

    // Insert random User into the database
    let random_user = random_user.insert(&database).await?;

    // Generate a random Session
    let random_refresh_token = helpers::mocks::sessions(&random_user)?;

    // Insert random Session into the database
    let random_refresh_token = random_refresh_token.insert(&database).await?;

    //-- Execute Test (Act)
    // Build rpc request message
    let request_message = SessionsRevokeRequest {
        id: random_refresh_token.id.to_string(),
    };
    // println!("{request_message:#?}");

    // Send request to the server with a response message being sent back
    let response_message = tonic_client
        .sessions()
        .revoke(request_message)
        .await?
        .into_inner();
    // println!("{response_message:#?}");

    //-- Checks (Assertions)
    // Confirm the database entry is removed
    assert_eq!(response_message.rows_affected, 1);

    Ok(())
}

#[sqlx::test]
async fn revoke_users_returns_int(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Generate a random password string
    let random_password = helpers::mocks::password()?;

    // Generate a random user for testing passing in the random password string
    let random_user = helpers::mocks::users(&random_password)?;

    // Insert random User into the database
    let random_user = random_user.insert(&database).await?;

    // Generate a random Session
    let random_refresh_token = helpers::mocks::sessions(&random_user)?;

    // Insert random Session into the database
    let _database_record = random_refresh_token.insert(&database).await?;

    //-- Execute Test (Act)
    // Build rpc request message
    let request_message = SessionsRevokeUserRequest {
        user_id: random_user.id.to_string(),
    };

    // Send request to the server with a response message being sent back
    let response_message = tonic_client
        .sessions()
        .revoke_user(request_message)
        .await?
        .into_inner();

    //-- Checks (Assertions)
    // Confirm the database entry is removed
    assert_eq!(response_message.rows_affected, 1);

    Ok(())
}

#[sqlx::test]
async fn revoke_all_returns_int(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Generate a random password string
    let random_password = helpers::mocks::password()?;

    // Generate a random user for testing passing in the random password string
    let random_user = helpers::mocks::users(&random_password)?;

    // Insert random User into the database
    let random_user = random_user.insert(&database).await?;

    // Generate and insert a random number of Sessions
    let random_count: i64 = (10..30).fake::<i64>();
    for _count in 0..random_count {
        // Generate a random Session
        let random_refresh_token = helpers::mocks::sessions(&random_user)?;

        // Insert random Session into the database
        let _random_refresh_token = random_refresh_token.insert(&database).await?;
    }

    //-- Execute Test (Act)
    // Build rpc request message
    let request_message = Empty {};

    // Send request to the server with a response message being sent back
    let response_message = tonic_client
        .sessions()
        .revoke_all(request_message)
        .await?
        .into_inner();

    //-- Checks (Assertions)
    // Confirm the database entry is removed
    assert_eq!(response_message.rows_affected, random_count);

    Ok(())
}
