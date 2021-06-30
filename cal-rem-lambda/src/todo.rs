use lambda_runtime::Error;
use std::env::var;
use cal_rem_shared::Todo;
use crate::s3::get_object_as_string;

pub async fn get_todo_entries() -> Result<String, Error> {
    let todo_entries: Vec<Todo> = get_todo_entries_from_aws().await?.iter()
                .map(|todo| Todo { description: todo.clone(), done: true }).collect();

    Ok(serde_json::to_string(&todo_entries)?)
}

pub async fn get_todo_entries_from_aws() -> Result<Vec<String>, Error> {
    Ok(get_todo_entries_from_text(&get_object_as_string(var("S3_MAIN_BUCKET")?, "todo.txt".to_string()).await?))
}

fn get_todo_entries_from_text(text: &str) -> Vec<String> {
    text.split("\n")
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
        let todos = get_todo_entries_from_text("Do A\n  Do B\n\n   \n Do C  \n\n--- Ferdig ---\nDo D\n");
        assert_eq!(todos[0], "Do A");
        assert_eq!(todos[1], "Do B");
        assert_eq!(todos[2], "Do C");
    }
}