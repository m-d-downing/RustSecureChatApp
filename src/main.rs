use eframe::{egui, egui::CentralPanel, egui::Context, egui::Layout, epi::App, epi::Frame};
use serde::{Deserialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct User{
    user_id: String,
    user_name: String
}

#[derive(Deserialize)]
struct Users {
    users: Vec<User>,
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
    available_users: Vec<User>,
}


fn get_available_users (available_users: &mut Vec<User>) {
    match reqwest::blocking::get(
        "https://0ibh96tdhk.execute-api.us-west-2.amazonaws.com/users",
    ) {
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

            get_available_users(&mut self.available_users)

            }

            for user in &self.available_users {
                ui.heading(String::from(&user.user_name) + " " + &user.user_id);
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
