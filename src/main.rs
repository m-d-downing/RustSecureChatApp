mod api;
use eframe::{egui, egui::CentralPanel, egui::Context, egui::Layout, epi::App, epi::Frame};
use rand;
use rsa::{
    self,
    pkcs8::{EncodePublicKey, LineEnding},
    PaddingScheme,
};
use serde::Deserialize;
use std::{
    env,
    process::exit,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Deserialize, Clone)]
pub struct User {
    user_id: String,
    user_name: String,
    public_key: String,
}

#[derive(Deserialize)]
pub struct Users {
    users: Vec<User>,
}
#[derive(Deserialize)]

pub struct DeleteResponse {
    success: bool,
}
#[derive(Deserialize)]

pub struct Message {
    message: String,
    sent_at: String,
}
#[derive(Deserialize)]

pub struct Messages {
    messages: Vec<Message>,
}

pub struct DisplayMessage {
    message: String,
    sent_at: String,
    user_name: String,
}

#[derive(Deserialize)]

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
    chatting_with: Option<User>,
    messages: Vec<DisplayMessage>,
    sent: Vec<Message>,
    send_messages: Sender<Vec<Message>>,
    recv_messages: Receiver<Vec<Message>>,
    public_key: rsa::RsaPublicKey,
    private_key: rsa::RsaPrivateKey,
}

impl Default for SecureChatApp {
    fn default() -> Self {
        let mut rng = rand::thread_rng();

        let bits = 2048;
        let private_key = rsa::RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate key");
        let public_key = rsa::RsaPublicKey::from(&private_key);
        let (send_messages, recv_messages) = channel();
        Self {
            message: String::new(),
            chat_history: String::new(),
            state: AppState::default(),
            user: None,
            count: 0,
            users_fetched: false,
            available_users: Vec::new(),
            chatting_with: None,
            messages: Vec::new(),
            sent: Vec::new(),
            send_messages,
            recv_messages,
            public_key,
            private_key,
        }
    }
}

impl SecureChatApp {
    fn new(user_id: String, user_name: String) -> SecureChatApp {
        let mut app = SecureChatApp::default();
        app.user = Some(User {
            user_id,
            user_name,
            public_key: app.public_key.to_public_key_pem(LineEnding::CRLF).unwrap(),
        });
        return app;
    }

    fn render_chat(&mut self, ctx: &Context) {
        if let Ok(response) = self.recv_messages.try_recv() {
            let mut sent_messages: Vec<DisplayMessage> = Vec::new();
            let mut recvd_messages: Vec<DisplayMessage> = Vec::new();

            for msg in &self.sent {
                sent_messages.push(DisplayMessage {
                    message: msg.message.clone(),
                    sent_at: msg.sent_at.clone(),
                    user_name: self.user.as_ref().unwrap().user_name.clone(),
                })
            }
            for msg in response {
                let padding = PaddingScheme::new_pkcs1v15_encrypt();
                let enc_data: Vec<u8> = serde_json::from_str(msg.message.as_str()).unwrap();
                let dec_data = self
                    .private_key
                    .decrypt(padding, &enc_data)
                    .expect("failed to decrpyt");

                let decoded_string = String::from_utf8(dec_data).expect("Failed to decode string");
                recvd_messages.push(DisplayMessage {
                    message: decoded_string,
                    sent_at: msg.sent_at,
                    user_name: self.chatting_with.as_ref().unwrap().user_name.clone(),
                })
            }

            recvd_messages.append(&mut sent_messages);
            recvd_messages.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));
            self.messages = recvd_messages;

