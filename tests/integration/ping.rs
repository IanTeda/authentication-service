// #![allow(unused)] // For beginning only.

use crate::helpers::*;

use personal_ledger_backend::rpc::proto::Empty;

use sqlx::{Pool, Postgres};

#[sqlx::test]
async fn ping_returns_pong(database: Pool<Postgres>) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = spawn_test_server(database).await?;
    // Give the test server a few ms to become available
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    //-- Execute Test (Act)
    let mut tonic_client = spawn_test_client(tonic_server.address).await?;
    let request_empty = tonic::Request::new(Empty {});
    let response = tonic_client.ping(request_empty)
        .await?
        .into_inner();
    // println!("{response:#?}");

    //-- Checks (Assertions)
    assert_eq!(
        response.message, 
        "Pong..."
    );

    Ok(())
}