mod api;

use eframe::{egui, egui::CentralPanel, egui::Context, egui::Layout, epi::App, epi::Frame};
use serde::Deserialize;
use std::{
    env,
    sync::mpsc::{channel, Receiver, Sender},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Deserialize, Clone)]
pub struct User {
    user_id: String,
    user_name: String,
}

#[derive(Deserialize)]
pub struct Users {
    users: Vec<User>,
}
#[derive(Deserialize, Clone, Debug)]

pub struct Message {
    message: String,
    sent_at: String,
}
#[derive(Deserialize)]

pub struct Messages {
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
        Self {
            message: String::new(),
            chat_history: String::new(),
            state: AppState::default(),
            user: None,
            count: 0,
            users_fetched: false,
            available_users: Vec::new(),
            chatting_with: String::new(),
            messages: Vec::new(),
            sent: Vec::new(),
            send_messages,
            recv_messages,
        }
    }
}

impl SecureChatApp {
    fn new(user_id: String, user_name: String) -> SecureChatApp {
        let mut app = SecureChatApp::default();
        app.user = Some(User { user_id, user_name });
        return app;
    }

    fn render_chat(&mut self, ctx: &Context) {
        if self.count % 200 == 0 {
            match &self.user {
                Some(user) => {
                    let send = self.send_messages.clone();
                    let sender_id = user.user_id.clone();
                    let recipient_id = self.chatting_with.clone();
                    std::thread::spawn(move || {
                        let messages =
                            api::get_messages(&sender_id.as_str(), &recipient_id.as_str());
                        send.send(messages).expect("Whoops!");
                    });
                }
                None => todo!(),
            }
        }
        if let Ok(mut response) = self.recv_messages.try_recv() {
            let mut sent_messages: Vec<Message> = self.sent.clone();
            response.append(&mut sent_messages);
            response.sort_by(|a, b| a.sent_at.cmp(&b.sent_at));
            println!("{:?}", response);
            self.messages = response;

            self.chat_history = String::new();
            for message in &mut self.messages {
                let new_string = message.message.clone() + " " + message.sent_at.as_str() + "\n";
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
                            api::send_message(
                                self.message.as_str(),
                                user.user_id.as_str(),
                                self.chatting_with.as_str(),
                            );
                        }
                        None => {
                            println!("No User");
                        }
                    }
                    self.sent.push(Message {
                        message: self.message.clone(),
                        sent_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time Error")
                            .as_millis()
                            .to_string(),
                    });
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
                    if api::set_user_status("signedin", &mut self.user) {
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
                            if api::set_user_status("signedout", &mut self.user) {
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
                api::get_available_users(&mut self.available_users);
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
    for (n, v) in env::vars() {
        println!("{}: {}", n, v);
    }
    let user_id = match env::var_os("CAPSTONE_CHAT_ID") {
        Some(v) => v.into_string().unwrap(),
        None => panic!("$USER is not set"),
    };
    let user_name = match env::var_os("CAPSTONE_CHAT_NAME") {
        Some(v) => v.into_string().unwrap(),
        None => panic!("$USER is not set"),
    };

    let app = SecureChatApp::new(user_id, user_name);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
