use eframe::{egui, egui::CentralPanel, egui::Context, egui::Layout, epi::App, epi::Frame};

#[derive(Default)]
struct SecureChatApp {
    message: String,
    chat_history: String,
}

impl App for SecureChatApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &Context, frame: &Frame) {
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
}

fn main() {
    let app = SecureChatApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
