use serde_json::{
    Map,
    Value,
    json
};
use reqwest::{
    blocking::{
        Client,
        Response,
        RequestBuilder,
    },
    header::{
        AUTHORIZATION,
        CONTENT_TYPE,
    }
};
use std::{
    env,
    fs::{
        File,
        OpenOptions,
    },
    io::{
        Write,
        BufRead,
        BufReader,
    },
    collections::HashMap,
};
use dotenv::dotenv;


pub struct Session {
    pub host:          String,
    pub client:        Client,
    pub token:         Option<String>,
    pub content_type:  String,
}

impl Session {
    pub fn new() -> Self{
        dotenv().ok();
        let token_: Option<String> = match env::var("TOKEN") {
            Ok(value) => Some(value),
            _         => None
        };

        Self {
            host:           env::var("HOST").expect("Host doesn't exists!"),
            client:         Client::new(),
            token:          token_,
            content_type:   "application/json".to_string(),
        }
    }

    fn request(&self, url: String, typ: &str, token: Option<String>, form: Option<&Value>) -> Value {
        let mut response: RequestBuilder;
        match typ {
            "get" => {
                response = self.client
                    .get(url)
                    .header(CONTENT_TYPE, self.content_type.clone());
            },
            "post" => {
                response = self.client
                    .post(url)
                    .header(CONTENT_TYPE, self.content_type.clone());
            },
            "put" => {
                response = self.client
                    .put(url)
                    .header(CONTENT_TYPE, self.content_type.clone());
            },
            _     => {
                response = self.client
                    .delete(url)
                    .header(CONTENT_TYPE, self.content_type.clone());
            },
        }

        response = match token {
            Some(value) => response.header(AUTHORIZATION, format!("Bearer {value}")),
            None        => response,
        };

        response = match form {
            Some(value) => response.json(value),
            None        => response,
        };

        let response: Response = response.send().expect("Request isn't sent properly!");
        response.json().unwrap()
    }

    // This alters an ENV variable inside .env file and resets
    // runtime ENVs
    fn set_dotenv_var(&mut self, key: String, value: String) {
        if key == "TOKEN".to_string() {
            self.token = Some(value.clone());
        }

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
    fn resp_val(&self, data: &Value, key: &str) -> Value{
        let map: Map<String, Value> = data
            .as_object()
            .unwrap()
            .clone();

        let value: Value = map.get(key).expect("No such key!").clone();
        value
    }

    fn val2vec(&self, data: &Value) -> Vec<String> {
        data.as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str())
            .map(|v| v.to_string())
            .collect()
    }

    fn check_stat(&self, response: &Value) -> bool {
        let status = self.resp_val(&response, "status");

        match status.as_str() {
            Some("ok") => true,
            _          => false,
        }
    }

    pub fn signup(&self, show_name: String, password: String,
        related_question: String, related_answer: String) -> HashMap<&str, String>{
        let url = format!("{}/auth/signup", self.host);
        let form = json!({
            "show_name":         show_name,
            "password":          password,
            "related_question":  related_question,
            "related_answer":    related_answer,
        });

        let response = self.request(url, "post", None, Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_val(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_val(&response, "message");
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
            let err = self.resp_val(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_val(&response, "message");
            let data  = self.resp_val(&response, "data");
            let token = self.resp_val(&data, "token");

            // Set token into .env file
            self.set_dotenv_var("TOKEN".to_string(), token.to_string());
            map.insert("ok", msg.to_string());
            map.insert("token", token.to_string());
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
            let err = self.resp_val(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_val(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn logout(&self) -> HashMap<&str, String> {
        let url   = format!("{}/users/logout", self.host);
        let token = self.token.clone().unwrap().to_string();

        let response = self.request(url, "get", Some(token), None);

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_val(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_val(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    // In case for checking token is still valid
    pub fn ping(&self) -> HashMap<&str, String> {
        let url = format!("{}/users/ping", self.host);
        let token = self.token.clone().unwrap().to_string();

        let response = self.request(url, "get", Some(token), None);

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_val(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_val(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn user_rename(&self, show_name: &str) -> HashMap<&str, String> {
        let url = format!("{}/users/rename", self.host);
        let token = self.token.clone().unwrap().to_string();
        let form = json!({
            "show_name":         show_name.to_string(),
        });

        let response = self.request(url, "put", Some(token), Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_val(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_val(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn user_repass(&self, current_pass: &str, new_pass: &str) -> HashMap<&str, String> {
        let url = format!("{}/users/repass", self.host);
        let token = self.token.clone().unwrap().to_string();
        let form = json!({
            "current_pass":         current_pass.to_string(),
            "new_pass":             new_pass.to_string(),
        });

        let response = self.request(url, "put", Some(token), Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_val(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_val(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn room_build(&self, name: &str, is_public: &str) -> HashMap<&str, String> {
        let url = format!("{}/rooms/build", self.host);
        let token = self.token.clone().unwrap().to_string();
        let form = json!({
            "name":                  name.to_string(),
            "is_public":             is_public.to_string(),
        });

        let response = self.request(url, "post", Some(token), Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_val(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_val(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn room_close(&self, hash: &str) -> HashMap<&str, String> {
        let url = format!("{}/rooms/close", self.host);
        let token = self.token.clone().unwrap().to_string();
        let form = json!({
            "hash":                  hash.to_string(),
        });

        let response = self.request(url, "del", Some(token), Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_val(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_val(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn room_rename(&self, hash: &str, new_name: &str) -> HashMap<&str, String> {
        let url = format!("{}/rooms/rename", self.host);
        let token = self.token.clone().unwrap().to_string();
        let form = json!({
            "hash":                  hash.to_string(),
            "new_name":              new_name.to_string(),
        });

        let response = self.request(url, "put", Some(token), Some(&form));

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_val(&response, "error");
            map.insert("error", err.to_string());
            map
        } else {
            let msg = self.resp_val(&response, "message");
            map.insert("ok", msg.to_string());
            map
        }
    }

    pub fn room_publist(&self) -> HashMap<&str, Vec<String>> {
        let url = format!("{}/rooms/publist", self.host);
        let token = self.token.clone().unwrap().to_string();

        let response = self.request(url, "get", Some(token), None);

        let mut map = HashMap::new();
        if !self.check_stat(&response) {
            let err = self.resp_val(&response, "error");
            map.insert("error", vec![format!("{err}")]);
            map
        } else {
            let data      = self.resp_val(&response, "data");
            let names     = self.resp_val(&data, "names");
            let hashes    = self.resp_val(&data, "hashes");
            let names     = self.val2vec(&names);
            let hashes    = self.val2vec(&hashes);
            map.insert("names", names);
            map.insert("hashes", hashes);
            map
        }
    }
}
