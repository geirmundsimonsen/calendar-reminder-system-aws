use lambda_runtime::{handler_fn, Context, Error};
use log::LevelFilter;
use maplit::hashmap;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use crate::todo::get_todo_entries;
use crate::calendar::get_calendar_events;
use crate::notifier::run_notifier;

mod calendar;
mod dynamodb;
mod matrix;
mod notifier;
mod notify;
mod parser;
mod s3;
mod todo;

/*
ApiGateway Request:
{
    "body":"{\"command\":\"GET_CALENDAR_EVENTS\",\"parameters\":\"\"}",
    "headers":{"Origin":"foo"},
    "httpMethod":"POST"
}

CloudWatch Event:
{
    "detail-type": "foo",
    "source": "bar"
}
*/

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Event {
    ApiGatewayRequest(ApiGatewayRequest),
    CloudWatchEvent(CloudWatchEvent),
}

#[derive(Serialize, Deserialize)]
pub struct CloudWatchEvent {
    #[serde(rename = "detail-type")]
    detail_type: String,
    source: String,
}

#[derive(Serialize, Deserialize)]
pub struct ApiGatewayRequest {
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
    #[serde(rename = "httpMethod")]
    pub http_method: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Header {
    #[serde(rename = "Access-Control-Allow-Headers")]
    AccessControlAllowHeaders,
    #[serde(rename = "Access-Control-Allow-Methods")]
    AccessControlAllowMethods,
    #[serde(rename = "Access-Control-Allow-Origin")]
    AccessControlAllowOrigin,
    #[serde(rename = "Cache-Control")]
    CacheControl,
    #[serde(rename = "Content-Type")]
    ContentType,
    ETag,
    Expires,
    #[serde(rename = "Last-Modified")]
    LastModified,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    #[serde(rename = "statusCode")]
    pub status_code: u32,
    pub headers: HashMap<Header, String>,
    pub body: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    let func = handler_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn my_handler(event: Event, _ctx: Context) -> Result<Response, Error> {
    return match event {
        Event::CloudWatchEvent(_cloud_watch_event) => {
            run_notifier().await?;
            Ok(Response { status_code: 200, headers: get_default_headers(), body: "".to_string()})
        },
        Event::ApiGatewayRequest(api_gateway_request) => {
            if api_gateway_request.body.is_none() {
                if api_gateway_request.http_method == "OPTIONS" {
                    return Ok(Response { status_code: 200, headers: get_default_headers(), body: "".to_string()})
                }
            }

            let etag = api_gateway_request.headers.get("if-none-match").and_then(|str| Some(str.clone()));
        
            if api_gateway_request.http_method == "GET" {
                return match api_gateway_request.path.as_str() {
                    "/get-all-calendar-entries" => {
                        get_calendar_events(etag).await
                    },
                    "/get-all-todo-entries" => {
                        get_todo_entries(etag).await
                    },
                    _ => {
                        Ok(Response { status_code: 404, headers: get_default_headers(), body: "Resource not found".to_string()})
                    }
                };
            }

            return Ok(Response { status_code: 404, headers: get_default_headers(), body: "Unknown command from client".to_string()})
        }
    }
}

pub fn get_default_headers() -> HashMap<Header, String> {
    hashmap! {
        Header::ContentType => "application/json".to_string(),
        Header::AccessControlAllowOrigin => "*".to_string(),
        Header::AccessControlAllowHeaders => "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token".to_string(),
        Header::AccessControlAllowMethods => "OPTIONS,POST,GET".to_string(),
    }
}