//-- ./tests/api/users/delete.rs

// #![allow(unused)] // For beginning only.

use sqlx::{Pool, Postgres};

use authentication_microservice::rpc::proto::DeleteUserRequest;

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

//TODO: Add error case tests

#[sqlx::test]
async fn delete_returns_rows_deleted(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password = helpers::mocks::password()?;

    // Generate random user, passing in the password string
    let random_user = helpers::mocks::users(&random_password)?;

    // Insert user into the database
    let _database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    //-- Execute Test (Act)
    // Build request message
    let request_message = DeleteUserRequest {
        id: random_user.id.to_string(),
    };

    // Send request to the server with a response message being sent back
    let response_message = tonic_client
        .users()
        .delete(request_message)
        .await?
        .into_inner();

    //-- Checks (Assertions)
    // Confirm the database entry is removed
    assert_eq!(response_message.rows_affected, 1);

    Ok(())
}
