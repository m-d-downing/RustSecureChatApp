use crate::DeleteResponse;
use crate::Message;
use crate::Messages;
use crate::User;
use crate::Users;
use rand;
use reqwest::StatusCode;
use rsa::pkcs8::DecodePublicKey;
use rsa::{self, PaddingScheme, PublicKey};

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

pub fn send_message(message: &str, sender: &str, recipient: &str, public_key_str: &str) -> bool {
    let mut rng = rand::thread_rng();

    let bits = 2048;
    let padding = PaddingScheme::new_pkcs1v15_encrypt();

    let public_key = rsa::RsaPublicKey::from_public_key_pem(public_key_str).unwrap();
    let enc_data = public_key
        .encrypt(&mut rng, padding, &message.as_bytes())
        .expect("Failed to encrypt");

    let mut map = HashMap::new();
    let encoded_message = format!("{:?}", enc_data);
    map.insert("sender", sender);
    map.insert("recipient", recipient);
    map.insert("message", &encoded_message.as_str());

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
pub fn delete_messages(sender: String, recipient: String) -> bool {
    let mut map = HashMap::new();

    map.insert("sender", sender);
    map.insert("recipient", recipient);

    let client = reqwest::blocking::Client::new();

    match client
        .post("https://0ibh96tdhk.execute-api.us-west-2.amazonaws.com/deleteMessages")
        .json(&map)
        .send()
    {
        Ok(response) => {
            println!("{:?}", response);
            if response.status() == StatusCode::OK {
                match response.json::<DeleteResponse>() {
                    Ok(data) => {
                        return data.success;
                    }
                    Err(_) => todo!(),
                }
            }
            return false;
        }
        Err(_) => todo!(),
    }
}

pub fn set_user_status(key: &str, user: &mut Option<User>) -> bool {
    match user {
        Some(usr) => {
            let mut map = HashMap::new();
            let pub_key = String::from(key);

            map.insert("user_id", &usr.user_id);
            map.insert("key", &pub_key);

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
