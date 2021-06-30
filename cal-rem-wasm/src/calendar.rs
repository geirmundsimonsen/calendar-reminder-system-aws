use seed::{prelude::*, *};
use chrono::{Datelike, Weekday};
use cal_rem_shared::{Entry, Month};
use crate::Msg;

pub fn future_calendar_nodes_from_entries(entries: &Vec<Entry>) -> Vec<Node<Msg>> {
    let now = (js_sys::Date::now() / 1000.0) as i64;
    let mut month: Option<Month> = None;
    let is_in_the_future = |entry: &&Entry| now - 3600 * 4 < entry.get_oslo_date_time().timestamp();
    
    entries.iter().filter(is_in_the_future).flat_map(|entry| {
        let mut els = vec![];
        month_header(&entry, &mut month).map(|month_header| els.push(month_header));
        els.push(calendar_entry(&entry));
        els
    }).collect()
}

fn calendar_entry(entry: &Entry) -> Node<Msg> { 
    div![
        day_date(entry),
        style!{St::Display => "flex"},
        style!{St::AlignItems => "baseline"},
        style!{St::MarginBottom => px(2)},
        div![
            style!{St::Display => "flex"},
            style!{St::FlexDirection => "column"},
            span![
                entry.description.clone(),
                style!{St::FontSize => px(24)},
            ],
            span![
                location(entry),
                style!{St::FontSize => px(12)},
            ],
            span![
                time(entry),
                style!{St::FontSize => px(12)},
                style!{St::MarginBottom => px(24)},
            ],
        ]
    ]
}

fn day_date(entry: &Entry) -> Vec<Node<Msg>> {
    if entry.start_date.is_some() && entry.end_date.is_none() {
        vec![
            span![
                short_day_name(&entry.get_oslo_date_time().weekday()),
                style!{St::Flex => "0 0 24px"},
                style!{St::FontSize => px(12)},
            ],
            span![
                date(&entry),
                style!{St::Flex => "0 0 36px"},
                style!{St::FontSize => px(24)},
                style!{St::PaddingRight => px(24)},
                style!{St::TextAlign => "right"},
            ],
        ]
    } else {
        vec![
            span![
                date(&entry),
                style!{St::Flex => "0 0 60px"},
                style!{St::FontSize => px(24)},
                style!{St::PaddingRight => px(24)},
                style!{St::TextAlign => "right"},
            ]
        ]
    }
}

fn month_header(entry: &Entry, current_month: &mut Option<Month>) -> Option<Node<Msg>> {
    if Some(entry.month) != *current_month {
        current_month.replace(entry.month);
        Some(h2![
            month_name(entry.month),
            style!{St::FontSize => px(32)}
        ])
    } else {
        None
    }
}

pub fn todays_date_description() -> String {
    let date = js_sys::Date::new(&JsValue::from_f64(js_sys::Date::now()));
    format!("{} {}. {}", short_day_name_from_js_day(date.get_utc_day()), date.get_utc_date(), month_name_from_js_month(date.get_utc_month()))
}

fn date(entry: &Entry) -> String {
    if entry.start_date.is_some() {
        if entry.end_date.is_some() {
            format!("{}-{}", entry.start_date.unwrap().to_string(), entry.end_date.unwrap().to_string())
        } else {
            format!("{}", entry.start_date.unwrap().to_string())
        }
    } else {
        "".to_string()
    }
}

fn time(entry: &Entry) -> String {
    if entry.start_time.is_some() {
        if entry.end_time.is_some() {
            format!("{:02}.{:02}-{:02}.{:02}", entry.start_time.unwrap().hour, entry.start_time.unwrap().minute, entry.end_time.unwrap().hour,  entry.end_time.unwrap().minute)
        } else {
            format!("{:02}.{:02}", entry.start_time.unwrap().hour, entry.start_time.unwrap().minute)
        }
    } else {
        "".to_string()
    }
}

fn location(entry: &Entry) -> String {
    if entry.location.is_some() {
        entry.location.clone().unwrap()
    } else {
        "".to_string()
    }
}

fn month_name(month: Month) -> &'static str {
    match month {
        Month::January => "Januar",
        Month::February => "Februar",
        Month::March => "Mars",
        Month::April => "April",
        Month::May => "Mai",
        Month::June => "Juni",
        Month::July => "Juli",
        Month::August => "August",
        Month::September => "September",
        Month::October => "Oktober",
        Month::November => "November",
        Month::December => "Desember",
    }
}

fn short_day_name(weekday: &Weekday) -> &str {
    match weekday {
        Weekday::Mon => "man",
        Weekday::Tue => "tir",
        Weekday::Wed => "ons",
        Weekday::Thu => "tor",
        Weekday::Fri => "fre",
        Weekday::Sat => "lør",
        Weekday::Sun => "søn",
    }
}

fn month_name_from_js_month(month: u32) -> &'static str {
    match month {
        0 => "Januar",
        1 => "Februar",
        2 => "Mars",
        3 => "April",
        4 => "Mai",
        5 => "Juni",
        6 => "Juli",
        7 => "August",
        8 => "September",
        9 => "Oktober",
        10 => "November",
        11 => "Desember",
        _ => "error",
    }
}

fn short_day_name_from_js_day(weekday: u32) -> &'static str {
    match weekday {
        1 => "man",
        2 => "tir",
        3 => "ons",
        4 => "tor",
        5 => "fre",
        6 => "lør",
        0 => "søn",
        _ => "error",
    }
}