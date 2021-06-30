use seed::{prelude::*, *};
use cal_rem_shared::Todo;
use crate::Msg;

pub fn sliding_todo(todo_list: &Vec<Todo>) -> Node<Msg> {
    let todo_string = todo_list.iter()
        .filter(|todo| todo.done)
        .map(|todo| format!("{} --------- ", todo.description))
        .collect::<Vec<String>>().join("");

    marquee(&todo_string)
}

fn marquee(content: &str) -> Node<Msg> {
    let content = format!("{}{}{}{}", content, content, content, content);
    div![
        style!{St::Overflow => "hidden"},
        div![
            style!{St::Display => "inline-block"},
            style!{St::MarginBottom => px(2)},
            div![
                style!{St::Display => "inline-block"},
                style!{St::WhiteSpace => "nowrap"},
                style!{St::Animation => "todo-marquee 60s linear infinite"},
                style!{St::Color => "#fff"},
                style!{St::Background => "rgb(110, 131, 157)"},
                style!{St::Padding => px(3)},
                
                div![
                    &content,
                    style!{St::Display => "inline-block"},
                    style!{St::FontSize => px(12)},
                ],
                div![
                    &content,
                    style!{St::Display => "inline-block"},
                    style!{St::FontSize => px(12)},
                ],
            ]
        ]
    ]        
}