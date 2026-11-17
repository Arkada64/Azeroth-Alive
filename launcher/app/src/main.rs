// SoloWoW launcher (desktop): one window to start the world, play, watch it live, and stop it.
#![windows_subsystem = "windows"]

use eframe::egui;
use solowow_core as core;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Default)]
struct Shared {
    status: core::Status,
    busy: Option<String>,        // what we're doing right now ("Starting the world...")
    message: String,             // the last result or error
    sim: String,                 // .sim status line
    chronicle: Vec<String>,
    we_opened_client: bool,
    client_seen: bool,
}

struct App {
    shared: Arc<Mutex<Shared>>,
    settings: core::Settings,
    saved: core::Settings,
    confirm: Option<bool>,       // asking before a fresh start (false) or bringing back the previous world (true)
    graphics: Option<usize>,     // the client's graphics preset (None = its own mix)
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let shared = Arc::new(Mutex::new(Shared::default()));
        let settings = core::load_settings();
        let app = App { shared: shared.clone(), saved: settings.clone(), settings, confirm: None, graphics: core::graphics_preset() };
        // background: watch the servers and the client, follow the chronicle, ask the world how it's doing
        let s = shared.clone();
        let ctx = cc.egui_ctx.clone();
        thread::spawn(move || {
            let mut last_sim = Instant::now() - Duration::from_secs(60);
            loop {
                let st = core::status();
                let chron = core::chronicle_tail(40);
                let sim = if st.world_up() && last_sim.elapsed() > Duration::from_secs(15) {
                    last_sim = Instant::now();
                    core::soap(".sim status").ok()
                } else { None };
                let stop_now = {
                    let mut g = s.lock().unwrap();
                    g.status = st;
                    g.chronicle = chron;
                    if let Some(line) = sim { g.sim = line; }
                    if !st.world_up() { g.sim.clear(); }
                    // the player closed WoW: stop the world too (if they want that and we opened it)
                    let closed = g.client_seen && !st.client && g.we_opened_client && g.busy.is_none();
                    if st.client { g.client_seen = true; }
                    if closed { g.client_seen = false; g.we_opened_client = false; }
                    closed && core::load_settings().stop_with_client
                };
                if stop_now {
                    s.lock().unwrap().busy = Some("WoW closed - stopping the world...".into());
                    ctx.request_repaint();
                    let r = core::stop_world();
                    let mut g = s.lock().unwrap();
                    g.busy = None;
                    g.message = match r { Ok(()) => "The world is stopped. It catches up on the time it missed when you come back.".into(), Err(e) => e };
                }
                ctx.request_repaint();
                thread::sleep(Duration::from_secs(2));
            }
        });
        app
    }

    fn run(&self, what: &str, ctx: &egui::Context, job: impl FnOnce() -> Result<String, String> + Send + 'static) {
        let s = self.shared.clone();
        {
            let mut g = s.lock().unwrap();
            if g.busy.is_some() { return; }
            g.busy = Some(what.to_string());
        }
        let ctx = ctx.clone();
        thread::spawn(move || {
            let r = job();
            let mut g = s.lock().unwrap();
            g.busy = None;
            g.message = match r { Ok(m) => m, Err(e) => format!("Problem: {e}") };
            ctx.request_repaint();
        });
    }
}

