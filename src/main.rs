//#![allow(dead_code, unused_imports)]

extern crate rvk;
extern crate serde_json;

use std::{env, io};

use rvk::{methods::*, methods::groups, objects::user::User, APIClient, Params};
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
    let mut params_users = Params::new();
    let mut params_groups = Params::new();

    // Используется связка "поле + значение".
    //params_users.insert("user_ids".into(), "528551383".into());
    
    params_groups.insert("group_id".into(), "74314716".into());
    //params_groups.insert("sort".into(), "id_asc".into());
    //params_groups.insert("offset".into(), "0".into());
    //params_groups.insert("count".into(), "1".into());
    params_groups.insert("fields".into(), "sex".into());
    println!("\nПередаём следующие данные: {:?}\n", params_groups);
    let members = groups::get_members(&api, params_groups);
    //let res = users::get(&api, params_users);

    match members {
        Ok(v) => {
            let users: Vec<User> = from_value(v).unwrap();
            let user = &users[0];
            println!(
                "User #{} {} {}", 
                user.id, user.first_name, user.last_name
            );
        }
        Err(e) => println!("{}", e), 
    };

    // match res {
    //     Ok(v) => { // v is `serde_json::Value`
    //         let users: Vec<User> = from_value(v).unwrap();
    //         let user = &users[0];

    //         println!(
    //             "User #{} is {} {}.",
    //             user.id, user.first_name, user.last_name
    //         );
    //     }
    //     Err(e) => println!("{}", e),
    // };
}
