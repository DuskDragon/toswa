#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use serde::{Deserialize, Serialize};
//use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};

use eframe::{
    self,
    egui::{
        self, Button, CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, Label,
        Layout, RichText, ScrollArea, Separator, TopBottomPanel, Ui, Vec2, Visuals,
    }, //, Spinner},
    run_native,
    App,
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
pub struct MainWindowConfig {
    pub light_mode: bool,
}

#[derive(Default)]
struct MainWindow {
    pub config: MainWindowConfig,
    text_lines: Vec<TextLineData>,
    main_entry_box: String,
}

struct TextLineData {
    command: String,
    result: String,
}

impl MainWindow {
    pub fn init(mut self, cc: &CreationContext) -> Self {
        if let Some(storage) = cc.storage {
            self.config = eframe::get_value(storage, APP_NAME).unwrap_or_default();
        }
        self.configure_fonts(&cc.egui_ctx);
        self
    }
    pub fn configure_fonts(&self, ctx: &Context) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS".to_string(),
            FontData::from_static(include_bytes!("../assets/MesloLGS_NF_Regular.ttf")),
        );

        font_def
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "MesloLGS".to_string());

        ctx.set_fonts(font_def);
    }
    pub fn render_text_lines(&self, ui: &mut eframe::egui::Ui) {
        for a in &self.text_lines {
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
            let result =
                Label::new(RichText::new(&a.result).text_style(eframe::egui::TextStyle::Button));
            ui.add(result);
            // render seporator
            ui.add_space(PADDING);
            ui.add(Separator::default());
        }
    }
    fn render_header(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading(APP_NAME);
        });
        let sep = Separator::default().spacing(20.);
        ui.add(sep);
    }
    fn render_top_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // define a Top Bottom Panel widget
        TopBottomPanel::top("toswa_top_panel").show(ctx, |ui| {
            //ui.add(PADDING);
            egui::menu::bar(ui, |ui| {
                //logo
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new(
                        RichText::new("🚀").text_style(egui::TextStyle::Heading),
                    ));
                    let file_btn = ui.add(Button::new(
                        RichText::new("File(add garbage)").text_style(egui::TextStyle::Body),
                    ));
                    if file_btn.clicked() {
                        self.text_lines.push(TextLineData {
                            command: "garbage in".to_string(),
                            result: "garbage out".to_string(),
                        });
                    }
                });
                // controls
                ui.with_layout(Layout::right_to_left(), |ui| {
                    if !IS_DECORATED {
                        let close_btn = ui.add(Button::new(
                            RichText::new("❌").text_style(egui::TextStyle::Body),
                        ));
                        if close_btn.clicked() {
                            frame.quit();
                        }
                    }
                    // theme button
                    let theme_btn = ui.add(Button::new(
                        RichText::new({
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
    fn render_bottom_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        TopBottomPanel::bottom("toswa_bottom_panel").show(ctx, |ui| {
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
                    self.text_lines.push(TextLineData {
                        command: self.main_entry_box.clone(),
                        result: "processing...".to_string(),
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
}

impl App for MainWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ////debugging tool
        //ctx.set_debug_on_hover(true);
        //draw window
        if self.config.light_mode {
            ctx.set_visuals(Visuals::light());
        } else {
            ctx.set_visuals(Visuals::dark());
        }
        self.render_top_panel(ctx, frame);
        self.render_bottom_panel(ctx, frame);
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().stick_to_bottom().show(ui, |ui| {
                self.render_header(ui);
                self.render_text_lines(ui);
            })
        });
        ctx.request_repaint();
    }
}

fn main() {
    let toswa = MainWindow::default();

    //icon
    let icon = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("Failed to process icon data")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    let win_options = NativeOptions {
        initial_window_size: Some(Vec2::new(540., 960.)),
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
