use crate::GarageDoorStatus;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error as DynamoDbError};
use lambda_http::{Body, Error as LambdaError, Request, Response};
use std::collections::HashMap;

const TABLE_NAME: &str = "garage_status";
const DEFAULT_GARAGE: &str = "main_garage";
const PARTITION_KEY: &str = "garage_name";
const STATUS_KEY: &str = "status";

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, LambdaError> {
    let body = event.body();

    // Convert body to string
    let garage_status: GarageDoorStatus = match body {
        Body::Text(text) => serde_json::from_str(text)?,
        Body::Binary(bytes) => serde_json::from_slice(bytes)?,
        Body::Empty => {
            return Ok(Response::builder()
                .status(400)
                .body(Body::from(r#"{"error": "Request body required"}"#))?)
        }
    };

    println!("Received: {:?}", garage_status);
    match update_garage_door_status(&garage_status).await {
        Ok(_) => {
            let resp = Response::builder().status(200).body(Body::Empty)?;
            Ok(resp)
        }
        Err(e) => {
            println!("DynamoDB error: {:?}", e);
            Ok(Response::builder()
                .status(500)
                .body(Body::from(format!(r#"{{"error": "Database error: {}"}}"#, e)))?)
        }
    }
}

async fn update_garage_door_status(garage_status: &GarageDoorStatus) -> Result<(), DynamoDbError> {
    let config = aws_config::load_from_env().await;
    let dynamodb_client = Client::new(&config);

    let item = HashMap::from([
        (
            PARTITION_KEY.to_string(),
            AttributeValue::S(DEFAULT_GARAGE.to_string()),
        ),
        (
            STATUS_KEY.to_string(),
            AttributeValue::S(garage_status.status.to_string()),
        ),
    ]);

    dynamodb_client
        .put_item()
        .table_name(TABLE_NAME)
        .set_item(Some(item))
        .send()
        .await?;

    println!("Garage door status {} saved!", garage_status.status);
    Ok(())
}
