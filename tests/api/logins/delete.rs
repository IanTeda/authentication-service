//-- ./tests/api/logins/delete.rs

// #![allow(unused)] // For beginning only.

use sqlx::{Pool, Postgres};

use authentication_microservice::rpc::proto::LoginsDeleteRequest;

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

//TODO: Add error case tests

#[sqlx::test]
async fn rows_of_logins_deleted(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&database).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    // Generate a random password string
    let random_password = helpers::mocks::password()?;

    // Generate a random user for testing passing in the random password string
    // and add to the database
    let random_user = helpers::mocks::users(&random_password)?;
    let random_user = random_user.insert(&database).await?;
    // println!("{random_user:#?}");

    // Generate a random login and add to database
    let random_login = helpers::mocks::logins(&random_user.id)?;
    let random_login = random_login.insert(&database).await?;

    //-- Execute Test (Act)
    // Generate a new Logins Delete Request
    let request_message = LoginsDeleteRequest {
        id: random_login.id.to_string(),
    };

    // Generate a new Tonic Request
    let request = tonic::Request::new(request_message);

    // Make a request of the Tonic server and get into the response
    let response_message = tonic_client.logins().delete(request).await?.into_inner();

    //-- Checks (Assertions)
    // Confirm the database entry is removed
    assert_eq!(response_message.rows_affected, 1);

    //-- Return
    Ok(())
}
