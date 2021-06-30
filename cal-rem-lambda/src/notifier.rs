use chrono::prelude::*;
use lambda_runtime::Error;
use rand::{Rng, SeedableRng, rngs::SmallRng, seq::SliceRandom};
use std::env::var;
use crate::matrix::Matrix;
use crate::notify::{create_notifications_from_calendar, get_notifications_within_time_window};
use crate::parser::get_calendar_entries_from_file;
use crate::s3::{get_object_as_string, save_string_as_object};
use crate::todo::get_todo_entries_from_aws;

pub async fn run_notifier() -> Result<(), Error> {
    let now = Utc::now().timestamp();

    let previous_now = get_object_as_string(var("S3_MAIN_BUCKET")?, "notification-last-run-timestamp.txt".to_string()).await
        .map_or(now - 3600, |s| {
            s.parse::<i64>().unwrap_or(now - 3600)
        });

        
    let notifications = create_notifications_from_calendar(&get_calendar_entries_from_file().await?);
    let notifications_within_time_window = get_notifications_within_time_window(&notifications, Utc::now().timestamp(), previous_now);
    
    let mut messages: Vec<String> = notifications_within_time_window.iter().map(|notification| {
        notification.msg.clone()
    }).collect();

    {
        let mut rng = SmallRng::from_entropy();
        let now = Utc::now();
        if now.hour() > 8 && now.hour() < 23 && rng.gen::<f64>() < 1.0/60.0 {
            let mut todo_entries = get_todo_entries_from_aws().await?;
            todo_entries.shuffle(&mut rng);
            todo_entries.first().map(|entry| messages.push(entry.clone()));
        }
    }
    
    if messages.len() > 0 {
        println!("{:?}", messages);
        Matrix { server: var("MATRIX_SERVER")? }.authenticate_and_send_messages_to_room(
            &var("MATRIX_USER")?,
            &var("MATRIX_PW")?,
            &var("MATRIX_REMINDER_ROOM")?,
            messages
        ).await;
    }

    save_string_as_object(now.to_string(), var("S3_MAIN_BUCKET")?, "notification-last-run-timestamp.txt".to_string()).await?;

    Ok(())
}