//#![allow(dead_code, unused_imports)]

extern crate rvk;
extern crate serde_json;
extern crate open;

use std::io;
use std::fs::OpenOptions;
use rvk::{methods::groups, APIClient, Params};
use serde_json::{json, to_writer_pretty, from_value, Value, from_reader};
use std::path::Path;
use std::io::BufReader;
use std::io::BufWriter;

// Easy input function.
fn get_input<T>(text: T) -> String 
    where T: ToString
{
    println!("{}", text.to_string());
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

// JSON reader, that also creates a file with the same name if it doesn't exist.
fn get_json_data(filename: &Path) -> Value {
    if !filename.exists() {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(filename)
            .unwrap();
        let w = BufWriter::new(file);
        let t = to_writer_pretty(w, &json!({
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
    let path_of_login = Path::new("login.json");
    let mut data = get_json_data(path_of_login);

    // Getting client_id's input.
    let mut client_id = data["client_id"].as_str().unwrap().to_string();
    if client_id == "" {
		client_id = get_input("Введите свой client_id:");
        data["client_id"] = json!(client_id);
        let f = OpenOptions::new().write(true).open("login.json").unwrap();
        let w = BufWriter::new(f);
        to_writer_pretty(w, &data).unwrap();
	};

    // VK API version.  
    let api_version: String = "5.92".to_string();

    // Getting token's input.
    let mut token = data["token"].as_str().unwrap().to_string();
    if token == "" {
        let url = format!("https://oauth.vk.com/authorize?client_id={}&display=page&redirect_uri=https://oauth.vk.com/blank.html/callback&scope=friends&response_type=token&v={}",
        client_id, api_version);  
        open::that(url).unwrap();
        token = get_input("Введите полученный access_token из открывшейся страницы:");
        data["token"] = json!(token);
        let f = OpenOptions::new().write(true).open("login.json").unwrap();
        let w = BufWriter::new(f);
        to_writer_pretty(w, &data).unwrap();
    };

    // Create an API Client.
    let api = APIClient::new(token);
    
    // Create a HashMap to store parameters.
    let mut count_offset = 0;
    let inc_offset = 10; // Default is 0, Max is 1000.

    // URL on get_members VK API: https://vk.com/dev/groups.getMembers
    let mut params_groups: Params = from_value(json!(
        {
            "group_id" : "61440523",
            "count" : "1",
            "offset" : "1",
            "fields" : "sex, city, bdate"
        }
    )).unwrap();

    println!("\nПередаём следующие данные: {:?}\n", params_groups);
    
    let mut stop = "1".to_string(); // "While"'s exit.
    while stop.trim() == "1" {
        let members = groups::get_members(&api, params_groups.clone());
        match members {
            Ok(v) => {
                let json_data: Value = from_value(v).unwrap();
                //println!("{:?}\n", json_data);
                //let slice = json_data["items"].clone();
                //println!("{:?}\n", slice);
                //let users: Vec<User> = from_value(slice).unwrap();
                //println!("{:?}\n", users);

                let f = OpenOptions::new()
                .write(true)
                .create(true)
                .open("accounts.json")
                .unwrap();
                let w = BufWriter::new(f);
                to_writer_pretty(w, &json_data).unwrap();
                
            }
            Err(e) => println!("{}", e)
        };
        count_offset += inc_offset;
        params_groups.insert("offset".into(), count_offset.to_string().into());
        
        // ограничение, для завершения цикла, так же нужна задержка, если убрать эту заслонку, дабы не забанили ор) от ддос атаки запросами
        stop = get_input("Для продолжения введите 1:"); 
        println!("stop = {}", stop);
    };
}
