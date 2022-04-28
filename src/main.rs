use eframe::{egui, egui::CentralPanel, egui::Context, egui::Layout, epi::App, epi::Frame};
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Users {
    users: Vec<String>,
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

#[derive(Default)]
struct SecureChatApp {
    message: String,
    chat_history: String,
    state: AppState,
    user: String,
    login_name: String,
    available_users: Vec<String>,
}

impl SecureChatApp {
    fn render_chat(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::bottom_up(eframe::egui::Align::LEFT), |ui| {
                let text_response = ui.add(
                    egui::TextEdit::singleline(&mut self.message)
                        .desired_width(600.0)
                        .lock_focus(true),
                );
                if text_response.lost_focus() {
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
            ui.heading("Login");
            ui.add(
                egui::TextEdit::singleline(&mut self.login_name)
                    .desired_width(600.0)
                    .lock_focus(true),
            );
            if ui.button("Submit").clicked() {
                let mut map = HashMap::new();
                map.insert("lang", "rust");
                map.insert("body", "json");

                match reqwest::blocking::get(
                    "https://0ibh96tdhk.execute-api.us-west-2.amazonaws.com/users",
                ) {
                    Ok(mut response) => {
                        if response.status() == reqwest::StatusCode::OK {
                            match response.json::<Users>() {
                                Ok(users) => {
                                    self.available_users = users.users;
                                }
                                Err(e) => println!("{:?}", e),
                            }
                        }
                    }
                    Err(_) => println!("Could not make request"),
                }
            }

            for user in &self.available_users {
                ui.heading(user);
            }
        });
    }
    fn render_lobby(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading(String::from("User: ") + &self.user);
            ui.heading(&self.user);
            if ui.button("Go To Chat").clicked() {
                self.state = AppState::RenderChat
            }
        });
    }
}

impl App for SecureChatApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &Context, frame: &Frame) {
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
