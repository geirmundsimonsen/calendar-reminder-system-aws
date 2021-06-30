use lambda_runtime::Error;
use std::env::var;
use crate::matrix::Matrix;
use crate::parser::get_calendar_entries_from_file;
use crate::notify::{create_notifications_from_calendar, get_notifications_within_time_window};
use crate::s3::{get_object_as_string, save_string_as_object};
use chrono::prelude::*;

pub async fn run_notifier() -> Result<(), Error> {
    let now = Utc::now().timestamp();

    let previous_now = get_object_as_string(var("S3_MAIN_BUCKET")?, "notification-last-run-timestamp.txt".to_string()).await
        .map_or(now - 3600, |s| {
            s.parse::<i64>().unwrap_or(now - 3600)
        });

        
    let notifications = create_notifications_from_calendar(&get_calendar_entries_from_file().await?);
    let notifications_within_time_window = get_notifications_within_time_window(&notifications, Utc::now().timestamp(), previous_now);
    
    println!("{:?}", notifications_within_time_window);

    let messages: Vec<String> = notifications_within_time_window.iter().map(|notification| {
        notification.msg.clone()
    }).collect();

    let matrix = Matrix { server: var("MATRIX_SERVER")? };

    matrix.authenticate_and_send_messages_to_room(
        &var("MATRIX_USER")?,
        &var("MATRIX_PW")?,
        &var("MATRIX_REMINDER_ROOM")?,
        messages
    ).await;

    save_string_as_object(now.to_string(), var("S3_MAIN_BUCKET")?, "notification-last-run-timestamp.txt".to_string()).await?;

    Ok(())
}