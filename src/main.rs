#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Receiver};
use std::thread::{self};

use eframe::{
    self,
    egui::{self, Color32, Ui, Visuals}, //, Spinner},
    run_native,
    CreationContext,
    NativeOptions,
};

const APP_NAME: &str = "toswa";
const IS_DECORATED: bool = true;
const PADDING: f32 = 5.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
//const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
//const RED: Color32 = Color32::from_rgb(255, 0, 0);

#[derive(Serialize, Deserialize, Default)]
pub struct TextCommandWindowConfig {
    pub light_mode: bool,
}

#[derive(Default)]
struct TextCommandWindow {
    pub config: TextCommandWindowConfig,
    text_lines: Vec<TextLineData>,
    main_entry_box: String,
}

struct TextLineData {
    command: String,
    result: String,
    rx_channel: Option<Receiver<String>>,
}

impl TextCommandWindow {
    pub fn init(mut self, cc: &CreationContext) -> Self {
        if let Some(storage) = cc.storage {
            self.config = eframe::get_value(storage, "TextCommandWindow").unwrap_or_default();
        }
        self.configure_fonts(&cc.egui_ctx);
        self
    }
    pub fn configure_fonts(&self, ctx: &egui::Context) {
        let mut font_def = egui::FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS".to_string(),
            egui::FontData::from_static(include_bytes!("../assets/MesloLGS_NF_Regular.ttf")),
        );
        font_def
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "MesloLGS".to_string());

        ctx.set_fonts(font_def);
    }
    pub fn render_text_lines(&mut self, ui: &mut eframe::egui::Ui) {
        for a in self.text_lines.iter() {
            ui.add_space(PADDING);
            // render command
            let command = format!("$ {}", a.command);
            if self.config.light_mode {
                ui.colored_label(BLACK, command);
            } else {
                ui.colored_label(WHITE, command);
            }
            // render result
            ui.add_space(PADDING);
            let result = egui::Label::new(
                egui::RichText::new(a.result.as_str()).text_style(egui::TextStyle::Button),
            );
            ui.add(result);
            // render seporator
            ui.add_space(PADDING);
            ui.add(egui::Separator::default());
        }
    }
    fn render_header(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading(APP_NAME);
        });
        let sep = egui::Separator::default().spacing(20.);
        ui.add(sep);
    }
    fn render_top_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // define a Top Bottom Panel widget
        egui::TopBottomPanel::top("toswa_top_panel").show(ctx, |ui| {
            //ui.add(PADDING);
            egui::menu::bar(ui, |ui| {
                //logo
                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("🚀").text_style(egui::TextStyle::Heading),
                    ));
                    let file_btn = ui.add(egui::Button::new(
                        egui::RichText::new("File(add garbage)").text_style(egui::TextStyle::Body),
                    ));
                    if file_btn.clicked() {
                        self.text_lines.push(TextLineData {
                            command: "garbage in".to_string(),
                            result: "garbage out".to_string(),
                            rx_channel: None,
                        });
                    }
                });
                // controls
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    if !IS_DECORATED {
                        let close_btn = ui.add(egui::Button::new(
                            egui::RichText::new("❌").text_style(egui::TextStyle::Body),
                        ));
                        if close_btn.clicked() {
                            frame.quit();
                        }
                    }
                    // theme button
                    let theme_btn = ui.add(egui::Button::new(
                        egui::RichText::new({
                            if self.config.light_mode {
                                "🌙"
                            } else {
                                "🔆"
                            }
                        })
                        .text_style(egui::TextStyle::Body),
                    ));
                    if theme_btn.clicked() {
                        self.config.light_mode = !self.config.light_mode;
                    }
                });
            });
        });
    }
    fn render_bottom_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("toswa_bottom_panel").show(ctx, |ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.main_entry_box).hint_text("Type something!"),
            );
            //if response.changed() {
            //    //..
            //}
            if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                if !self.main_entry_box.is_empty() {
                    if self.main_entry_box.to_lowercase().eq("exit") {
                        frame.quit();
                        return;
                    }
                    let command = self.main_entry_box.clone();
                    let rx_channel = self.dispatch_comamnd(self.main_entry_box.clone(), ctx);
                    self.text_lines.push(TextLineData {
                        command,
                        result: "processing...".to_string(),
                        rx_channel,
                    });
                    self.main_entry_box.clear();
                }
                response.request_focus();
            }
            if !response.has_focus()
                && ui
                    .input_mut()
                    .consume_key(egui::Modifiers::NONE, egui::Key::I)
            {
                response.request_focus();
            }
            if ui.input_mut().consume_key(
                egui::Modifiers {
                    alt: false,
                    command: true,
                    shift: true,
                    ..Default::default()
                },
                egui::Key::Q,
            ) {
                frame.quit();
                //return
            }
        });
    }
    fn dispatch_comamnd(
        &mut self,
        command: String,
        ctx: &egui::Context,
    ) -> Option<Receiver<String>> {
        let ctx_clone = ctx.clone();
        let (result_tx, result_rx) = channel();
        thread::spawn(move || {
            if let Err(e) = result_tx.send(process_command(command)) {
                eprintln!("Error sending command result: {}", e);
            }
            ctx_clone.request_repaint();
        });
        Some(result_rx)
    }
    fn poll_open_requests(&mut self) {
        for a in self.text_lines.iter_mut() {
            if let Some(rx_channel) = &a.rx_channel {
                match rx_channel.try_recv() {
                    Ok(command_result) => {
                        a.rx_channel = None;
                        a.result = command_result;
                    }
                    Err(_) => continue,
                }
            }
        }
    }
}

fn process_command(command: String) -> String {
    format!("processed: {}", command)
}

impl eframe::App for TextCommandWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ////debugging tool
        //ctx.set_debug_on_hover(true);
        //draw window
        self.poll_open_requests();
        if self.config.light_mode {
            ctx.set_visuals(Visuals::light());
        } else {
            ctx.set_visuals(Visuals::dark());
        }
        self.render_top_panel(ctx, frame);
        self.render_bottom_panel(ctx, frame);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom()
                .show(ui, |ui| {
                    self.render_header(ui);
                    self.render_text_lines(ui);
                })
        });
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "TextCommandWindow", &self.config);
    }
}

fn main() {
    let toswa = TextCommandWindow::default();
    //icon
    let icon = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("Failed to process icon data")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    let win_options = NativeOptions {
        initial_window_size: Some(egui::Vec2::new(540., 960.)),
        decorated: IS_DECORATED,
        icon_data: Some(eframe::IconData {
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }),
        ..Default::default()
    };
    run_native(
        APP_NAME,
        win_options,
        Box::new(|cc| Box::new(toswa.init(cc))),
    );
}
