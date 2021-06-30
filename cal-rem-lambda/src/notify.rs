use chrono::prelude::*;
use chrono::Duration;
use chrono_tz::Tz;

use cal_rem_wasm_com::Entry;

#[derive(Debug, Clone)]
pub struct Notification {
    time: DateTime<Utc>,
    pub msg: String
}

pub fn create_notifications_from_calendar(entries: &Vec<Entry>) -> Vec<Notification> {
    let mut notifications: Vec<Notification> = entries.iter().map(|entry| {
        let event_time = entry.get_oslo_date_time();
        
        let msg = entry.create_message();
        
        let mut v = Vec::new();
        v.push(Notification { time: utc_notification_time_short_notice(event_time), msg: format!("Om 20 min: {}", msg.clone()) });
        v.push(Notification { time: utc_notification_time_medium_notice(event_time), msg: format!("husk: {}", msg.clone()) });
        v.push(Notification { time: utc_notification_time_24h_notice(event_time), msg: format!("I morgen: {}", msg.clone()) });
                
        v
    }).flatten().collect();
    
    notifications.sort_by(|a, b| {
        a.time.cmp(&b.time)
    });
    
    notifications
}

pub fn get_notifications_within_time_window(notifications: &Vec<Notification>, current_time: i64, previous_time: i64) -> Vec<Notification> {
    notifications.iter().filter_map(|notification| {
        if notification.time.timestamp() > previous_time && notification.time.timestamp() <= current_time {
            Some(notification.clone())
        } else {
            None
        }
    }).collect()
}

fn utc_notification_time_short_notice(event_time: DateTime<Tz>) -> DateTime<Utc> {
    event_time.with_timezone(&Utc).checked_sub_signed(Duration::minutes(20)).unwrap()
}

fn utc_notification_time_medium_notice(event_time: DateTime<Tz>) -> DateTime<Utc> {
    // if event time is between 01:00 and 10:59, subtract so the notification time is 11 PM local time.
    if event_time.hour() >= 1 && event_time.hour() <= 10 {
        event_time.with_timezone(&Utc).checked_sub_signed(
            Duration::hours(2 + event_time.hour() as i64 - 1) +
            Duration::minutes(event_time.minute() as i64)
        ).unwrap()
    } else {
        event_time.with_timezone(&Utc).checked_sub_signed(Duration::hours(2)).unwrap()
    }
}

fn utc_notification_time_24h_notice(event_time: DateTime<Tz>) -> DateTime<Utc> {
    event_time.with_timezone(&Utc).checked_sub_signed(Duration::hours(24)).unwrap()
}