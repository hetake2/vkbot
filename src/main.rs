//#![allow(dead_code, unused_imports)]

extern crate rvk;
extern crate serde_json;
extern crate open;

use std::io;
use std::fs::OpenOptions;
use rvk::{methods::groups, objects::user::User, APIClient, Params};
use serde_json::{json, to_writer_pretty, from_value, Value, from_reader};
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;

// Easy input function.
fn get_input<T>(text: T) -> String 
    where T: std::string::ToString
{
    println!("{}", text.to_string());
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

// JSON reader
fn get_json_data(filename: &str) -> Value
{
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    from_reader(reader).unwrap()
}

fn main() {
    let mut data = get_json_data("login.json");
    let f = OpenOptions::new().write(true).open("login.json").unwrap();
    let w = BufWriter::new(f);

    // Getting client_id's input.
    let mut client_id = data["client_id"].as_str().unwrap().to_string();
    if client_id == "" {
		client_id = get_input("Введите свой client_id:");
        data["client_id"] = json!(client_id);
        to_writer_pretty(w, &data).unwrap();
	};

    // VK API version.  
    let api_version: String = "5.92".to_string();
    let f = OpenOptions::new().write(true).open("login.json").unwrap();
    let w = BufWriter::new(f);
    // Getting token's input.
    let mut token = data["token"].as_str().unwrap().to_string();
    if token == "" {
        let url = format!("https://oauth.vk.com/authorize?client_id={}&display=page&redirect_uri=https://oauth.vk.com/blank.html/callback&scope=friends&response_type=token&v={}",
        client_id, api_version);  
        open::that(url).unwrap();
        token = get_input("Введите полученный access_token из открывшейся страницы:");
        data["token"] = json!(token);
        to_writer_pretty(w, &data).unwrap();
    };
    println!("token {}", token);
    // Create an API Client.
    let api = APIClient::new(token.to_string());
    
    // Create a HashMap to store parameters.
    let mut count_offset = 0;
    let inc_offset = 10; // Default is 0, Max is 1000.

    // URL on get_members VK api: https://vk.com/dev/groups.getMembers
    let mut params_groups = Params::new();
    
    // Adding some "Key" + "Value" to our Hashmap.
    params_groups.insert("group_id".into(), "61440523".into());
    params_groups.insert("count".into(), "10".into());
    params_groups.insert("offset".into(), "0".to_string().into());
    params_groups.insert("fields".into(), "sex, city, bdate, is_closed".into());
        
    println!("\nПередаём следующие данные: {:?}\n", params_groups);
    
    let mut stop = "1".to_string(); // переменная остановки
    while stop.trim() == "1" {
        let members = groups::get_members(&api, params_groups.clone());
        match members {
            Ok(v) => {
                let json_data: Value = from_value(v).unwrap();
                //println!("{:?}\n", json_data);
                let slice = json_data["items"].clone();
                //println!("{:?}\n", slice);
                let users: Vec<User> = from_value(slice).unwrap();
                //println!("{:?}\n", users);
                
                for user in &users {
                    println!(
                        "User ID: {:?}\nName: {} {}\nBirthday: {:?}\nSex: {:?}\nCity: {:?}\n",
                        user.id, user.first_name, user.last_name, user.bdate, user.sex, user.city 
                    );
                };
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
