use crate::{GarageDoorStatus, Status};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error as DynamoDbError};
use lambda_http::http::Method;
use lambda_http::{Body, Error as LambdaError, Request, Response};
use std::collections::HashMap;

const TABLE_NAME: &str = "garage_status";
const DEFAULT_GARAGE: &str = "main_garage";
const PARTITION_KEY: &str = "garage_name";
const STATUS_KEY: &str = "status";

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, LambdaError> {
    let config = aws_config::load_from_env().await;
    let dynamodb_client = Client::new(&config);

    match event.method() {
        &Method::GET => match get_garage_door_status(&dynamodb_client).await {
            Ok(garage_status) => {
                let json_body = serde_json::to_string(&garage_status)?;
                println!("Response: {}", json_body);
                let resp = Response::builder()
                    .status(200)
                    .body(Body::from(json_body))?;
                Ok(resp)
            }
            Err(e) => {
                println!("DynamoDB error: {:?}", e);
                Ok(Response::builder().status(500).body(Body::from(format!(
                    r#"{{"error": "Database error: {}"}}"#,
                    e
                )))?)
            }
        },
        &Method::POST => {
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
            match update_garage_door_status(&dynamodb_client, &garage_status).await {
                Ok(_) => {
                    let resp = Response::builder().status(200).body(Body::Empty)?;
                    Ok(resp)
                }
                Err(e) => {
                    println!("DynamoDB error: {:?}", e);
                    Ok(Response::builder().status(500).body(Body::from(format!(
                        r#"{{"error": "Database error: {}"}}"#,
                        e
                    )))?)
                }
            }
        }
        _ => Ok(Response::builder()
            .status(405)
            .body(Body::from(r#"{"error": "Method not allowed"}"#))?),
    }
}

async fn get_garage_door_status(
    dynamodb_client: &Client,
) -> Result<Option<GarageDoorStatus>, DynamoDbError> {
    let result = dynamodb_client
        .get_item()
        .table_name(TABLE_NAME)
        .key(PARTITION_KEY, AttributeValue::S(DEFAULT_GARAGE.to_string()))
        .send()
        .await?;

    if let Some(item) = result.item {
        if let Some(status_attr) = item.get(STATUS_KEY) {
            if let AttributeValue::S(status_str) = status_attr {
                let status = match status_str.as_str() {
                    "OPEN" => Status::Open,
                    "CLOSED" => Status::Closed,
                    _ => return Ok(None), // Invalid status
                };
                println!("Retrieved Garage Door Status {}", status);
                return Ok(Some(GarageDoorStatus { status }));
            }
        }
    }
    Ok(None)
}

async fn update_garage_door_status(
    dynamodb_client: &Client,
    garage_status: &GarageDoorStatus,
) -> Result<(), DynamoDbError> {
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
