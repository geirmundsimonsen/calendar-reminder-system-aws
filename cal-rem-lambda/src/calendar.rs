use lambda_runtime::Error;
use maplit::hashmap;
use std::env::var;
use crate::Header;
use crate::Response;
use crate::parser::parse_calendar_file;
use crate::s3::{BrowserCachedData, get_object_as_string_if_etags_differ};

pub async fn get_calendar_events(etag: Option<String>) -> Result<Response, Error> {
    let str = get_object_as_string_if_etags_differ(var("S3_MAIN_BUCKET")?, "calendar.txt".to_string(), etag).await?;

    Ok(match str {
        BrowserCachedData::InCache => {
            Response { status_code: 304, headers: hashmap! {
                Header::ContentType => "application/json".to_string(),
                Header::AccessControlAllowOrigin => "*".to_string(),
                Header::AccessControlAllowHeaders => "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token".to_string(),
                Header::AccessControlAllowMethods => "OPTIONS,POST,GET".to_string(),
            }, body: "".to_string()}
        },
        BrowserCachedData::NotInCache { data, etag } => {
            Response { status_code: 200, headers: hashmap! {
                Header::ContentType => "application/json".to_string(),
                Header::ETag => etag,
                Header::AccessControlAllowOrigin => "*".to_string(),
                Header::AccessControlAllowHeaders => "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token".to_string(),
                Header::AccessControlAllowMethods => "OPTIONS,POST,GET".to_string(),
            }, body: serde_json::to_string(&parse_calendar_file(&data))?}
        }
    })
}