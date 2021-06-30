use lazy_static::lazy_static;
use std::env::var;
use regex::Regex;
use lambda_runtime::Error;
use crate::s3::get_object_as_string;

use cal_rem_shared::{Entry, Month, HourMinute};

fn year_regex(unparsed_entry: &str) -> Option<u32> {
    lazy_static! {
        static ref YEAR: Regex = Regex::new(r"(?x)
        ^
        (?P<year>\d{4})
        \s*
        $
").unwrap();
    }

    let valid_year = match YEAR.captures(unparsed_entry) {
        Some(entry) => entry,
        None => return None
    };

    Some(valid_year["year"].parse().unwrap())
}

fn month_regex(unparsed_entry: &str) -> Option<Month> {
    lazy_static! {
        static ref MONTH: Regex = Regex::new(r"(?x)
        ^
        (?P<month>Januar|Februar|Mars|April|Mai|Juni|Juli|August|September|Oktober|November|Desember)
        \s*
        $
").unwrap();
    }

    let valid_month = match MONTH.captures(unparsed_entry) {
        Some(entry) => entry,
        None => return None
    };

    match &valid_month["month"] {
        "Januar" => Some(Month::January),
        "Februar" => Some(Month::February),
        "Mars" => Some(Month::March),
        "April" => Some(Month::April),
        "Mai" => Some(Month::May),
        "Juni" => Some(Month::June),
        "Juli" => Some(Month::July),
        "August" => Some(Month::August),
        "September" => Some(Month::September),
        "Oktober" => Some(Month::October),
        "November" => Some(Month::November),
        "Desember" => Some(Month::December),
        _ => None
    }
}

fn event_entry_regex(unparsed_entry: &str, year: u32, month: Month) -> Option<Entry> {
    lazy_static! {
        static ref DATE: Regex = Regex::new(r"(?x)
        ^
        (
            (?P<date>\d+) | 
            (?P<maybe_date>\d+)\? | 
            (?P<start_date>\d+)-(?P<end_date>\d+) | 
            (?P<maybe_start_date>\d+)-(?P<maybe_end_date>\d+)\? | 
            \?
        )
        \.

        (?P<description>[^@\[]*) # stop when we reach @ or [

        (
            @
            (?P<location>[^\[]*) # stop when we reach [
        )?

        (
            \[
            (
                (?P<hour>\d{1,2})\.(?P<minute>\d{2}) |
                (?P<start_hour>\d{1,2})\.(?P<start_minute>\d{2})-(?P<end_hour>\d{1,2})\.(?P<end_minute>\d{2})
            )
            \]
        )?
").unwrap();
    }

    let valid_entry = match DATE.captures(unparsed_entry) {
        Some(entry) => entry,
        None => return None
    };

    let mut entry = Entry {
        year: year,
        month: month,
        start_date: None,
        end_date: None,
        start_time: None,
        end_time: None,
        description: "".to_string(),
        location: None
    };

    if valid_entry.name("date") != None {
        entry.start_date = Some(valid_entry["date"].parse().unwrap());
    } else if valid_entry.name("maybe_date") != None {
        entry.start_date = Some(valid_entry["maybe_date"].parse().unwrap());
    } else if valid_entry.name("start_date") != None {
        entry.start_date = Some(valid_entry["start_date"].parse().unwrap());
        entry.end_date = Some(valid_entry["end_date"].parse().unwrap());
    } else if valid_entry.name("maybe_start_date") != None {
        entry.start_date = Some(valid_entry["maybe_start_date"].parse().unwrap());
        entry.end_date = Some(valid_entry["maybe_end_date"].parse().unwrap());
    }

    entry.description = valid_entry["description"].trim().to_string();
    valid_entry.name("location").map(|loc| { entry.location = Some(loc.as_str().trim().to_string()) });

    // usage of map with unused return.
    valid_entry.name("hour").map(|hour| {
        entry.start_time = Some(HourMinute { 
            hour: hour.as_str().parse().unwrap(), 
            minute: valid_entry["minute"].parse().unwrap()
        });
    });

    valid_entry.name("start_hour").map(|hour| {
        entry.start_time = Some(HourMinute {
            hour: hour.as_str().parse().unwrap(),
            minute: valid_entry["start_minute"].parse().unwrap()
        });

        entry.end_time = Some(HourMinute {
            hour: valid_entry["end_hour"].parse().unwrap(),
            minute: valid_entry["end_minute"].parse().unwrap()
        });
    });

    return Some(entry)
}

pub async fn get_calendar_entries_from_file() -> Result<Vec<Entry>, Error> {
    let str = &get_object_as_string(var("S3_MAIN_BUCKET")?, "calendar.txt".to_string()).await?;

    let mut year: Option<u32> = None;
    let mut month: Option<Month> = None;

    Ok(str.split("\n").filter_map(|line| {
        year_regex(line).map(|y| year = Some(y));
        month_regex(line).map(|m| month = Some(m));
        if year.is_some() && month.is_some() {
            event_entry_regex(line, year.clone().unwrap(), month.clone().unwrap())
        } else {
            None
        }
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_parsing_test() {
        let event = event_entry_regex("10. Event with start time, end time, and location @ Place A [18.30-20.00]", 2020, Month::May).unwrap();
        assert_eq!(10, event.start_date.unwrap());
        assert!(event.end_date.is_none());
        assert_eq!("Event with start time, end time, and location", event.description);
        assert_eq!("Place A", event.location.unwrap());
        assert_eq!(HourMinute { hour: 18, minute: 30 }, event.start_time.unwrap());
        assert_eq!(HourMinute { hour: 20, minute: 00 }, event.end_time.unwrap());
        
        let event = event_entry_regex("11. Event with start time and location @ Place B [19.30]", 2020, Month::May).unwrap();
        assert_eq!(11, event.start_date.unwrap());
        assert!(event.end_date.is_none());
        assert_eq!("Event with start time and location", event.description);
        assert_eq!("Place B", event.location.unwrap());
        assert_eq!(HourMinute { hour: 19, minute: 30 }, event.start_time.unwrap());
        assert!(event.end_time.is_none());
        
        let event = event_entry_regex("17. Event with start time and no location [10.00]", 2020, Month::May).unwrap();
        assert_eq!(17, event.start_date.unwrap());
        assert!(event.end_date.is_none());
        assert_eq!("Event with start time and no location", event.description);
        assert!(event.location.is_none());
        assert_eq!(HourMinute { hour: 10, minute: 0 }, event.start_time.unwrap());
        assert!(event.end_time.is_none());
        
        let event = event_entry_regex("24?. Event with uncertain date", 2020, Month::May).unwrap();
        assert_eq!(24, event.start_date.unwrap());
        assert!(event.end_date.is_none());
        assert_eq!("Event with uncertain date", event.description);
        assert!(event.location.is_none());
        assert!(event.start_time.is_none());
        assert!(event.end_time.is_none());
        
        let event = event_entry_regex("5-11. Multiple day event with start time and location @ Place C [11.00]", 2020, Month::July).unwrap();
        assert_eq!(5, event.start_date.unwrap());
        assert_eq!(11, event.end_date.unwrap());
        assert_eq!("Multiple day event with start time and location", event.description);
        assert_eq!("Place C", event.location.unwrap());
        assert_eq!(HourMinute { hour: 11, minute: 0 }, event.start_time.unwrap());
        assert!(event.end_time.is_none());
        
        let event = event_entry_regex("?. Event with unknown date (while stile belonging to a month)", 2020, Month::October).unwrap();
        assert!(event.start_date.is_none());
        assert!(event.end_date.is_none());
        assert_eq!("Event with unknown date (while stile belonging to a month)", event.description);
        assert!(event.location.is_none());
        assert!(event.start_time.is_none());
        assert!(event.end_time.is_none());
        
        let event = event_entry_regex("Some text that is not interpreted as an event.", 2020, Month::May);
        assert!(event.is_none());
        
        // A year marker is not an event
        let event = event_entry_regex("2021", 2020, Month::May);
        assert!(event.is_none());
        
        // A month marker is not an event
        let event = event_entry_regex("Mai", 2020, Month::April);
        assert!(event.is_none());
    }
}