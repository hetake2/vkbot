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
    methods::{friends::add, groups::get_members},
    Params,
};
use serde_json::{from_value, json, Value};

fn main() {
    // First database for user storage.
    let d = DB::new("add.db");
    // Second databse for checked users.
    let d2 = DB::new("check.db");

    // Current date.
    let current_date: NaiveDate = Utc::today().naive_utc();

    // Create an API Client.
    let api = get_api();

    // Create a HashMap to store parameters.
    let mut params_groups: Params = from_value(json!(
        {
            "group_id" : "53664217",
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
                    if from_value(user["sex"].clone()).unwrap_or(0) == 1
                        && from_value(user["city"]["id"].clone()).unwrap_or(0) == 1
                        && from_value(user["can_send_friend_request"].clone()).unwrap_or(0) == 1
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

    // This part is about sending friend requests with messages.
    match get_input("Start send requests? 1 for Yes.").as_ref() {
        "1" => {
            for i in d.get_vec() {
                let user_id = i;
                if !d2.contains(user_id) {
                    //let greetings_file = get_json_data("greetings.json");
                    //let greetings = greetings_file["greetings"].clone();
                    //let text = greetings[""].clone();
                    let text = "Добрый день)";
                    let mut params: Params = from_value(json!(
                    {
                        "user_id" : user_id.to_string(),
                        "text" : text,
                    }))
                    .unwrap();
                    let mut completed = false;
                    while !completed {
                        println!("\nDEBUG: {:?}", params);
                        match add(&api, params.clone()) {
                            Ok(_) => {
                                d2.add(user_id);
                                completed = true
                            }
                            Err(API(e)) => error_handler(e, &mut params),
                            _ => {}
                        }
                        //sleep(Duration::from_secs(300));
                    }
                }
            }
        }
        _ => {}
    }
}
