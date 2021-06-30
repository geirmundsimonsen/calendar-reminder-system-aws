use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use chrono_tz::{Tz, Europe::Oslo};

#[derive(Serialize, Deserialize)]
pub enum Command {
    GET_TODO_ENTRIES,
    GET_CALENDAR_EVENTS,
}

#[derive(Serialize, Deserialize)]
pub struct RequestBody {
    pub command: Command,
    pub parameters: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct HourMinute {
    pub hour: u32,
    pub minute: u32
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Entry {
    pub description: String,
    pub location: Option<String>,
    pub year: u32,
    pub month: Month,
    pub start_date: Option<u32>,
    pub end_date: Option<u32>,
    pub start_time: Option<HourMinute>,
    pub end_time: Option<HourMinute>
}

impl Entry {
    pub fn create_message(&self) -> String {
        format!("{:?}{}{}: {}",
            self.month,
            self.start_date.map_or(" no date".to_string(), |date| format!(" {}.", &date.to_string())),
            self.start_time.map_or("".to_string(), |time| format!(", {:02}.{:02}", time.hour, time.minute)),
            self.description)
    }

    pub fn get_oslo_date_time(&self) -> DateTime<Tz> {
        Oslo.ymd(
            self.year as i32, 
            month_to_num(self.month), 
            self.start_date.unwrap_or(1)
        ).and_hms(
            self.start_time.map_or(8, |hm| hm.hour),
            self.start_time.map_or(0, |hm| hm.minute),
            0
        )
    }
}

fn month_to_num(month: Month) -> u32 {
    match month {
        Month::January => 1,
        Month::February => 2,
        Month::March => 3,
        Month::April => 4,
        Month::May => 5,
        Month::June => 6,
        Month::July => 7,
        Month::August => 8,
        Month::September => 9,
        Month::October => 10,
        Month::November => 11,
        Month::December => 12
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Todo {
    pub description: String,
    pub done: bool
}