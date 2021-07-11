use lambda_runtime::Error;
use std::env::var;
use crate::{Header, Response, get_default_headers, s3::{BrowserCachedData, get_object_as_string_if_etags_differ}};
use cal_rem_shared::Todo;

pub async fn get_todo_entries(etag: Option<String>) -> Result<Response, Error> {
    let cached_data = get_object_as_string_if_etags_differ(var("S3_MAIN_BUCKET")?, "todo.txt".to_string(), etag).await?;

    let mut headers = get_default_headers();

    Ok(match cached_data {
        BrowserCachedData::InCache => {
            Response { status_code: 304, headers, body: "".to_string()}
        },
        BrowserCachedData::NotInCache { data, etag } => {
            headers.insert(Header::ETag, etag);
            let todos: Vec<Todo> = parse_todo_file(&data).iter().map(|todo| Todo { description: todo.clone(), done: true }).collect();
            Response { status_code: 200, headers, body: serde_json::to_string(&todos)?}
        }
    })
}

pub fn parse_todo_file(file: &str) -> Vec<String> {
    file.split("\n")
    .filter_map(|line| {
        let description = line.trim();
        
        if description.len() > 0 {
            Some(description.to_string())
        } else {
            None
        }
    })
    .take_while(|line| line != "--- Ferdig ---")
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_parsing_test() {
        let todos = parse_todo_file("Do A\n  Do B\n\n   \n Do C  \n\n--- Ferdig ---\nDo D\n");
        assert_eq!(todos[0], "Do A");
        assert_eq!(todos[1], "Do B");
        assert_eq!(todos[2], "Do C");
    }
}