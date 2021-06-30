use lambda_runtime::Error;
use crate::parser::get_calendar_entries_from_file;

pub async fn get_calendar_events() -> Result<String, Error> {
    let calendar_events = get_calendar_entries_from_file().await?;
    Ok(serde_json::to_string(&calendar_events)?)
}