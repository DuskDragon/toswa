use serde::{Serialize, Deserialize};
//use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};

use eframe::{
    egui::{CentralPanel, Context, FontData, FontDefinitions, FontFamily, Visuals}, //, ScrollArea, Spinner, Visuals},
    App, run_native, NativeOptions, CreationContext, 
};

const APP_NAME: &str = "toswa";

#[derive(Serialize, Deserialize)]
#[derive(Default)]
pub struct MainWindowConfig {
    pub dark_mode: bool,
}

/*
impl Default for MainWindowConfig {
    fn default() -> Self {
        Self {
            dark_mode: Default::default(),
        }
    }
}*/

#[derive(Default)]
struct MainWindow {
    pub config: MainWindowConfig,
}

impl MainWindow{
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
}

impl App for MainWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        if self.config.dark_mode {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello World");
        });        
    }
}

fn main() {
    let toswa = MainWindow::default();
    let win_option = NativeOptions::default();
    run_native(
        "Toswa", 
        win_option, 
        Box::new(|cc| Box::new(toswa.init(cc)))
    );
}
