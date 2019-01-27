//#![allow(dead_code, unused_imports)]

extern crate rvk;
extern crate serde_json;

use std::io;

use rvk::{methods::groups, objects::user::User, APIClient, Params};
use serde_json::*;

fn main() {

    let input = io::stdin();

    println!("Введите свой client_id:");
    let client_id = { // 6835330
        let mut buf = String::new();
        input.read_line(&mut buf).unwrap();
        buf
    };

    let api_version: String = "5.92".to_string();

    println!("\nВставьте эту ссылку в браузер:\nhttps://oauth.vk.com/authorize?client_id={}&display=page&redirect_uri=https://oauth.vk.com/blank.html/callback&scope=friends&response_type=token&v={}\n",
    client_id.trim(), api_version);

    println!("И введите полученный access_token:"); 
    let token = {
        let mut buf = String::new();
        input.read_line(&mut buf).unwrap();
        buf
    };

    // Create an API Client.
    let api = APIClient::new(token.trim().to_string());

    // Create a HashMap to store parameters.
    let mut params_groups = Params::new();

    // Используется связка "поле + значение".

    params_groups.insert("group_id".into(), "61440523".into());
    params_groups.insert("count".into(), "10".into());
    params_groups.insert("fields".into(), "sex, city, bdate, is_closed".into());
    
    println!("\nПередаём следующие данные: {:?}\n", params_groups);
    
    let members = groups::get_members(&api, params_groups);

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
}
