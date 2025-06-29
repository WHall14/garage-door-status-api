use crate::{GarageDoorStatus, Status};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error as DynamoDbError};
use lambda_http::{Body, Error, Request, Response};

const TABLE_NAME: &str = "garage_status";
const DEFAULT_GARAGE: &str = "main_garage";
const PARTITION_KEY: &str = "garage_name";
const STATUS_KEY: &str = "status";

pub(crate) async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    match get_garage_door_status().await {
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
    }
}

async fn get_garage_door_status() -> Result<Option<GarageDoorStatus>, DynamoDbError> {
    let config = aws_config::load_from_env().await;
    let dynamodb_client = Client::new(&config);

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
