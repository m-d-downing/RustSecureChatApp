use eframe::{egui, egui::CentralPanel, egui::Context, egui::Layout, epi::App, epi::Frame};
use reqwest::StatusCode;
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender}, time::SystemTime,
};
use std::time::SystemTime;

#[derive(Deserialize, Clone)]
struct User {
    user_id: String,
    user_name: String,
}

#[derive(Deserialize)]
struct Users {
    users: Vec<User>,
}
#[derive(Deserialize, Clone, Debug)]

struct Message {
    message: String,
    sent_at: String,
}
#[derive(Deserialize)]

struct Messages {
    messages: Vec<Message>,
}

enum AppState {
    RenderLogin,
    RenderChat,
    RenderLobby,
}

impl Default for AppState {
    fn default() -> AppState {
        AppState::RenderLogin
    }
}

struct SecureChatApp {
    message: String,
    chat_history: String,
    state: AppState,
    user: Option<User>,
    count: u64,
    users_fetched: bool,
    available_users: Vec<User>,
    chatting_with: String,
    messages: Vec<Message>,
    sent: Vec<Message>,
    send_messages: Sender<Vec<Message>>,
    recv_messages: Receiver<Vec<Message>>,
}

impl Default for SecureChatApp {
    fn default() -> Self {
        let (send_messages, recv_messages) = channel();
        Self{message: String::new(),
            chat_history: String::new(),
            state: AppState::default(),
            user: None,
            count: 0,
            users_fetched: false,
            available_users: Vec::new(),
            chatting_with: String::new(),
            messages:Vec::new(),
            sent:Vec::new(),
            send_messages, recv_messages}
    }
}

fn get_available_users(available_users: &mut Vec<User>) {
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

fn send_message(message: &str, sender: &str, recipient: &str) -> bool {
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

fn get_messages(sender: &str, recipient: &str) -> Vec<Message> {
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
                    Ok(data) => return data.messages,
                    Err(_) => todo!(),
                }
            }
            return Vec::new();
        }
        Err(_) => todo!(),
    }
}

fn set_user_status(status: &str, user: &mut Option<User>) -> bool {
    let mut map = HashMap::new();

    map.insert("user_id", "5cc0f84a-35b0-4331-8798-47013f9eeb40");
    map.insert("status", status);

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

impl SecureChatApp {
    fn render_chat(&mut self, ctx: &Context) {
        if self.count % 200 == 0 {
            match &self.user {
                Some(user) => {
                    let send = self.send_messages.clone();
                    let sender_id = user.user_id.clone();
                    let recipient_id = self.chatting_with.clone();
                    std::thread::spawn(move || {
                        let messages =
                            get_messages(&sender_id.as_str(), &recipient_id.as_str());
                        send.send(messages).expect("Whoops!");
                    });
                }
             None => todo!(),
            }
        }    
                if let Ok(response) = self.recv_messages.try_recv() {
            
                            self.messages = response;
                            for message in &mut self.messages {
                                let new_string = message.message.clone()
                                    + " "
                                    + message.sent_at.as_str()
                                    + "\n";
                                self.chat_history += new_string.as_str();
                                
                            }
                        
                    
                }
        
        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::bottom_up(eframe::egui::Align::LEFT), |ui| {
                let text_response = ui.add(
                    egui::TextEdit::singleline(&mut self.message)
                        .desired_width(600.0)
                        .lock_focus(true),
                );
                if text_response.lost_focus() {
                    if self.message == "" {
                        return;
                    }

                    match &self.user {
                        Some(user) => {
                            send_message(
                                self.message.as_str(),
                                user.user_id.as_str(),
                                self.chatting_with.as_str(),
                            );
                        }
                        None => {
                            println!("No User");
                        }
                    }
                    self.sent.push(Message{message:self.message, sent_at: SystemTime::now() });
                    self.message.push('\n');
                    self.chat_history += &self.message;
                    self.message.clear();
                }
                text_response.request_focus();
                ui.heading(&self.chat_history);
            })
        });
    }
    fn render_login(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.count.to_string());

            ui.vertical_centered(|ui| {
                ui.add_space(200.0);
                let response = ui.add_sized([200.0, 50.0], egui::Button::new("Login"));
                if response.clicked() {
                    if set_user_status("signedin", &mut self.user) {
                        self.state = AppState::RenderLobby
                    }
                }
            })
        });
    }
    fn render_lobby(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            if self.users_fetched {
                match &self.user {
                    Some(user) => {
                        ui.heading(String::from("User: ") + &user.user_name);
                        ui.heading("Start Chat With:");
                        for available_user in &self.available_users {
                            // if available_user.user_id == user.user_id {
                            //     continue;
                            // }
                            let response = ui.add_sized(
                                [75.0, 35.0],
                                egui::Button::new(String::from(&available_user.user_name)),
                            );
                            if response.clicked() {
                                println!("Starting chat with {}", &available_user.user_id);
                                self.chatting_with = available_user.user_id.clone();
                                self.state = AppState::RenderChat;
                            }
                            ui.add_space(20.0)
                        }

                        ui.add_space(100.0);

                        if ui.button("Log Out").clicked() {
                            if set_user_status("signedout", &mut self.user) {
                                self.state = AppState::RenderLobby;
                            }
                        }
                    }
                    None => {
                        ui.heading("Error logging in user");
                    }
                }
            } else {
                ui.heading("Loading");
                get_available_users(&mut self.available_users);
                self.users_fetched = true
            }
        });
    }
}

impl App for SecureChatApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &Context, _frame: &Frame) {
        self.count = self.count + 1;
        match &self.state {
            AppState::RenderLogin => self.render_login(ctx),
            AppState::RenderChat => self.render_chat(ctx),
            AppState::RenderLobby => self.render_lobby(ctx),
        }
    }
}

fn main() {
    let app = SecureChatApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
