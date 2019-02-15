//#![allow(dead_code, unused_imports)]

extern crate rvk;
extern crate serde_json;
extern crate open;
extern crate rusqlite;
extern crate chrono;

use std::io;
use std::fs::OpenOptions;
use rvk::{methods::groups, APIClient, Params};
use serde_json::{json, to_writer_pretty, from_value, Value, from_reader};
use std::path::Path;
use std::io::BufReader;
use std::io::BufWriter;
use rvk::API_VERSION;
use rusqlite::{Connection, NO_PARAMS};
use chrono::{NaiveDate, Utc};

// File config for auth
const LOGIN_FILE : &str = "login.json";

// Simple Database
struct DB {
    db : Connection
}

impl DB {

    // constructor
    fn new(file : &str) -> DB {
        let mut d = Connection::open(file).unwrap();
        d.execute("create table if not exists u (i unsigned integer)", NO_PARAMS).unwrap();
        DB {db : d}
    }

    // checks value in database
    fn contains(&self, i : u32) -> bool {
        let r : u32 = self.db.query_row("select count(i) from u where i=?1", &[&i], |r| r.get(0)).unwrap();
        r > 0
    }

    // adding value to database
    fn add(&self, i : u32) {
        if !self.contains(i) {
            self.db.execute("insert into u values (?1)", &[&i]).unwrap();
        }
    }

    // returns length of values
    fn len(&self) -> u32 {
        self.db.query_row("select count(i) from u", NO_PARAMS, |r| r.get(0)).unwrap()
    }
    
    fn print(&self) {
        if self.len() > 0 {
            let mut t = self.db.prepare("select i from u").unwrap();
            for i in t.query_map(NO_PARAMS, |r| -> u32 { r.get(0) } ).unwrap() {
                println!("id {}", i.unwrap());
            }
        } else {
            println!("Nothing");
        }
    }
}

// Easy input function.
fn get_input<T>(text: T) -> String 
    where T: ToString
{
    println!("{}", text.to_string());
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

fn check_token(token : String) -> bool {
    let api = APIClient::new(token);
    let result = groups::get_members(&api, from_value(json!({
        "group_id" : "61440523"
    })).unwrap());
    match result {
        Ok(_v) => { return true }
        Err(_e) => { return false}
    };
}

fn get_data_with_value(key: &str) -> (Value, String) {
    let v = get_json_data(LOGIN_FILE);
    let s = v[key].as_str().unwrap().to_string();
    (v, s)
}

fn get_token(client_id: String) -> String {
    let (mut data, mut token) = get_data_with_value("token");
    if !check_token(token.clone()) {
        let url = format!("https://oauth.vk.com/authorize?client_id={}&display=page&redirect_uri=https://oauth.vk.com/blank.html/callback&scope=friends&response_type=token&v={}",
        client_id, API_VERSION);
        open::that(url).unwrap();
        while !check_token(token.clone()) {
            token = get_input("Type your 'access_token' from opened browser page:");
        };
        data["token"] = json!(token);
        save_json(&data)
    };
    token
}

fn get_client_id() -> String {
    let (mut data, mut client_id) = get_data_with_value("client_id");
    if client_id == "" {
		client_id = get_input("Type your 'client_id':");
        data["client_id"] = json!(client_id);
        save_json(&data);
	};
    client_id
}

fn save_json(data: &Value) {
    let f = OpenOptions::new().write(true).open(LOGIN_FILE).unwrap();
    let w = BufWriter::new(f);
    let _t = to_writer_pretty(w, data).unwrap();
}

// JSON reader, that also creates a file with the same name if it doesn't exist.
fn get_json_data(filenames: &str) -> Value {
    let filename = Path::new(filenames);
    if !filename.exists() {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(filename)
            .unwrap();
        let w = BufWriter::new(file);
        let _t = to_writer_pretty(w, &json!({
            "token" : "",
            "client_id" : ""
        })).unwrap();
    };
    let file = OpenOptions::new()
        .read(true)
        .open(filename)
        .unwrap();
    let reader = BufReader::new(file);
    from_reader(reader).unwrap()
}

fn main() {
    // first database
    let mut d = DB::new("add.db");
    
    // current date
    let current_date : NaiveDate = Utc::today().naive_utc();

    // Getting client_id's input.
    let client_id = get_client_id();

    // Getting token's input.
    let token = get_token(client_id);
    
    // Create an API Client.
    let api = APIClient::new(token);
    
    // Create a HashMap to store parameters.
    let mut params_groups: Params = from_value(json!(
        {
            "group_id" : "61440523",
            "count" : "1000",
            "offset" : "0", // Don't change.
            "fields" : "sex, city, bdate"
        }
    )).unwrap();

    println!("\nWe transfer the following data: {:?}\n", params_groups);

    let mut stop = "1".to_string(); // "While"'s exit.
    let mut offset = 0;
    let mut count = 0;
    let count_inc = 1000;
    while stop == "1" && (count == 0 || offset < count) {
        // URL on get_members VK API: https://vk.com/dev/groups.getMembers
        let members = groups::get_members(&api, params_groups.clone());
        match members {
            Ok(v) => {
                let json_data: Value = from_value(v).unwrap();
                let count: u32 = from_value(json_data["count"].clone()).unwrap();
                println!("Parse left: {}\n", count);
                let items = json_data["items"].clone();
                for i in 0..1000 {
                    let user = items[i].clone();
                    let date = user["bdate"].as_str().unwrap_or("").to_string();
                    if user["sex"].as_u64().unwrap_or(0) == 1 &&
                    user["city"]["id"].as_u64().unwrap_or(0) == 1 {
                        let date = NaiveDate::parse_from_str(&date, "%d.%m.%Y");
                        match date {
                            Ok(v) => {
                                let result = (current_date - v).num_days() / 365;
                                println!("{} {} ей {} лет", user["first_name"].as_str().unwrap(), user["last_name"].as_str().unwrap(), result);
                            },
                            Err(e) => std::thread::sleep(std::time::Duration::from_secs(0))
                        };
                    };
                };
            }
            Err(e) => println!("{}", e)
        };
        offset += count_inc;
        params_groups.insert("offset".into(), offset.to_string().into());
        
        // To exit from "While" or continue.
        stop = get_input("To continue type 1:"); 
        println!("stop = {}", stop);
    };
}
