use serde::{Deserialize, Serialize};
use reqwest::{
    Client,
    header::{
        AUTHORIZATION,
        HeaderMap
    }
};

pub struct Matrix {
    pub server: String
}

#[derive(Deserialize, Debug)]
pub struct ClientVersionResponse {
    versions: Vec<String>,
    unstable_features: std::collections::HashMap<String, bool>
}

#[derive(Deserialize, Debug)]
pub struct LoginResponse {
    user_id: String,
    pub access_token: String,
    home_server: String,
    device_id: String,
    well_known: std::collections::HashMap<String, std::collections::HashMap<String, String>>
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum MatrixResponse<T> {
    Ok(T),
    Err(ErrorResponse)
}

#[derive(Deserialize, Debug)]
pub struct EventResponse {
    event_id: String
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    errcode: String,
    error: String
}

#[derive(Serialize)]
struct MessageRequest {
    body: String,
    msgtype: String
}

impl Matrix {
    async fn login(&self, user: &str, password: &str) -> Result<MatrixResponse<LoginResponse>, reqwest::Error> {
        let mut map = std::collections::HashMap::new();
        map.insert("type", "m.login.password");
        map.insert("user", user);
        map.insert("password", password);

        Client::new().post(format!("https://{}/_matrix/client/r0/login", self.server))
            .json(&map)
            .send()
            .await?
            .json::<MatrixResponse<LoginResponse>>()
            .await
    }

    async fn send_msg_to_room(&self, message: &str, token: &str, room_id: &str) -> Result<MatrixResponse<EventResponse>, reqwest::Error> {
        Client::new().put(format!("https://{}/_matrix/client/r0/rooms/{}/send/m.room.message/{}", self.server, room_id, rand::random::<u32>()))
            .headers(authorization_header_map(token))
            .json(&MessageRequest { body: message.to_string(), msgtype: "m.text".to_string() })
            .send()
            .await?
            .json::<MatrixResponse<EventResponse>>()
            .await
    }

    pub async fn authenticate_and_send_messages_to_room(&self, user: &str, password: &str, room_id: &str, messages: Vec<String>) {
        let response = self.login(user, password).await;
        
        match response {
            Ok(login_response) => {
                match login_response {
                    MatrixResponse::Ok(login_response) => {
                        let token = login_response.access_token;
                        for message in messages {
                            match self.send_msg_to_room(message.as_str(), &token, room_id).await {
                                Ok(event_response) => {
                                    match event_response {
                                        MatrixResponse::Ok(_) => {},
                                        MatrixResponse::Err(error_response) => {
                                            error_log("send message", &error_response);
                                        }
                                    }
                                },
                                Err(err) => {
                                    error_log("send message", &err);
                                }
                            }
                        }
                    },
                    MatrixResponse::Err(error_response) => {
                        error_log("login", &error_response);
                    }

                };
            },
            Err(err) => {
                error_log("login", &err);
            }
        };
    }
}

fn authorization_header_map(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
    headers
}

fn error_log<T: std::fmt::Debug>(msg: &str, error: &T) {
    println!("{}: {:?}", msg, error);
}