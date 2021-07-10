use cal_rem_shared::{Command, RequestBody};
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
            Ok(Response { status_code: 200, headers: hashmap! {
                Header::ContentType => "application/json".to_string(),
                Header::AccessControlAllowOrigin => "*".to_string(),
                Header::AccessControlAllowHeaders => "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token".to_string(),
                Header::AccessControlAllowMethods => "OPTIONS,POST,GET".to_string(),
            }, body: "".to_string()})
        },
        Event::ApiGatewayRequest(api_gateway_request) => {
            if api_gateway_request.body.is_none() {
                if api_gateway_request.http_method == "OPTIONS" {
                    return Ok(Response { status_code: 200, headers: hashmap! {
                        Header::ContentType => "application/json".to_string(),
                        Header::AccessControlAllowOrigin => "*".to_string(),
                        Header::AccessControlAllowHeaders => "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token".to_string(),
                        Header::AccessControlAllowMethods => "OPTIONS,POST,GET".to_string(),
                    }, body: "".to_string()})
                }
            }
        
            let body: RequestBody = serde_json::from_str(&api_gateway_request.body.unwrap())?;
        
            let body = match body.command {
                Command::GetTodoEntries => get_todo_entries().await?,
                Command::GetCalendarEvents => get_calendar_events().await?,
            };

            if let Some(etag) = api_gateway_request.headers.get("If-None-Match") { println!("if none match! {}", etag) }

            return Ok(Response { status_code: 200, headers: hashmap! {
                Header::ContentType => "application/json".to_string(),
                //Header::CacheControl => "max-age=3600000, must-revalidate".to_string(),
                //Header::LastModified => "Mon, 29 Jun 1998 02:28:12 GMT".to_string(),
                //Header::ETag => "1234".to_string(),
                //Header::Expires => "Mon, 26 Jul 2021 02:28:12 GMT".to_string(),
                Header::AccessControlAllowOrigin => "*".to_string(),
                Header::AccessControlAllowHeaders => "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token".to_string(),
                Header::AccessControlAllowMethods => "OPTIONS,POST,GET".to_string(),
            }, body})
        }
    }
}