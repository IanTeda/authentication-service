//-- ./tests/api/users/update.rs

// #![allow(unused)] // For beginning only.

use sqlx::{Pool, Postgres};

use authentication_microservice::rpc::proto::UpdateUserRequest;

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

//TODO: Add error case tests

#[sqlx::test]
async fn user_returns_update(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password_original = helpers::mocks::password()?;

    // Generate random user, passing in the password string
    let random_user_original =
        helpers::mocks::users(&random_password_original)?;

    // Insert user into the database
    let _database_record = random_user_original.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    //-- Execute Test (Act)
    // Generate update data
    let random_user_update = helpers::mocks::users(&random_password_original)?;

    // Build rpc request message
    let request_message = UpdateUserRequest {
        id: random_user_original.id.to_string(),
        email: random_user_update.email.to_string(),
        name: random_user_update.name.to_string(),
        role: random_user_update.role.to_string(),
        is_active: random_user_update.is_active,
        is_verified: random_user_update.is_verified,
    };

    // Build tonic request
    let request = tonic::Request::new(request_message);

    // Send request to tonic server and get response message
    let response_message = tonic_client
        .users()
        .update(request)
        .await?
        .into_inner();
    // println!("{response_message:#?}");

    //-- Checks (Assertions)
    // User id should equal the original id as update will not change this
    assert_eq!(random_user_original.id.to_string(), response_message.id);

    // User email should be equal
    assert_eq!(random_user_update.email.as_ref(), response_message.email);

    // Username should be equal
    assert_eq!(random_user_update.name.as_ref(), response_message.name);

    // User role should be equal
    assert_eq!(random_user_update.role.as_ref(), response_message.role);

    // User is_active should be equal
    assert_eq!(random_user_update.is_active, response_message.is_active);

    // User is verified should be equal
    assert_eq!(random_user_update.is_verified, response_message.is_verified);

    // User created should equal the original as update will not change this
    assert_eq!(
        random_user_original.created_on.to_string(),
        response_message.created_on
    );

    Ok(())
}
