use crossbeam_channel::{Receiver, Sender};
use dotenv::dotenv;
use http::header::HeaderValue;
use reqwest::{
    blocking::{Client, RequestBuilder, Response},
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde_json::{json, Map, Value};
use std::{
    collections::HashMap,
    env,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
};
use tungstenite::{client::IntoClientRequest, connect, stream::NoDelay, Message};


#[derive(Clone)]
pub struct Session {
    pub host:          String,
    pub client:        Client,
    pub token:         Option<String>,
    pub content_type:  String,
    pub from_cli:      Receiver<String>,
    pub to_cli:        Sender<String>,
    pub stop_from_cli: Receiver<bool>,
}

impl Session {
    pub fn new(fr_c: Receiver<String>, to_c: Sender<String>, stop: Receiver<bool>) -> Self {
        dotenv().ok();
        let token_: Option<String> = match env::var("TOKEN") {
            Ok(value) => Some(value),
            _ => None,
        };

        Self {
            host: env::var("HOST").expect("Host doesn't exists!"),
            client:       Client::new(),
            token:        token_,
            content_type: "application/json".to_string(),
            from_cli:     fr_c,
            to_cli:       to_c,
            stop_from_cli:stop,
        }
    }

    fn request(
        &self,
        url: String,
        typ: &str,
        token: Option<String>,
        form: Option<&Value>,
    ) -> Value {
        let mut response: RequestBuilder;
        match typ {
            "get" => {
                response = self
                    .client
                    .get(url)
                    .header(CONTENT_TYPE, self.content_type.clone());
            }
            "post" => {
                response = self
                    .client
                    .post(url)
                    .header(CONTENT_TYPE, self.content_type.clone());
            }
            "put" => {
                response = self
                    .client
                    .put(url)
                    .header(CONTENT_TYPE, self.content_type.clone());
            }
            _ => {
                response = self
                    .client
                    .delete(url)
                    .header(CONTENT_TYPE, self.content_type.clone());
            }
        }

        response = match token {
            Some(value) => response.header(AUTHORIZATION, format!("Bearer {value}")),
            None => response,
        };

        response = match form {
            Some(value) => response.json(value),
            None => response,
        };

        let response: Response = response.send().expect("Request isn't sent properly!");
        response.json().unwrap()
    }

    // This alters an ENV variable inside .env file and resets
    // runtime ENVs
    fn set_dotenv_var(&mut self, key: &str, value: String) {
        let file = File::open(".env").expect("Failed to open .env file");
        let reader = BufReader::new(file);
        let mut found = false;

        let lines: Vec<String> = reader
            .lines()
            .filter_map(Result::ok)
            .map(|line| {
                if line.trim_start().starts_with(&format!("{key}=")) {
                    found = true;
                    format!("{key}={value}")
                } else {
                    line
                }
            })
            .collect();

        let final_lines = if !found {
            let mut new_lines = lines.clone();
            new_lines.push(format!("{key}={value}"));
            new_lines
        } else {
            lines
        };

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(".env")
            .expect("Failed to open .env file for writing");

        for line in final_lines {
            writeln!(file, "{}", line).expect("Failed to write to .env file");
        }
        dotenv().ok();
    }

    // Let this function take care of all response types
    fn resp_val(&self, data: &Value, key: &str) -> Value {
        let map: Map<String, Value> = data.as_object().unwrap().clone();

        let value: Value = map.get(key).expect("No such key!").clone();
        value
    }

    fn resp_str(&self, data: &Value, key: &str) -> String {
        let value = self.resp_val(data, key);
        let value = value.as_str().map(|s| s.to_string()).unwrap();
        value
    }

    fn resp_arr(&self, data: &Value, key: &str) -> Vec<String> {
        let value = self.resp_val(data, key);
        value.as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str())
            .map(|v| v.to_string())
            .collect()
    }

    fn check_stat(&self, response: &Value) -> bool {
        let status = self.resp_str(response, "status");

        match status.as_str() {
            "ok" => true,
            _    => false,
        }
    }

    pub fn signup(
        &self,
        show_name: &str,
        password: &str,
        related_question: &str,
        related_answer: &str,
    ) -> HashMap<&str, String> {
        let url = format!("{}/auth/signup", self.host);
        let form = json!({
            "show_name":         show_name.to_string(),
            "password":          password.to_string(),
            "related_question":  related_question.to_string(),
            "related_answer":    related_answer.to_string(),
        });

        let response = self.request(url, "post", None, Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_str(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn login(&mut self, show_name: &str, password: &str) -> HashMap<&str, String> {
        let url = format!("{}/auth/login", self.host);
        let form = json!({
            "show_name":         show_name.to_string(),
            "password":          password.to_string(),
        });

        let response = self.request(url, "post", None, Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg   = self.resp_str(&response, "message");
            let data  = self.resp_val(&response, "data");
            let token = self.resp_str(&data, "token");

            // Set token into .env file
            self.set_dotenv_var("TOKEN", token.clone());
            self.token = Some(token.clone());
            map.insert("ok", msg.to_string());
            map.insert("token", token.clone().to_string());
            map
        }
    }

    pub fn user_exists(&self, show_name: &str) -> HashMap<&str, String> {
        let url = format!("{}/auth/uexist", self.host);
        let form = json!({
            "show_name": show_name.to_string(),
        });

        let response = self.request(url, "post", None, Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_str(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn logout(&self) -> HashMap<&str, String> {
        let url = format!("{}/users/logout", self.host);
        let token = self.token.clone().unwrap();

        let response = self.request(url, "get", Some(token), None);

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_str(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    // In case for checking token is still valid
    pub fn ping(&self) -> HashMap<&str, String> {
        let url = format!("{}/users/ping", self.host);
        let token = self.token.clone().unwrap();

        let response = self.request(url, "get", Some(token), None);

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_str(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn user_rename(&self, show_name: &str) -> HashMap<&str, String> {
        let url = format!("{}/users/rename", self.host);
        let token = self.token.clone().unwrap();
        let form = json!({
            "show_name":         show_name.to_string(),
        });

        let response = self.request(url, "put", Some(token), Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_str(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn user_repass(&self, current_pass: &str, new_pass: &str) -> HashMap<&str, String> {
        let url = format!("{}/users/repass", self.host);
        let token = self.token.clone().unwrap();
        let form = json!({
            "current_pass":         current_pass.to_string(),
            "new_pass":             new_pass.to_string(),
        });

        let response = self.request(url, "put", Some(token), Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_str(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn room_build(&self, name: &str, is_public: &str) -> HashMap<&str, String> {
        let url = format!("{}/rooms/build", self.host);
        let token = self.token.clone().unwrap();
        let form = json!({
            "name":                  name.to_string(),
            "is_public":             is_public.to_string(),
        });

        let response = self.request(url, "post", Some(token), Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_str(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn room_close(&self, hash: &str) -> HashMap<&str, String> {
        let url = format!("{}/rooms/close", self.host);
        let token = self.token.clone().unwrap();
        let form = json!({
            "hash":                  hash.to_string(),
        });

        let response = self.request(url, "del", Some(token), Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_str(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn room_rename(&self, hash: &str, new_name: &str) -> HashMap<&str, String> {
        let url = format!("{}/rooms/rename", self.host);
        let token = self.token.clone().unwrap();
        let form = json!({
            "hash":                  hash.to_string(),
            "new_name":              new_name.to_string(),
        });

        let response = self.request(url, "put", Some(token), Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_str(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn room_publist(&self) -> HashMap<&str, Vec<String>> {
        let url = format!("{}/rooms/publist", self.host);
        let token = self.token.clone().unwrap();

        let response = self.request(url, "get", Some(token.clone()), None);

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_str(&response, "error");
            map.insert("error", vec![format!("{err}")]);
            map
        } else {
            let data   = self.resp_val(&response, "data");
            let names  = self.resp_arr(&data, "names");
            let hashes = self.resp_arr(&data, "hashes");
            //let names = self.res(&names);
            //let hashes = self.val2vec(&hashes);
            map.insert("ok", vec!["".to_string()]);
            map.insert("names", names);
            map.insert("hashes", hashes);
            map
        }
    }

    pub fn chat_connect(&self, room_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/chat/manage", self.host).replace("http", "ws");
        let mut request = url.into_client_request()?;
        let token = format!("Bearer {}", self.token.clone().unwrap());

        // headers
        request
            .headers_mut()
            .insert("Authorization", HeaderValue::from_str(&token).unwrap());
        request
            .headers_mut()
            .insert("X-Room-Hash", HeaderValue::from_str(room_hash).unwrap());

        let (mut socket, _) = connect(request)?;

        // This block makes stream happen imidiatly and not adding up to a buffer
        match socket.get_mut() {
            tungstenite::stream::MaybeTlsStream::Plain(stream) => stream.set_nonblocking(true),
            _ => unimplemented!(),
        }?;
        ///////////////////////////////////////////////////////////////////////

        loop {
            if let Ok(flag) = self.stop_from_cli.try_recv() {
                if flag {
                    socket.close(None)?;
                    break
                }
            }

            if let Ok(msg) = self.from_cli.try_recv() {
                if let Err(e) = socket.write(Message::Text(msg.into())) {
                    eprintln!("Write error: {}", e);
                    break
                }
            }

            if let Ok(msg) = socket.read() {
                match msg {
                    Message::Text(text) => {
                        println!("{}", text.clone().to_string());
                        self.to_cli.send(text.to_string())?;
                    }
                    Message::Close(_) => {
                        println!("WebSocket connection closed");
                        break
                    }
                    _ => {}
                }
            }

            socket.flush()?;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }
}
