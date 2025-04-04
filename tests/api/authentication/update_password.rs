// #![allow(unused)] // For beginning only.

use cookie::Cookie;
use sqlx::{Pool, Postgres};
// use uuid::Uuid;

// use authentication_service::domain;
use authentication_service::rpc::proto::{AuthenticationRequest, UpdatePasswordRequest};

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn returns_access_refresh_access(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password_original = helpers::mocks::password()?;
    let mut random_user = helpers::mocks::users(&random_password_original)?;
    random_user.is_active = true;
    random_user.is_verified = true;
    let _database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Build tonic login request
    let login_request_message = AuthenticationRequest {
        email: random_user.email.to_string(),
        password: random_password_original.to_string(),
    };
    // println!("{login_request_message:#?}");

    let login_request = tonic::Request::new(login_request_message);

    // Send tonic client login request to server
    let login_response_message = tonic_client
        .authentication()
        .authentication(login_request)
        .await?
        .into_inner();
    // println!("{login_response_message:#?}");

    //-- Execute Test (Act)
    // Generate a new random password
    let random_password_update = helpers::mocks::password()?;

    let update_password_request_message = UpdatePasswordRequest {
        email: random_user.email.to_string(),
        password_original: random_password_original.to_string(),
        password_new: random_password_update.to_string(),
    };

    let access_token = login_response_message.access_token.clone();
    let access_cookie = Cookie::new("access_token", access_token).to_string();

    // Build Update Password Request
    let mut update_password_request = tonic::Request::new(update_password_request_message);

    // Append access token from login to request
    update_password_request
        .metadata_mut()
        .insert("cookie", access_cookie.parse().unwrap());
    // println!("{update_password_request:#?}");

    // Send update password request to server
    let response = tonic_client
        .authentication()
        .update_password(update_password_request)
        .await?
        .into_inner();

    //-- Checks (Assertions)
    assert_eq!(response.success, true);
    assert_eq!(response.message, "Password updated successfully");

    Ok(())
}

#[sqlx::test]
async fn incorrect_original_password_returns_unauthenticated(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password_original = helpers::mocks::password()?;
    let mut random_user = helpers::mocks::users(&random_password_original)?;
    random_user.is_active = true;
    random_user.is_verified = true;
    let _database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Build tonic login request
    let login_request_message = AuthenticationRequest {
        email: random_user.email.to_string(),
        password: random_password_original.to_string(),
    };
    // println!("{request_message:#?}");

    let login_request = tonic::Request::new(login_request_message);

    // Send tonic client login request to server
    let login_response_message = tonic_client
        .authentication()
        .authentication(login_request)
        .await?
        .into_inner();
    // println!("{login_response_message:#?}");

    //-- Execute Test (Act)
    // Generate a new random password
    let random_password_update = helpers::mocks::password()?;

    // Generate a new password that is different from the original password
    let wrong_password_original = helpers::mocks::password()?;

    let update_password_request_message = UpdatePasswordRequest {
        email: random_user.email.to_string(),
        password_original: wrong_password_original.to_string(),
        password_new: random_password_update.to_string(),
    };

    // Build Update Password Request
    let mut update_password_request = tonic::Request::new(update_password_request_message);

    // Append access token from login to request
    update_password_request
        .metadata_mut()
        .append("access_token", login_response_message.access_token.parse().unwrap());

    // Send update password request to server
    let update_password_response = tonic_client
        .authentication()
        .update_password(update_password_request)
        .await
        .unwrap_err();

    //-- Checks (Assertions)
    // Confirm Tonic response status code
    assert_eq!(update_password_response.code(), tonic::Code::Unauthenticated);

    // Confirm Tonic response message
    assert_eq!(update_password_response.message(), "Authentication Failed!");
    Ok(())
}


#[sqlx::test]
async fn bad_new_password(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password_original = helpers::mocks::password()?;
    let mut random_user = helpers::mocks::users(&random_password_original)?;
    random_user.is_active = true;
    random_user.is_verified = true;
    let _database_record = random_user.insert(&database).await?;

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Build tonic login request
    let login_request_message = AuthenticationRequest {
        email: random_user.email.to_string(),
        password: random_password_original.to_string(),
    };
    // println!("{request_message:#?}");

    let login_request = tonic::Request::new(login_request_message);

    // Send tonic client login request to server
    let login_response_message = tonic_client
        .authentication()
        .authentication(login_request)
        .await?
        .into_inner();
    // println!("{login_response_message:#?}");

    //-- Execute Test (Act)
    // Generate a new random password
    let random_password_update = "bad_password";

    let update_password_request_message = UpdatePasswordRequest {
        email: random_user.email.to_string(),
        password_original: random_password_original.to_string(),
        password_new: random_password_update.to_string(),
    };

    // Build Update Password Request
    let mut update_password_request = tonic::Request::new(update_password_request_message);

    // Append access token from login to request
    update_password_request
        .metadata_mut()
        .append("access_token", login_response_message.access_token.parse().unwrap());

    // Send update password request to server
    let update_password_response = tonic_client
        .authentication()
        .update_password(update_password_request)
        .await
        .unwrap_err();
    // println!("{update_password_response:#?}");

    //-- Checks (Assertions)
    // Confirm Tonic response status code
    assert_eq!(update_password_response.code(), tonic::Code::Unauthenticated);

    // Confirm Tonic response message
    // assert_eq!(update_password_response.message(), "Authentication Failed!");
    Ok(())
}