fn light(ui: &mut egui::Ui, on: bool, label: &str) {
    let color = if on { egui::Color32::from_rgb(90, 200, 110) } else { egui::Color32::from_rgb(110, 110, 110) };
    ui.label(egui::RichText::new("●").color(color));
    ui.label(label);
    ui.add_space(10.0);
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (st, busy, message, sim, chronicle) = {
            let g = self.shared.lock().unwrap();
            (g.status, g.busy.clone(), g.message.clone(), g.sim.clone(), g.chronicle.clone())
        };
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("SoloWoW");
                ui.add_space(16.0);
                light(ui, st.database, "Database");
                light(ui, st.login, "Login");
                light(ui, st.world, "World");
                light(ui, st.client, "WoW");
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let idle = busy.is_none();
                let play = ui.add_enabled(idle && !st.client, egui::Button::new(egui::RichText::new("  Play  ").size(20.0)));
                if play.clicked() {
                    let s = self.shared.clone();
                    self.run("Starting the world...", ctx, move || {
                        if !core::status().world_up() { core::start_world()?; }
                        core::launch_client()?;
                        s.lock().unwrap().we_opened_client = true;
                        Ok("Have fun. The world keeps going while you play.".into())
                    });
                }
                if ui.add_enabled(idle && !st.world_up(), egui::Button::new("Start the world only")).clicked() {
                    self.run("Starting the world...", ctx, core::start_world);
                }
                if ui.add_enabled(idle && (st.world || st.login), egui::Button::new("Stop the world")).clicked() {
                    self.run("Stopping the world...", ctx, || core::stop_world().map(|_| "The world is stopped.".to_string()));
                }
            });
            ui.add_space(4.0);
            if let Some(b) = &busy { ui.horizontal(|ui| { ui.spinner(); ui.label(b); }); }
            else if !message.is_empty() { ui.label(&message); }
            ui.add_space(6.0);
        });

        egui::TopBottomPanel::bottom("settings").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.collapsing("Settings", |ui| {
                ui.add(egui::Slider::new(&mut self.settings.wanderers, 0..=300).text("adventurers around you"));
                ui.add(egui::Slider::new(&mut self.settings.townsfolk, 0..=200).text("townsfolk around you"));
                ui.checkbox(&mut self.settings.stop_with_client, "Stop the world when I close WoW");
                ui.checkbox(&mut self.settings.era, "Era preset").on_hover_text(
                    "Slower levelling (half XP), no dungeon-finder teleports, no quest areas on the map, riding at vanilla/TBC levels and costs. Off = stock 3.3.5.");
                ui.checkbox(&mut self.settings.unload_grids, "Let go of map areas nobody is near").on_hover_text(
                    "Keeps memory flat on long sessions. Turn it off if the world ever crashes right after you leave an area.");
                ui.horizontal(|ui| {
                    ui.label("Graphics:");
                    let current = self.graphics.map(|i| core::GRAPHICS[i]).unwrap_or("your own settings");
                    let mut pick: Option<usize> = None;
                    ui.add_enabled_ui(!st.client, |ui| {
                        egui::ComboBox::from_id_salt("graphics").selected_text(current).show_ui(ui, |ui| {
                            for (i, name) in core::GRAPHICS.iter().enumerate() {
                                if ui.selectable_label(self.graphics == Some(i), *name).clicked() { pick = Some(i); }
                            }
                        });
                    });
                    if st.client { ui.label(egui::RichText::new("(close WoW to change)").weak()); }
                    if let Some(i) = pick {
                        let r = core::set_graphics_preset(i);
                        let mut g = self.shared.lock().unwrap();
                        g.message = match r { Ok(()) => { self.graphics = Some(i); format!("Graphics set to {} - it applies the next time WoW starts.", core::GRAPHICS[i]) } Err(e) => format!("Problem: {e}") };
                    }
                });
                ui.horizontal(|ui| {
                    if ui.add_enabled(self.settings != self.saved && busy.is_none(), egui::Button::new("Save")).clicked() {
                        let s = self.settings.clone();
                        let era_changed = s.era != self.saved.era;
                        self.saved = s.clone();
                        let up = st.world_up();
                        self.run(if era_changed { "Saving (and switching the Era preset)..." } else { "Saving..." }, ctx, move || {
                            core::save_settings(&s)?;
                            Ok(if up { "Saved. Stop and start the world for it to apply.".into() } else { "Saved. It applies the next time the world starts.".into() })
                        });
                    }
                    if core::find_client().is_none() { ui.colored_label(egui::Color32::LIGHT_RED, "No WoW client found in C:\\SoloWoW\\WOTLK Client"); }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let idle = busy.is_none() && self.confirm.is_none();
                    if ui.add_enabled(idle, egui::Button::new("Start fresh...")).on_hover_text("A brand-new living world. Your characters stay.").clicked() {
                        self.confirm = Some(false);
                    }
                    if core::has_previous_world() && ui.add_enabled(idle, egui::Button::new("Bring back the previous world...")).clicked() {
                        self.confirm = Some(true);
                    }
                });
            });
            ui.add_space(4.0);
        });

        if let Some(undo) = self.confirm {
            egui::Window::new(if undo { "Bring back the previous world?" } else { "Start fresh?" })
                .collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if undo {
                        ui.label("The world you have now is thrown away, and the one you set aside comes back as it was.");
                    } else {
                        ui.label("Everyone in the living world is replaced: new adventurers, new townsfolk, new bounties and requests.");
                        ui.label("The new world lives a fortnight on its own before you arrive, so it has some history.");
                        ui.add_space(4.0);
                        ui.label("Your characters, their bags, gold and mail stay as they are.");
                        ui.label("The current world is kept aside once, so you can still bring it back (until the next fresh start).");
                    }
                    if st.world_up() || st.client { ui.colored_label(egui::Color32::LIGHT_YELLOW, "The world will be stopped first - you'll be disconnected if you're playing."); }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let go = ui.button(if undo { "Bring it back" } else { "Start fresh" }).clicked();
                        if ui.button("Cancel").clicked() { self.confirm = None; }
                        if go {
                            self.confirm = None;
                            self.run(if undo { "Bringing back the previous world..." } else { "Setting the old world aside..." }, ctx, move || core::reset_world(undo));
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("The world");
            if !sim.is_empty() { ui.label(egui::RichText::new(&sim).weak()); }
            else if !st.world_up() { ui.label(egui::RichText::new("The world is asleep. Start it and the chronicle carries on from where it stopped.").weak()); }
            ui.add_space(6.0);
            egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                for line in &chronicle {
                    if line.starts_with("=== Day") { ui.add_space(4.0); ui.label(egui::RichText::new(line.trim_matches(|c| c == '=' || c == ' ')).strong()); }
                    else if line.starts_with("-- ") { ui.label(egui::RichText::new(&line[3..]).weak()); }
                    else { ui.label(line); }
                }
            });
        });
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

/// The window icon, drawn here (no image files to ship): a gold coin with a dark keyhole-like tree in it.
fn icon() -> egui::IconData {
    const N: usize = 64;
    let mut rgba = vec![0u8; N * N * 4];
    for y in 0..N {
        for x in 0..N {
            let (fx, fy) = (x as f32 - 31.5, y as f32 - 31.5);
            let d = (fx * fx + fy * fy).sqrt();
            let i = (y * N + x) * 4;
            if d > 30.5 { continue; }
            let rim = d > 26.0;
            // the tree: a trunk and a round crown
            let trunk = fx.abs() < 3.5 && fy > 2.0 && fy < 20.0;
            let crown = (fx * fx + (fy + 6.0) * (fy + 6.0)).sqrt() < 13.0;
            let (r, g, b) = if rim { (150, 105, 30) } else if trunk || crown { (40, 30, 20) } else { (222, 170, 60) };
            let a = if d > 29.5 { ((30.5 - d) * 255.0) as u8 } else { 255 };
            rgba[i..i + 4].copy_from_slice(&[r, g, b, a]);
        }
    }
    egui::IconData { rgba, width: N as u32, height: N as u32 }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([680.0, 600.0]).with_title("SoloWoW").with_icon(icon()),
        ..Default::default()
    };
    eframe::run_native("SoloWoW", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
