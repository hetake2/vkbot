//#![allow(dead_code, unused_imports)]

extern crate chrono;
extern crate open;
extern crate rusqlite;
extern crate rvk;
extern crate serde_json;

mod lib;
use chrono::{NaiveDate, Utc};
use lib::*;
use rvk::{
    error::Error::API,
    methods::{friends::add, friends::are_friends, groups::get_members},
    APIClient, Params,
};
use serde_json::{from_value, json, to_string_pretty, Value};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // First database.
    let d = DB::new("add.db");

    // Current date.
    let current_date: NaiveDate = Utc::today().naive_utc();

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
            "sort" : "id_desc",
            "count" : "1000",
            "offset" : "0", // Don't change.
            "fields" : "sex, city, bdate, can_send_friend_request"
        }
    ))
    .unwrap();

    println!("\nWe transfer the following data: {:?}", params_groups);

    let mut stop = "1".to_string(); // "While"'s exit.
    let mut offset = 0;
    let mut count = 0;
    let count_inc = 1000;
    while stop == "1" && (count == 0 || offset < count) {
        // URL on get_members VK API: https://vk.com/dev/groups.getMembers
        let members = get_members(&api, params_groups.clone());
        match members {
            Ok(v) => {
                // Our JSON data with array (items) of users.
                let json_data: Value = from_value(v).unwrap();
                count = from_value(json_data["count"].clone()).unwrap();
                println!("\nParse left: {}\n", count - offset);

                // Our filter to get certain ids.
                let items = json_data["items"].clone();
                for i in 0..1000 {
                    let user = items[i].clone();
                    // Getting user's birthday.
                    let date = user["bdate"].as_str().unwrap_or("").to_string();
                    // Getting user's sex and location.
                    if user["sex"].as_u64().unwrap_or(0) == 1
                        && user["city"]["id"].as_u64().unwrap_or(0) == 1
                        && user["can_send_friend_request"].as_u64().unwrap_or(0) == 1
                    {
                        let date = NaiveDate::parse_from_str(&date, "%d.%m.%Y");
                        match date {
                            Ok(v) => {
                                let result = (current_date - v).num_days() / 365;
                                if result > 16 && result < 26 {
                                    println!(
                                        "{} {}, {} years old;",
                                        user["first_name"].as_str().unwrap(),
                                        user["last_name"].as_str().unwrap(),
                                        result
                                    );
                                    d.add(from_value(user["id"].clone()).unwrap())
                                }
                            }
                            Err(_) => {}
                        };
                    };
                }
            }
            Err(e) => println!("{}", e),
        };
        offset += count_inc;
        if offset > count {
            break;
        }
        params_groups.insert("offset".into(), offset.to_string().into());

        // To exit from "While" or continue.
        stop = get_input("\nTo continue type 1:");
        println!("stop = {}", stop);
    }
    println!("\nTotal users in DataBase: {}\n", d.len());

    match get_input("Start send requests? 1 for Yes.").as_ref() {
        "1" => {
            for i in d.get_vec() {
                let user_id = i;
                let are_friends_params: Params = from_value(json!(
                {
                    "user_ids" : user_id.to_string(),
                    "need_sign" : "1",
                }))
                .unwrap();
                let response = are_friends(&api, are_friends_params);
                match response {
                    Ok(v) => {
                        let json_data: Value = from_value(v).unwrap();
                        let resp = json_data["response"].clone();
                        let friend_status = resp["friend_status"].clone();
                        if friend_status.as_u64().unwrap_or(5) != 1
                            || friend_status.as_u64().unwrap_or(5) != 3
                            || friend_status.as_u64().unwrap_or(5) != 2
                        {
                            let text = "Привет)";
                            let mut params: Params = from_value(json!(
                            {
                                "user_id" : user_id.to_string(),
                                "text" : text,
                            }))
                            .unwrap();
                            let mut completed = false;
                            while !completed {
                                println!("\n{:?}", params);
                                match add(&api, params.clone()) {
                                    Ok(_) => completed = true,
                                    Err(e) => match e {
                                        API(e) => match e.code() {
                                            14 => {
                                                let json_data = e.extra();
                                                let captcha_sid: String =
                                                    from_value(json_data["captcha_sid"].clone())
                                                        .unwrap();
                                                let captcha_img: String =
                                                    from_value(json_data["captcha_img"].clone())
                                                        .unwrap();
                                                println!("{}\n", captcha_img);
                                                open::that(captcha_img).unwrap();
                                                let captcha_key =
                                                    get_input("\nWaiting for captcha...");
                                                println!(
                                                    "sid = {}, key = {}",
                                                    captcha_sid, captcha_key
                                                );
                                                params.insert(
                                                    "captcha_sid".into(),
                                                    captcha_sid.into(),
                                                );
                                                params.insert(
                                                    "captcha_key".into(),
                                                    captcha_key.into(),
                                                );

                                                sleep(Duration::from_secs(5));
                                            }
                                            6 => {
                                                println!(
                                                    "Превышение запросов за 1 секунду"
                                                );
                                                sleep(Duration::from_secs(1))
                                            }
                                            _ => println!(
                                                "{:?}",
                                                to_string_pretty(&json!(e.extra()))
                                            ),
                                        },
                                        _ => {}
                                    },
                                }
                                //sleep(Duration::from_secs(300));
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        _ => {}
    };
}
