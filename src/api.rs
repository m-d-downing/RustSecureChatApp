use reqwest::StatusCode;

use crate::Message;
use crate::Messages;
use crate::User;
use crate::Users;

use std::collections::HashMap;

pub fn get_available_users(available_users: &mut Vec<User>) {
    match reqwest::blocking::get("https://0ibh96tdhk.execute-api.us-west-2.amazonaws.com/users") {
        Ok(response) => {
            if response.status() == reqwest::StatusCode::OK {
                match response.json::<Users>() {
                    Ok(users) => {
                        *available_users = users.users;
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
        }
        Err(_) => println!("Could not make request"),
    }
}

pub fn send_message(message: &str, sender: &str, recipient: &str) -> bool {
    let mut map = HashMap::new();

    map.insert("sender", sender);
    map.insert("recipient", recipient);
    map.insert("message", message);

    let client = reqwest::blocking::Client::new();

    match client
        .post("https://0ibh96tdhk.execute-api.us-west-2.amazonaws.com/message")
        .json(&map)
        .send()
    {
        Ok(response) => {
            if response.status() == StatusCode::OK {
                println!("Fetched");
                true
            } else {
                println!("Fetched, but fucked");
                false
            }
        }
        Err(_) => {
            println!("Failed to update status");
            false
        }
    }
}

pub fn get_messages(sender: String, recipient: String) -> Vec<Message> {
    println!("{},{}", sender, recipient);
    let mut map = HashMap::new();

    map.insert("sender", sender);
    map.insert("recipient", recipient);

    let client = reqwest::blocking::Client::new();

    match client
        .post("https://0ibh96tdhk.execute-api.us-west-2.amazonaws.com/getmessages")
        .json(&map)
        .send()
    {
        Ok(response) => {
            if response.status() == StatusCode::OK {
                match response.json::<Messages>() {
                    Ok(data) => {
                        println!("{:?}", data.messages);
                        return data.messages;
                    }
                    Err(_) => todo!(),
                }
            }
            return Vec::new();
        }
        Err(_) => todo!(),
    }
}

pub fn set_user_status(status: &str, user: &mut Option<User>) -> bool {
    match user {
        Some(usr) => {
            let mut map = HashMap::new();
            let user_status = String::from(status);

            map.insert("user_id", &usr.user_id);
            map.insert("status", &user_status);

            let client = reqwest::blocking::Client::new();

            match client
                .post("https://0ibh96tdhk.execute-api.us-west-2.amazonaws.com/status")
                .json(&map)
                .send()
            {
                Ok(response) => {
                    if response.status() == StatusCode::OK {
                        println!("Fetched");
                        match response.json::<User>() {
                            Ok(response_user) => {
                                *user = Some(response_user);
                            }
                            Err(e) => println!("{:?}", e),
                        }
                        true
                    } else {
                        println!("Server Error");
                        false
                    }
                }
                Err(_) => {
                    println!("Failed to update status");
                    false
                }
            }
        }
        None => return false,
    }
}
