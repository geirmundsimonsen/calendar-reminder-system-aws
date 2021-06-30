use serde::{Deserialize, Serialize};
use lambda_runtime::{handler_fn, Context, Error};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use cal_rem_wasm_com::{Command, RequestBody};
use crate::todo::get_todo_entries;
use crate::calendar::get_calendar_events;
use crate::notifier::run_notifier;

mod calendar;
mod todo;
mod parser;
mod matrix;
mod notifier;
mod notify;
mod s3;

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
    pub headers: std::collections::HashMap<String, String>,
    #[serde(rename = "httpMethod")]
    pub http_method: String,
}

#[derive(Serialize, Deserialize)]
pub struct Headers {
    #[serde(rename = "Content-Type")]
    pub content_type: String,
    #[serde(rename = "Access-Control-Allow-Origin")]
    pub access_control_allow_origin: String,
    #[serde(rename = "Access-Control-Allow-Headers")]
    pub access_control_allow_headers: String,
    #[serde(rename = "Access-Control-Allow-Methods")]
    pub access_control_allow_methods: String,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    #[serde(rename = "statusCode")]
    pub status_code: u32,
    pub headers: Headers,
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
            Ok(Response { status_code: 200, headers: Headers {
                content_type: "application/json".to_string(),
                access_control_allow_origin: "*".to_string(),
                access_control_allow_headers: "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token".to_string(),
                access_control_allow_methods: "OPTIONS,POST,GET".to_string()
            }, body: "".to_string()})
        },
        Event::ApiGatewayRequest(api_gateway_request) => {
            if api_gateway_request.body.is_none() {
                if api_gateway_request.http_method == "OPTIONS" {
                    return Ok(Response { status_code: 200, headers: Headers {
                        content_type: "application/json".to_string(),
                        access_control_allow_origin: "*".to_string(),
                        access_control_allow_headers: "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token".to_string(),
                        access_control_allow_methods: "OPTIONS,POST,GET".to_string()
                    }, body: "".to_string()})
                }
            }
        
            let body: RequestBody = serde_json::from_str(&api_gateway_request.body.unwrap())?;
        
            let body = match body.command {
                Command::GetTodoEntries => get_todo_entries().await?,
                Command::GetCalendarEvents => get_calendar_events().await?,
            };
        
            return Ok(Response { status_code: 200, headers: Headers {
                content_type: "application/json".to_string(),
                access_control_allow_origin: "*".to_string(),
                access_control_allow_headers: "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token".to_string(),
                access_control_allow_methods: "OPTIONS,POST,GET".to_string()
            }, body})
        }
    }
}