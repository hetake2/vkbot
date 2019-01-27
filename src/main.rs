#![allow(dead_code, unused_imports)]

extern crate rvk;
extern crate serde_json;

use std::{env, io};

use rvk::{methods::*, objects::user::User, APIClient, Params};
use serde_json::from_value;

fn main() {

    let input = io::stdin();

    println!("Введите свой client_id:");
    let client_id = { // 6835330
        let mut buf = String::new();
        input.read_line(&mut buf).unwrap();
        buf
    };

    let api_version: String = "5.92".to_string();

    println!("Вставьте эту ссылку в браузер:\nhttps://oauth.vk.com/authorize?client_id={}&display=page&redirect_uri=https://oauth.vk.com/blank.html/callback&scope=friends&response_type=token&v={}\n",
    client_id.trim(), api_version);

    println!("И введите полученный access_token:"); 
    let token = {
        let mut buf = String::new();
        input.read_line(&mut buf).unwrap();
        buf
    };

    // Create an API Client.
    let mut api = APIClient::new(token.trim().to_string());

    // Create a HashMap to store parameters.
    let mut params = Params::new();

    // Используется связка "поле + значение".
    params.insert("user_ids".into(), "528551383".into());

    let res = users::get(&api, params);

    match res {
        Ok(v) => { // v is `serde_json::Value`
            let users: Vec<User> = from_value(v).unwrap();
            let user = &users[0];

            println!(
                "User #{} is {} {}.",
                user.id, user.first_name, user.last_name
            );
        }
        Err(e) => println!("{}", e),
    };
}
