//-- ./tests/api/users/create.rs

// #![allow(unused)] // For beginning only.

use sqlx::{Pool, Postgres};

use authentication_microservice::rpc::proto::CreateUserRequest;

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

//TODO: Add error case tests

#[sqlx::test]
async fn returns_created_user(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Generate a random password string
    let random_password = helpers::mocks::password()?;

    // Generate a random user for testing passing in the random password string
    let random_user = helpers::mocks::users(&random_password)?;
    // println!("{random_user:#?}");

    //-- Execute Test (Act)
    // Build rpc request message
    let request_message = CreateUserRequest {
        email: random_user.email.to_string(),
        name: random_user.name.to_string(),
        password: random_password,
        role: random_user.role.to_string(),
        is_active: random_user.is_active,
        is_verified: random_user.is_verified,
    };

    // Build tonic request
    let request = tonic::Request::new(request_message);

    // Send request to tonic server and get response message
    let response_message = tonic_client
        .users()
        .create_user(request)
        .await?
        .into_inner();
    // println!("{response_message:#?}");

    //-- Checks (Assertions)
    // User id should not be equal as the server will generate
    assert_ne!(random_user.id.to_string(), response_message.id);

    // User email should be equal
    assert_eq!(random_user.email.as_ref(), response_message.email);

    // User name should be equal
    assert_eq!(random_user.name.as_ref(), response_message.name);

    // User role should be equal
    assert_eq!(random_user.role.as_ref(), response_message.role);

    // User is_active should be equal
    assert_eq!(random_user.is_active, response_message.is_active);

    // User is verified should be equal
    assert_eq!(random_user.is_verified, response_message.is_verified);

    // User created on should not be equal as the server will generate
    assert_ne!(
        random_user.created_on.to_string(),
        response_message.created_on
    );

    Ok(())
}
