//-- ./tests/api/logins/create.rs

// #![allow(unused)] // For beginning only.

use sqlx::{Pool, Postgres};

use authentication_microservice::rpc::proto::LoginsCreateRequest;

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

//TODO: Add error case tests

// #[sqlx::test]
// async fn returns_created_logins(database: Pool<Postgres>) -> Result<()> {
//     //-- Setup and Fixtures (Arrange)
//     // Spawn Tonic test server
//     let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

//     // Spawn Tonic test client
//     let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

//     // Generate a random password string
//     let random_password = helpers::mocks::password()?;

//     // Generate a random user for testing passing in the random password string
//     let random_user = helpers::mocks::users(&random_password)?;
//     // println!("{random_user:#?}");

//     //-- Execute Test (Act)

//     //-- Checks (Assertions)

//     //-- Return
//     Ok(())
// }

#[sqlx::test]
async fn returns_created_logins(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Generate a random password string
    let random_password = helpers::mocks::password()?;

    // Generate a random user for testing passing in the random password string
    let random_user = helpers::mocks::users(&random_password)?;
    let random_user = random_user.insert(&database).await?;
    // println!("{random_user:#?}");

    let random_login = helpers::mocks::logins(&random_user.id)?;

    //-- Execute Test (Act)
    let request_message = LoginsCreateRequest {
        user_id: random_login.user_id.to_string(),
        login_on: random_login.login_on.to_string(),
        login_ip: random_login.login_ip,
    };

    // Build new Tonic request
    let request = tonic::Request::new(request_message);

    // Send request to tonic server and get response message
    let response_message = tonic_client
        .logins()
        .create(request)
        .await?
        .into_inner();
    // println!("{response_message:#?}");

    //-- Checks (Assertions)
    // Login id should not be equal as the server will generate
    assert_ne!(random_login.id.to_string(), response_message.id);

    // Login user id should be equal
    assert_eq!(random_login.user_id.to_string(), response_message.user_id);

    // Login on should not be equal as the server will generate
    assert_ne!(random_login.login_on.to_string(), response_message.login_on);

    // Login login_ip should equal
    assert_eq!(random_login.login_ip, response_message.login_ip);

    //-- Return
    Ok(())
}

