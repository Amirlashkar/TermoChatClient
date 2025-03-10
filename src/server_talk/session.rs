use serde_json::{
    Value, json,
    to_string_pretty,
};
use reqwest::{
    blocking::Client,
    Error as ReqError,
    header::{
        AUTHORIZATION,
        CONTENT_TYPE,
    }
};
use std::{
    env,
    fs::OpenOptions,
    io::Write,
};
use dotenv::dotenv;


pub struct Session {
    host:          String,
    client:        Client,
    token:         Option<String>,
    content_type:  String,
}

impl Session {
    pub fn new() -> Self{
        dotenv().ok();
        let token_: Option<String> = match env::var("TOKEN") {
            Ok(value) => Some(value),
            _         => None
        };

        Self {
            host:           env::var("HOST").expect("Host exists"),
            client:         Client::new(),
            token:          token_,
            content_type:   "application/json".to_string(),
        }
    }

    pub fn set_dotenv_var(&mut self, key: String, value: String) {
        if key == "TOKEN".to_string() {
            self.token = Some(value.clone());
        }

        let mut file = OpenOptions::new()
            .create(false)
            .append(true)
            .open(".env")
            .expect("There is no such file");

        writeln!(file, "{}={}", key, value).expect("");
    }

    // In case for checking token is still valid
    pub fn ping(self) -> Result<Value, ReqError>{
        let url = format!("{}/users/ping", self.host);

        let response = self.client
            .get(url)
            .header(CONTENT_TYPE, self.content_type)
            .header(AUTHORIZATION, format!("Bearer {}", self.token.expect("Endpoint needs token")))
            .send()?;

        let response = response.json()?;
        Ok(response)
    }

    pub fn signup(self, show_name: String, password: String,
                        related_question: String, related_answer: String) -> Result<Value, ReqError> {
        let url = format!("{}/auth/signup", self.host);
        let form = json!({
            "show_name":         show_name,
            "password":          password,
            "related_question":  related_question,
            "related_answer":    related_answer,
        });

        let response = self.client
            .post(url)
            .header(CONTENT_TYPE, self.content_type)
            .json(&form)
            .send()?;

        let response: Value = response.json()?;
        Ok(response)
    }

     pub fn login(&mut self, show_name: String, password: String) -> Result<Value, ReqError> {
        let url = format!("{}/auth/login", self.host);
        let form = json!({
            "show_name":         show_name,
            "password":          password,
        });

        let response = self.client.clone()
            .post(url)
            .header(CONTENT_TYPE, self.content_type.clone())
            .json(&form)
            .send()?;

        let response: Value = response.json()?;
        // Set token into .env file
        let token = response.get("token").and_then(|v| v.as_str()).expect("").to_string();
        self.set_dotenv_var("TOKEN".to_string(), token.clone());
        println!("{}", token.clone());

        Ok(response)
    }

    pub fn logout(self) -> Result<Value, ReqError> {
        let url = format!("{}/users/logout", self.host);

        let response = self.client
            .get(url)
            .header(CONTENT_TYPE, self.content_type)
            .header(AUTHORIZATION, format!("Bearer {}", self.token.expect("Endpoint needs token")))
            .send()?;

        let response: Value = response.json()?;
        Ok(response)
    }
}
