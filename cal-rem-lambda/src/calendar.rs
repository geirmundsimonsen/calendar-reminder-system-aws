use lambda_runtime::Error;
use std::env::var;
use crate::{get_default_headers, Header, Response};
use crate::parser::parse_calendar_file;
use crate::s3::{BrowserCachedData, get_object_as_string_if_etags_differ};

pub async fn get_calendar_events(etag: Option<String>) -> Result<Response, Error> {
    let str = get_object_as_string_if_etags_differ(var("S3_MAIN_BUCKET")?, "calendar.txt".to_string(), etag).await?;

    let mut headers = get_default_headers();

    Ok(match str {
        BrowserCachedData::InCache => {
            Response { status_code: 304, headers, body: "".to_string()}
        },
        BrowserCachedData::NotInCache { data, etag } => {
            headers.insert(Header::ETag, etag);
            Response { status_code: 200, headers, body: serde_json::to_string(&parse_calendar_file(&data))?}
        }
    })
}