            self.chat_history = String::new();
            for message in &mut self.messages {
                let new_string = message.message.clone() + " " + message.sent_at.as_str() + "\n";
                self.chat_history += new_string.as_str();
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::bottom_up(eframe::egui::Align::LEFT), |ui| {
                let response = ui
                    .add_sized([75.0, 35.0], egui::Button::new(String::from("End Session")))
                    .clicked();

                if response {
                    match &self.user {
                        Some(user) => match &self.chatting_with {
                            Some(sender) => {
                                let response = api::delete_messages(
                                    sender.user_id.to_string(),
                                    user.user_id.to_string(),
                                );

                                if response {
                                    self.sent = Vec::new();
                                    self.state = AppState::RenderLobby
                                }
                            }
                            None => todo!(),
                        },
                        None => todo!(),
                    }
                }

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
                        Some(user) => match &self.chatting_with {
                            Some(recipient) => {
                                let message = self.message.clone();
                                let sender = user.user_id.clone();
                                let reciever = recipient.user_id.clone();
                                let pub_key = recipient.public_key.clone();

                                std::thread::spawn(move || {
                                    api::send_message(
                                        &message,
                                        sender.as_str(),
                                        reciever.as_str(),
                                        &pub_key.as_str(),
                                    );
                                });
                            }
                            None => todo!(),
                        },
                        None => {
                            println!("No User");
                        }
                    }

                    let new_sent_message = Message {
                        message: self.message.clone(),
                        sent_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time Error")
                            .as_millis()
                            .to_string(),
                    };

                    let mut new_display_messages: Vec<DisplayMessage> = vec![DisplayMessage {
                        message: self.message.clone(),
                        sent_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time Error")
                            .as_millis()
                            .to_string(),
                        user_name: self.user.as_ref().unwrap().user_name.clone(),
                    }];

                    self.sent.push(new_sent_message);
                    new_display_messages.append(&mut self.messages);
                    self.messages = new_display_messages;
                    self.message.clear();
                }
                text_response.request_focus();

                for msg in &self.messages {
                    ui.heading(format!("[{}] {}", msg.user_name, msg.message));
                }
            })
        });
    }
    fn render_login(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(200.0);
                let login_response = ui.add_sized([200.0, 50.0], egui::Button::new("Login"));
                if login_response.clicked() {
                    if api::set_user_status(
                        self.public_key
                            .to_public_key_pem(LineEnding::CRLF)
                            .unwrap()
                            .as_str(),
                        &mut self.user,
                        true,
                    ) {
                        self.state = AppState::RenderLobby
                    }
                }
                let login_response = ui.add_sized([200.0, 50.0], egui::Button::new("Exit"));
                if login_response.clicked() {
                    exit(0);
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
                            if available_user.user_id == user.user_id {
                                continue;
                            }
                            let response = ui.add_sized(
                                [75.0, 35.0],
                                egui::Button::new(String::from(&available_user.user_name)),
                            );
                            if response.clicked() {
                                self.chatting_with = Some(available_user.clone());
                                match &self.user {
                                    Some(user) => match &self.chatting_with {
                                        Some(sender) => {
                                            let send = self.send_messages.clone();
                                            let recipient_id = user.user_id.clone();
                                            let sender_id = sender.user_id.clone();

                                            std::thread::spawn(move || loop {
                                                let messages = api::get_messages(
                                                    sender_id.clone(),
                                                    recipient_id.clone(),
                                                );
                                                send.send(messages).expect("Whoops!");
                                                thread::sleep(Duration::from_secs(5))
                                            });
                                        }
                                        None => todo!(),
                                    },
                                    None => todo!(),
                                }
                                self.state = AppState::RenderChat;
                            }
                            ui.add_space(20.0)
                        }

                        ui.add_space(100.0);

                        if ui.button("Log Out").clicked() {
                            api::set_user_status("", &mut self.user, false);
                            self.state = AppState::RenderLogin;
                        }

                        if ui.button("Refresh").clicked() {
                            api::get_available_users(&mut self.available_users);
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
        "Downing Camera Co. SecureChat"
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
    let user_id = match env::var_os("CAPSTONE_CHAT_ID") {
        Some(v) => v.into_string().unwrap(),
        None => panic!("$USER is not set"),
    };
    let user_name = match env::var_os("CAPSTONE_CHAT_NAME") {
        Some(v) => v.into_string().unwrap(),
        None => panic!("$USER is not set"),
    };

    let app = SecureChatApp::new(user_id, user_name);
    let mut native_options = eframe::NativeOptions::default();
    native_options.decorated = false;

    eframe::run_native(Box::new(app), native_options);
}
