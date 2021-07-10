#![allow(clippy::wildcard_imports)]

mod calendar;
mod todo;

use seed::{prelude::*, *};
use cal_rem_shared::{Entry, Todo};
use cal_rem_shared::{Command, RequestBody};
use crate::calendar::{future_calendar_nodes_from_entries, todays_date_description};
use crate::todo::sliding_todo;

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.send_msg(Msg::CalendarEntryRequest);
    orders.send_msg(Msg::TodoEntryRequest);

    Model { calendar_entries: vec![], todo_entries: vec![] }
}

struct Model {
    calendar_entries: Vec<Entry>,
    todo_entries: Vec<Todo>,
}

pub enum Msg {
    CalendarEntryRequest,
    CalendarEntryResponse(Vec<Entry>),
    TodoEntryRequest,
    TodoEntryResponse(Vec<Todo>),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::CalendarEntryRequest => {
            orders.skip().perform_cmd(async {
                //let req_body = RequestBody { command: Command::GetCalendarEvents, parameters: "".to_string() };
                //let req = Request::new("https://97g5b34p9e.execute-api.eu-north-1.amazonaws.com/default/calendar-reminder-api-functions").method(Method::Post).json(&req_body).unwrap();
                
                let req = Request::new("https://97g5b34p9e.execute-api.eu-north-1.amazonaws.com/default/calendar-reminder-api-functions/get-all-calendar-entries").method(Method::Get);
                let response = req.fetch().await.expect("HTTP request failed");
                let response = response.check_status().expect("status failed").json().await.expect("deserialization failed");
                Msg::CalendarEntryResponse(response)
            });
        },
        Msg::CalendarEntryResponse(response) => {
            model.calendar_entries = response;
        },
        Msg::TodoEntryRequest => {
            orders.skip().perform_cmd(async {
                //let req_body = RequestBody { command: Command::GetTodoEntries, parameters: "".to_string() };
                //let req = Request::new("https://97g5b34p9e.execute-api.eu-north-1.amazonaws.com/default/calendar-reminder-api-functions").method(Method::Post).json(&req_body).unwrap();
                
                let req = Request::new("https://97g5b34p9e.execute-api.eu-north-1.amazonaws.com/default/calendar-reminder-api-functions/get-all-todo-entries").method(Method::Get);
                let response = req.fetch().await.expect("HTTP request failed");
                let response = response.check_status().expect("status failed").json().await.expect("deserialization failed");
                Msg::TodoEntryResponse(response)
            });
        },
        Msg::TodoEntryResponse(response) => {
            model.todo_entries = response;
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    
    div![
        id!["mainpage"],
        sliding_todo(&model.todo_entries),
        div![
            style!{St::Margin => px(16)},
            span![ todays_date_description() ],
            future_calendar_nodes_from_entries(&model.calendar_entries),
        ]
    ]

    /*
    button![
        style!{St::Padding => px(20) + " " + &px(15)},
        "FETCH",
        ev(Ev::Click, |_| Msg::CalendarEntryRequest),
    ],
    */

}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
