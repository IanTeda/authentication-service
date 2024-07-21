//-- ./tests/api/users/read.rs

// #![allow(unused)] // For beginning only.

use fake::Fake;
use sqlx::{Pool, Postgres};

use authentication_microservice::{
    database,
    rpc::proto::{ReadUserRequest, UserIndexRequest},
};

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

//TODO: Add error case tests

#[sqlx::test]
async fn id_returns_user(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Generate random user data and insert into database for testing
    let random_password_original = helpers::mocks::password()?;
    let random_user = helpers::mocks::users(&random_password_original)?;
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
    let response_message =
        tonic_client.users().read(request).await?.into_inner();
    // println!("{response_message:#?}");

    //-- Checks (Assertions)
    // User id should be equal
    assert_eq!(database_record.id.to_string(), response_message.id);

    // User email should be equal
    assert_eq!(database_record.email.as_ref(), response_message.email);

    // Username should be equal
    assert_eq!(database_record.name.as_ref(), response_message.name);

    // User role should be equal
    assert_eq!(database_record.role.as_ref(), response_message.role);

    // User is_active should be equal
    assert_eq!(database_record.is_active, response_message.is_active);

    // User is verified should be equal
    assert_eq!(database_record.is_verified, response_message.is_verified);

    // User created on should be equal
    assert_eq!(
        database_record.created_on.to_string(),
        response_message.created_on
    );
    Ok(())
}

//TODO: Fix edge case tests that fail
/// Check the read user index returns a collection of users
#[sqlx::test]
async fn index_returns_users(pool: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    // Get a random number between 10 and 30
    let random_count: i64 = (10..30).fake::<i64>();
    // println!("{random_count:#?}");

    // Initiate vector to store users for assertion
    let mut test_vec: Vec<database::Users> = Vec::new();

    // Iterate over the random count generating a user and adding, inserting it
    // into the database and pushing the response to the vector
    for _count in 0..random_count {
        // Generate random user data and insert into database for testing
        let random_password_original = helpers::mocks::password()?;
        let random_user = helpers::mocks::users(&random_password_original)?;
        let database_record = random_user.insert(&pool).await?;
        test_vec.push(database_record);
    }

    // Spawn Tonic test server
    let tonic_server = helpers::TonicServer::spawn_server(&pool).await?;

    // Spawn Tonic test client
    let mut tonic_client = helpers::TonicClient::spawn_client(&tonic_server).await?;

    //-- Execute Test (Act)
    // Generate a random limit and offset based on number of user entries
    let random_limit = (1..random_count).fake::<i64>();
    let random_offset = (1..random_count).fake::<i64>();

    // Build Tonic request message
    let request_message = UserIndexRequest {
        limit: random_limit,
        offset: random_offset,
    };
    // println!("{request_message:#?}");

    // Send request to tonic server and get a response message
    let response_message = tonic_client
        .users()
        .index(request_message)
        .await?
        .into_inner();
    // println!("{response:#?}");

    // Get the user index from the response message
    let index = response_message.users;

    //-- Checks (Assertions)
    let count_less_offset: i64 = random_count - random_offset;

    let expected_records = if count_less_offset < random_limit {
        count_less_offset
    } else {
        random_limit
    };

    // Check the number of returned users equals the limit and offset parameters
    assert_eq!(expected_records, index.len() as i64);

    Ok(())
}
