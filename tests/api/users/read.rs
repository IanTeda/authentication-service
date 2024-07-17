// #![allow(unused)] // For beginning only.

use personal_ledger_backend::rpc::ledger::ReadUserRequest;
use sqlx::{Pool, Postgres};

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn id_returns_user(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password_original = helpers::mocks::password()?;
    let random_user = helpers::mocks::user_model(&random_password_original)?;
    let database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    let request_message = ReadUserRequest { 
        id: random_user.id.to_string(),
    };

    //-- Execute Test (Act)
    // Build tonic request
    let request = tonic::Request::new(request_message);

    // Send request to tonic server and get a response message
    let response_message = tonic_client.users().read_user(request).await?.into_inner();
    // println!("{response_message:#?}");

    //-- Checks (Assertions)
    // User id should be equal
    assert_eq!(database_record.id.to_string(), response_message.id);

    // User email should be equal
    assert_eq!(database_record.email.as_ref(), response_message.email);

    // User name should be equal
    assert_eq!(database_record.name.as_ref(), response_message.name);

    // User role should be equal
    assert_eq!(database_record.role.as_ref(), response_message.role);

    // User is_active should be equal
    assert_eq!(database_record.is_active, response_message.is_active);

    // User is verified should be equal
    assert_eq!(database_record.is_verified, response_message.is_verified);

    // User created on should be equal
    assert_eq!(database_record.created_on.to_string(), response_message.created_on);
    Ok(())
}