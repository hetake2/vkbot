#![allow(dead_code)]
extern crate chrono;
extern crate open;
extern crate rusqlite;
extern crate rvk;
extern crate serde_json;

use rusqlite::{Connection, NO_PARAMS};
use rvk::{error::APIError, methods::groups::get_members, APIClient, Params, API_VERSION};
use serde_json::{from_reader, from_value, json, to_string_pretty, to_writer_pretty, Value};
use std::fs::OpenOptions;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

// File config for auth.
const LOGIN_FILE: &str = "login.json";

// Simple Database.
pub struct DB {
    db: Connection,
}

pub fn error_handler(e: APIError, params: &mut Params) {
    match e.code() {
        14 => {
            let json_data = e.extra();
            let captcha_sid: String = from_value(json_data["captcha_sid"].clone()).unwrap();
            let captcha_img: String = from_value(json_data["captcha_img"].clone()).unwrap();
            println!("{}\n", captcha_img);
            open::that(captcha_img).unwrap();
            let captcha_key = get_input("\nWaiting for captcha...");
            println!("sid = {}, key = {}", captcha_sid, captcha_key);
            params.extend(
                from_value::<Params>(json!({
                    "captcha_sid": captcha_sid,
                    "captcha_key": captcha_key
                }))
                .unwrap(),
            );
            sleep(Duration::from_secs(5));
        }
        6 => {
            println!("Превышение запросов за 1 секунду");
            sleep(Duration::from_secs(1))
        }
        _ => println!("{:?}", to_string_pretty(&json!(e.extra()))),
    }
}

impl DB {
    // Constructor.
    pub fn new(file: &str) -> DB {
        let d = Connection::open(file).unwrap();
        d.execute(
            "create table if not exists u (i unsigned integer)",
            NO_PARAMS,
        )
        .unwrap();
        DB { db: d }
    }

    // Checks value in database.
    pub fn contains(&self, i: u32) -> bool {
        let r: u32 = self
            .db
            .query_row("select count(i) from u where i=?1", &[&i], |r| r.get(0))
            .unwrap();
        r > 0
    }

    // Adding value to database.
    pub fn add(&self, i: u32) {
        if !self.contains(i) {
            self.db.execute("insert into u values (?1)", &[&i]).unwrap();
        }
    }

    // Returns length of values.
    pub fn len(&self) -> u32 {
        self.db
            .query_row("select count(i) from u", NO_PARAMS, |r| r.get(0))
            .unwrap()
    }

    // Returns an array with indexes from database.
    pub fn get_vec(&self) -> Vec<u32> {
        let mut result: Vec<u32> = Vec::new();
        let mut t = self.db.prepare("select i from u").unwrap();
        for i in t.query_map(NO_PARAMS, |r| -> u32 { r.get(0) }).unwrap() {
            result.push(i.unwrap());
        }
        result
    }

    pub fn print(&self) {
        if self.len() > 0 {
            for i in self.get_vec() {
                println!("id {}", i);
            }
        } else {
            println!("Nothing");
        }
    }

    pub fn delete(&self, i: u32) {
        if self.contains(i) {
            self.db.execute("delete from u where i=?1", &[&i]).unwrap();
        }
    }

    pub fn clean(&self) {
        if self.len() > 0 {
            for i in self.get_vec() {
                self.delete(i)
            }
        } else {
            println!("Is clean!!!")
        }
    }
}

pub fn get_api() -> APIClient {
    APIClient::new(get_token())
}

// Easy input function.
pub fn get_input<T>(text: T) -> String
where
    T: ToString,
{
    println!("{}", text.to_string());
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

fn check_token(token: String) -> bool {
    let api = APIClient::new(token);
    let result = get_members(
        &api,
        from_value(json!({
            "group_id" : "61440523"
        }))
        .unwrap(),
    );
    match result {
        Ok(_) => return true,
        Err(_) => return false,
    };
}

fn get_data_with_value(key: &str) -> (Value, String) {
    let v = get_json_data(LOGIN_FILE);
    let s: String = from_value(v[key].clone()).unwrap();
    (v, s)
}

fn get_token() -> String {
    let (mut data, mut token) = get_data_with_value("token");
    if !check_token(token.clone()) {
        let client_id = get_client_id();
        let url = format!("https://oauth.vk.com/authorize?client_id={}&display=page&redirect_uri=https://oauth.vk.com/blank.html/callback&scope=friends&response_type=token&v={}",
        client_id, API_VERSION);
        while !check_token(token.clone()) {
            open::that(url.clone()).unwrap();
            token = get_input("Type your 'access_token' from opened browser page:");
        }
        data["client_id"] = json!(client_id);
        data["token"] = json!(token);
        save_json(&data, LOGIN_FILE)
    };
    token
}

fn get_client_id() -> String {
    let (_, mut client_id) = get_data_with_value("client_id");
    if client_id == "" {
        client_id = get_input("Type your 'client_id':");
    };
    client_id
}

fn save_json(data: &Value, file: &str) {
    let f = OpenOptions::new()
        .create(true)
        .write(true)
        .open(file)
        .unwrap();
    let w = BufWriter::new(f);
    let _t = to_writer_pretty(w, data).unwrap();
}

// JSON reader, that also creates a file with the same name if it doesn't exist.
fn get_json_data(filenames: &str) -> Value {
    let filename = Path::new(filenames);
    if !filename.exists() {
        let _t = save_json(
            &json!({
                "token" : "",
                "client_id" : ""
            }),
            LOGIN_FILE,
        );
    };
    let file = OpenOptions::new().read(true).open(filename).unwrap();
    let reader = BufReader::new(file);
    from_reader(reader).unwrap()
}
