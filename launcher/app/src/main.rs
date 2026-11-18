// SoloWoW launcher (desktop): the one window for everything - play, the servers (start, stop, restart, RAM, uptime,
// logs, the world's console), accounts, living worlds (new, save a copy, load), and settings. Nov 18: replaces the
// PowerShell manager; nothing here runs PowerShell (but the Era switch, a database script).
#![windows_subsystem = "windows"]

use eframe::egui;
use solowow_core as core;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Default)]
struct Shared {
    status: core::Status,
    ready: bool,                 // the world lets players in (not still making or catching up the living world)
    players: Option<u32>,
    busy: Option<String>,        // what we're doing right now ("Starting the world...")
    message: String,             // the last result or error
    sim: String,                 // .sim status line
    chronicle: Vec<String>,
    console: Vec<String>,        // the world console's answers
    accounts: Vec<core::Account>,
    worlds: Vec<core::SavedWorld>,
    refresh_lists: bool,
    we_opened_client: bool,
    client_seen: bool,
    ask_force: bool,             // the world didn't shut down in time: force it?
}

#[derive(PartialEq, Clone, Copy)]
enum Tab { Play, Server, Accounts, Worlds, Settings }

#[derive(Clone)]
enum Confirm { NewWorld, LoadWorld(String), DeleteWorld(String), DeleteAccount(String), Close }

struct App {
    shared: Arc<Mutex<Shared>>,
    tab: Tab,
    settings: core::Settings,
    saved: core::Settings,
    graphics: Option<usize>,
    confirm: Option<Confirm>,
    log_pick: usize,             // 0 world, 1 login, 2 errors
    command: String,
    new_name: String, new_pass: String, new_pass2: String, new_gm: bool,
    pass_for: Option<String>, pass_new: String,
    world_name: String,
    closing_ok: bool,
}

const LOGS: [(&str, &str); 3] = [("World", "Server.log"), ("Login", "Auth.log"), ("Errors", "Errors.log")];

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let shared = Arc::new(Mutex::new(Shared { refresh_lists: true, ..Default::default() }));
        let settings = core::load_settings();
        let app = App {
            shared: shared.clone(), tab: Tab::Play, saved: settings.clone(), settings, graphics: core::graphics_preset(), confirm: None,
            log_pick: 0, command: String::new(), new_name: String::new(), new_pass: String::new(), new_pass2: String::new(), new_gm: false,
            pass_for: None, pass_new: String::new(), world_name: String::new(), closing_ok: false,
        };
        // background: watch the servers and the client, follow the chronicle and the logs, ask the world how it's doing
        let s = shared.clone();
        let ctx = cc.egui_ctx.clone();
        thread::spawn(move || {
            let mut last_sim = Instant::now() - Duration::from_secs(60);
            loop {
                let st = core::status();
                let ready = st.world.open && core::world_ready();
                let (sim, players) = if ready && last_sim.elapsed() > Duration::from_secs(15) {
                    last_sim = Instant::now();
                    (core::soap(".sim status").ok(), core::players_online())
                } else { (None, None) };
                let chron = core::chronicle_tail(60);
                let refresh = { let mut g = s.lock().unwrap(); std::mem::take(&mut g.refresh_lists) };
                let lists = if refresh && st.database.open { Some((core::accounts().unwrap_or_default(), core::saved_worlds())) } else if refresh { Some((Vec::new(), core::saved_worlds())) } else { None };
                let stop_now = {
                    let mut g = s.lock().unwrap();
                    g.status = st;
                    g.ready = ready;
                    g.chronicle = chron;
                    if let Some(line) = sim { g.sim = line; }
                    if players.is_some() { g.players = players; }
                    if !st.world.open { g.sim.clear(); g.players = None; }
                    if let Some((a, w)) = lists { if !a.is_empty() || !st.database.open { g.accounts = a; } g.worlds = w; }
                    // the player closed WoW: stop the world too (if they want that and we opened it)
                    let closed = g.client_seen && !st.client && g.we_opened_client && g.busy.is_none();
                    if st.client { g.client_seen = true; }
                    if closed { g.client_seen = false; g.we_opened_client = false; }
                    closed && core::load_settings().stop_with_client
                };
                if stop_now {
                    s.lock().unwrap().busy = Some("WoW closed - saving and stopping the world...".into());
                    ctx.request_repaint();
                    let r = core::stop_all(core::load_settings().stop_database_too);
                    let mut g = s.lock().unwrap();
                    g.busy = None;
                    match r {
                        Ok(()) => g.message = "Everything is stopped. The world catches up on the time it missed when you come back.".into(),
                        Err(e) => { g.message = format!("Problem: {e}"); g.ask_force = e.contains("4 minutes"); }
                    }
                }
                ctx.request_repaint();
                thread::sleep(Duration::from_secs(2));
            }
        });
        app
    }

    /// A job on its own thread; `say` updates the busy line while it runs.
    fn run(&self, what: &str, ctx: &egui::Context, job: impl FnOnce(&dyn Fn(String)) -> Result<String, String> + Send + 'static) {
        let s = self.shared.clone();
        {
            let mut g = s.lock().unwrap();
            if g.busy.is_some() { return; }
            g.busy = Some(what.to_string());
        }
        let ctx = ctx.clone();
        thread::spawn(move || {
            let s2 = s.clone();
            let c2 = ctx.clone();
            let say = move |t: String| { s2.lock().unwrap().busy = Some(t); c2.request_repaint(); };
            let r = job(&say);
            let mut g = s.lock().unwrap();
            g.busy = None;
            g.refresh_lists = true;
            match r {
                Ok(m) => g.message = m,
                Err(e) => { g.ask_force = e.contains("4 minutes"); g.message = format!("Problem: {e}"); }
            }
            ctx.request_repaint();
        });
    }

    fn say(&self, text: impl Into<String>) { self.shared.lock().unwrap().message = text.into(); }
}

/// Starts whatever isn't running and waits until the world lets players in - a new world lives a fortnight first and a
/// restart catches up on the time it was away; a client that connects before then sits at "Connected" (Nov 18).
fn bring_world_up(say: &dyn Fn(String)) -> Result<(), String> {
    say("Starting the database...".into());
    core::start_database()?;
    say("Starting the login server...".into());
    core::start_login()?;
    say("Waking the world...".into());
    core::start_world_server()?;
    let t0 = Instant::now();
    while !core::world_ready() {
        if t0.elapsed() > Duration::from_secs(25 * 60) { return Err("the world took more than 25 minutes to get ready - see the World log".into()); }
        if !core::status().world.running { return Err("the world server stopped while getting ready - see the World log".into()); }
        let p = core::world_progress();
        say(format!("The world is getting ready ({} s){}", t0.elapsed().as_secs(), if p.is_empty() { String::new() } else { format!(" - {}", trim(&p, 90)) }));
        thread::sleep(Duration::from_secs(2));
    }
    Ok(())
}

fn trim(s: &str, n: usize) -> String { if s.chars().count() > n { s.chars().take(n).collect::<String>() + "..." } else { s.to_string() } }

fn light(ui: &mut egui::Ui, on: bool, warm: bool, label: &str) {
    let color = if on { egui::Color32::from_rgb(90, 200, 110) } else if warm { egui::Color32::from_rgb(230, 170, 60) } else { egui::Color32::from_rgb(110, 110, 110) };
    ui.label(egui::RichText::new("●").color(color));
    ui.label(label);
    ui.add_space(10.0);
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (st, ready, players, busy, message, sim, chronicle, console, accounts, worlds, ask_force) = {
            let g = self.shared.lock().unwrap();
            (g.status, g.ready, g.players, g.busy.clone(), g.message.clone(), g.sim.clone(), g.chronicle.clone(), g.console.clone(),
             g.accounts.clone(), g.worlds.clone(), g.ask_force)
        };
        let idle = busy.is_none();

        // closing the window with servers running: ask
        if ctx.input(|i| i.viewport().close_requested()) && !self.closing_ok && st.anything_running() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm = Some(Confirm::Close);
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("SoloWoW");
                ui.add_space(16.0);
                light(ui, st.database.open, st.database.running, "Database");
                light(ui, st.login.open, st.login.running, "Login");
                light(ui, ready, st.world.running, if st.world.open && !ready { "World (getting ready)" } else { "World" });
                light(ui, st.client, false, "WoW");
                if let Some(n) = players { ui.label(format!("{n} online")); }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for (t, name) in [(Tab::Play, "Play"), (Tab::Server, "Server"), (Tab::Accounts, "Accounts"), (Tab::Worlds, "Worlds"), (Tab::Settings, "Settings")] {
                    ui.selectable_value(&mut self.tab, t, name);
                }
            });
            ui.add_space(2.0);
            if let Some(b) = &busy { ui.horizontal(|ui| { ui.spinner(); ui.label(b); }); }
            else if !message.is_empty() { ui.label(&message); }
            if ask_force && idle {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::LIGHT_YELLOW, "The world is still saving or stuck.");
                    if ui.button("Force it to stop").on_hover_text("It loses what happened since its last save (every 10 minutes).").clicked() {
                        self.shared.lock().unwrap().ask_force = false;
                        self.run("Forcing the world to stop...", ctx, |_| { core::force_stop_world(); core::stop_login()?; Ok("The world is stopped (forced).".into()) });
                    }
                    if ui.button("Keep waiting").clicked() { self.shared.lock().unwrap().ask_force = false; }
                });
            }
            ui.add_space(6.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Play => self.play_tab(ui, ctx, &st, ready, idle, &sim, &chronicle, &accounts),
            Tab::Server => self.server_tab(ui, ctx, &st, ready, idle, &console),
            Tab::Accounts => self.accounts_tab(ui, ctx, &st, idle, &accounts),
            Tab::Worlds => self.worlds_tab(ui, ctx, &st, idle, &sim, &worlds),
            Tab::Settings => self.settings_tab(ui, ctx, &st, idle),
        });

        if let Some(c) = self.confirm.clone() { self.confirm_window(ctx, c, &st); }
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

impl App {
    #[allow(clippy::too_many_arguments)]
    fn play_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, st: &core::Status, ready: bool, idle: bool, sim: &str, chronicle: &[String], accounts: &[core::Account]) {
        // every account can play - the first characters were made on the launcher's own (SOLO)
        let mine: Vec<&core::Account> = accounts.iter().collect();
        if self.settings.account.is_empty() {
            if let Some(a) = accounts.iter().max_by_key(|a| a.characters).filter(|a| a.characters > 0) {
                self.settings.account = a.name.clone(); self.saved.account = a.name.clone(); let _ = core::save_settings(&self.saved);
            }
        }
        ui.horizontal(|ui| {
            ui.label("Account:");
            let current = if self.settings.account.is_empty() { "choose...".to_string() } else { self.settings.account.clone() };
            let mut pick = None;
            egui::ComboBox::from_id_salt("acct").selected_text(current).show_ui(ui, |ui| {
                for a in &mine {
                    let label = format!("{} ({} characters)", a.name, a.characters);
                    if ui.selectable_label(self.settings.account.eq_ignore_ascii_case(&a.name), label).clicked() { pick = Some(a.name.clone()); }
                }
            });
            if let Some(n) = pick { self.settings.account = n; self.saved.account = self.settings.account.clone(); let _ = core::save_settings(&self.saved); }
            if mine.len() <= 1 && st.database.open { ui.label(egui::RichText::new("Make an account of your own under Accounts.").weak()); }
            if mine.is_empty() && st.database.open { ui.colored_label(egui::Color32::LIGHT_YELLOW, "No account yet - make one under Accounts."); }
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let can_play = idle && !st.client && !self.settings.account.is_empty();
            if ui.add_enabled(can_play, egui::Button::new(egui::RichText::new("  Play  ").size(22.0))).clicked() {
                let s = self.shared.clone();
                let account = self.settings.account.clone();
                self.run("Starting...", ctx, move |say| {
                    bring_world_up(say)?;
                    say("Opening WoW...".into());
                    core::launch_client(&account)?;
                    s.lock().unwrap().we_opened_client = true;
                    Ok(format!("Have fun. Log in as {} - the world keeps going while you play.", account.to_uppercase()))
                });
            }
            if ui.add_enabled(idle && !ready, egui::Button::new("Start the world only")).clicked() {
                self.run("Starting...", ctx, |say| { bring_world_up(say)?; Ok("The world is up and lets players in.".into()) });
            }
            if ui.add_enabled(idle && st.anything_running(), egui::Button::new("Stop everything")).clicked() {
                let db = self.settings.stop_database_too;
                self.run("Saving and stopping the world...", ctx, move |_| { core::stop_all(db)?; Ok("Everything is stopped.".into()) });
            }
        });
        ui.add_space(10.0);
        ui.heading("The world");
        if !sim.is_empty() { ui.label(egui::RichText::new(sim).weak()); }
        else if st.world.open && !ready { ui.label(egui::RichText::new("Getting the living world ready - you can connect once the World light is green.").weak()); }
        else if !st.world.open { ui.label(egui::RichText::new("The world is asleep. Start it and the chronicle carries on from where it stopped.").weak()); }
        ui.add_space(6.0);
        egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
            for line in chronicle {
                if line.starts_with("=== Day") { ui.add_space(4.0); ui.label(egui::RichText::new(line.trim_matches(|c| c == '=' || c == ' ')).strong()); }
                else if let Some(rest) = line.strip_prefix("-- ") { ui.label(egui::RichText::new(rest).weak()); }
                else { ui.label(line); }
            }
        });
    }

    fn server_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, st: &core::Status, ready: bool, idle: bool, console: &[String]) {
        egui::Grid::new("procs").num_columns(6).spacing([14.0, 6.0]).striped(true).show(ui, |ui| {
            ui.strong("Server"); ui.strong("Status"); ui.strong("RAM"); ui.strong("Up"); ui.strong(""); ui.end_row();
            let rows: [(&str, core::Proc, &str, bool); 3] = [("Database", st.database, "db", true), ("Login server", st.login, "login", true), ("World server", st.world, "world", ready)];
            for (name, p, key, ok) in rows {
                ui.label(name);
                let (text, color) = if p.open && ok { ("Running", egui::Color32::from_rgb(90, 200, 110)) }
                    else if p.open { ("Getting ready", egui::Color32::from_rgb(230, 170, 60)) }
                    else if p.running { ("Starting...", egui::Color32::from_rgb(230, 170, 60)) }
                    else { ("Stopped", egui::Color32::from_rgb(200, 90, 90)) };
                ui.colored_label(color, text);
                ui.label(if p.running && p.mem_kb > 0 { core::fmt_mem(p.mem_kb) } else { String::new() });
                ui.label(p.uptime.map(core::fmt_uptime).unwrap_or_default());
                ui.horizontal(|ui| {
                    let start = ui.add_enabled(idle && !p.running, egui::Button::new("Start")).clicked();
                    let stop = ui.add_enabled(idle && p.running, egui::Button::new("Stop")).clicked();
                    let restart = ui.add_enabled(idle && p.running, egui::Button::new("Restart")).clicked();
                    let key = key.to_string();
                    if start || stop || restart {
                        self.run(&format!("{} the {}...", if start { "Starting" } else if stop { "Stopping" } else { "Restarting" }, name.to_lowercase()), ctx, move |say| {
                            let up = |say: &dyn Fn(String)| -> Result<(), String> {
                                match key.as_str() { "db" => core::start_database(), "login" => core::start_login(), _ => bring_world_up(say) }
                            };
                            let down = || -> Result<(), String> {
                                match key.as_str() { "db" => core::stop_database(), "login" => core::stop_login(), _ => core::stop_world_server() }
                            };
                            if stop || restart { down()?; }
                            if start || restart { up(say)?; }
                            Ok("Done.".into())
                        });
                    }
                });
                ui.end_row();
            }
        });
        ui.horizontal(|ui| {
            if ui.add_enabled(idle && !ready, egui::Button::new("Start all")).clicked() {
                self.run("Starting...", ctx, |say| { bring_world_up(say)?; Ok("Everything is running.".into()) });
            }
            if ui.add_enabled(idle && st.anything_running(), egui::Button::new("Stop all")).clicked() {
                self.run("Saving and stopping...", ctx, |_| { core::stop_all(true)?; Ok("Everything is stopped.".into()) });
            }
            if ui.add_enabled(idle && st.world.open, egui::Button::new("Save now")).clicked() {
                self.run("Saving...", ctx, |_| { core::soap("saveall")?; core::soap(".sim save").ok(); Ok("Saved.".into()) });
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Log:");
            for (i, (name, _)) in LOGS.iter().enumerate() { ui.selectable_value(&mut self.log_pick, i, *name); }
        });
        let lines = core::log_tail(LOGS[self.log_pick].1, 400);
        let console_h = 150.0;
        egui::ScrollArea::vertical().id_salt("log").stick_to_bottom(true).auto_shrink([false, false]).max_height(ui.available_height() - console_h - 40.0).show(ui, |ui| {
            for l in &lines { ui.label(egui::RichText::new(l).monospace().size(11.5)); }
        });
        ui.separator();
        ui.label("World console");
        egui::ScrollArea::vertical().id_salt("console").stick_to_bottom(true).auto_shrink([false, false]).max_height(console_h - 30.0).show(ui, |ui| {
            for l in console { ui.label(egui::RichText::new(l).monospace().size(11.5)); }
        });
        ui.horizontal(|ui| {
            let edit = ui.add(egui::TextEdit::singleline(&mut self.command).hint_text("a server command, e.g. .sim status").desired_width(ui.available_width() - 330.0));
            let enter = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let mut send = |cmd: String, me: &mut App| {
                if cmd.trim().is_empty() { return; }
                let s = me.shared.clone();
                s.lock().unwrap().console.push(format!("> {cmd}"));
                let ctx = ctx.clone();
                thread::spawn(move || {
                    let r = core::soap(&cmd).unwrap_or_else(|e| format!("[{e}]"));
                    let mut g = s.lock().unwrap();
                    for l in r.lines() { g.console.push(l.to_string()); }
                    let n = g.console.len(); if n > 500 { g.console.drain(0..n - 500); }
                    ctx.request_repaint();
                });
            };
            if (ui.add_enabled(st.world.open, egui::Button::new("Send")).clicked() || enter) && st.world.open {
                let c = std::mem::take(&mut self.command); send(c, self); edit.request_focus();
            }
            for (label, cmd) in [("Living world", ".sim status"), ("Server info", "server info"), ("Who's on", "account onlinelist")] {
                if ui.add_enabled(st.world.open, egui::Button::new(label)).clicked() { send(cmd.to_string(), self); }
            }
        });
    }

    fn accounts_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, st: &core::Status, idle: bool, accounts: &[core::Account]) {
        if !st.database.open {
            ui.label("The database is asleep.");
            if ui.add_enabled(idle, egui::Button::new("Start the database")).clicked() {
                self.run("Starting the database...", ctx, |_| { core::start_database()?; Ok("The database is up.".into()) });
            }
            return;
        }
        ui.heading("New account");
        ui.horizontal(|ui| {
            ui.label("Name"); ui.add(egui::TextEdit::singleline(&mut self.new_name).desired_width(130.0));
            ui.label("Password"); ui.add(egui::TextEdit::singleline(&mut self.new_pass).password(true).desired_width(120.0));
            ui.label("Again"); ui.add(egui::TextEdit::singleline(&mut self.new_pass2).password(true).desired_width(120.0));
            ui.checkbox(&mut self.new_gm, "Game master").on_hover_text("Full GM commands in game (.gm on, .tele...). Off for an ordinary player.");
            if ui.add_enabled(idle && !self.new_name.is_empty() && !self.new_pass.is_empty(), egui::Button::new("Create")).clicked() {
                if self.new_pass != self.new_pass2 { self.say("Problem: the two passwords don't match."); }
                else {
                    let (n, p, gm) = (self.new_name.clone(), self.new_pass.clone(), if self.new_gm { 3 } else { 0 });
                    let first = accounts.iter().all(|a| a.name.eq_ignore_ascii_case(core::SERVICE_ACCOUNT));
                    if first || self.settings.account.is_empty() { self.settings.account = n.to_uppercase(); self.saved.account = self.settings.account.clone(); let _ = core::save_settings(&self.saved); }
                    self.new_pass.clear(); self.new_pass2.clear(); self.new_name.clear();
                    self.run("Creating the account...", ctx, move |_| core::create_account(&n, &p, gm));
                }
            }
        });
        ui.label(egui::RichText::new("Names are letters and digits; passwords up to 16 characters (what the 3.3.5 client allows).").weak());
        ui.add_space(10.0);
        ui.heading("Accounts");
        egui::Grid::new("accts").num_columns(6).spacing([14.0, 6.0]).striped(true).show(ui, |ui| {
            ui.strong("Name"); ui.strong("Rights"); ui.strong("Characters"); ui.strong("Last login"); ui.strong(""); ui.end_row();
            for a in accounts {
                let service = a.name.eq_ignore_ascii_case(core::SERVICE_ACCOUNT);
                ui.label(if a.online { format!("{} (online)", a.name) } else { a.name.clone() });
                if service { ui.label(egui::RichText::new("the launcher's own").weak()); }
                else {
                    let mut gm = a.gm;
                    egui::ComboBox::from_id_salt(format!("gm{}", a.id)).width(110.0).selected_text(match gm { 0 => "Player", 1 => "Moderator", 2 => "Game master", _ => "Administrator" }).show_ui(ui, |ui| {
                        for (lvl, n) in [(0u8, "Player"), (1, "Moderator"), (2, "Game master"), (3, "Administrator")] { ui.selectable_value(&mut gm, lvl, n); }
                    });
                    if gm != a.gm { let n = a.name.clone(); self.run("Changing rights...", ctx, move |_| core::set_gm(&n, gm)); }
                }
                ui.label(a.characters.to_string());
                ui.label(if a.last_login.is_empty() { "never".to_string() } else { a.last_login.clone() });
                ui.horizontal(|ui| {
                    if service {
                        if ui.add_enabled(idle, egui::Button::new("Play as")).clicked() { self.settings.account = a.name.clone(); self.saved.account = a.name.clone(); let _ = core::save_settings(&self.saved); self.say(format!("Play logs in as {} (password solo).", a.name)); }
                    } else {
                        if ui.add_enabled(idle, egui::Button::new("Play as")).clicked() { self.settings.account = a.name.clone(); self.saved.account = a.name.clone(); let _ = core::save_settings(&self.saved); self.say(format!("Play logs in as {}.", a.name)); }
                        if ui.add_enabled(idle, egui::Button::new("Password...")).clicked() { self.pass_for = Some(a.name.clone()); self.pass_new.clear(); }
                        if ui.add_enabled(idle, egui::Button::new("Delete...")).clicked() { self.confirm = Some(Confirm::DeleteAccount(a.name.clone())); }
                    }
                });
                ui.end_row();
            }
        });
        if let Some(name) = self.pass_for.clone() {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(format!("New password for {name}:"));
                ui.add(egui::TextEdit::singleline(&mut self.pass_new).password(true).desired_width(140.0));
                if ui.add_enabled(idle && !self.pass_new.is_empty(), egui::Button::new("Set")).clicked() {
                    let p = std::mem::take(&mut self.pass_new); self.pass_for = None;
                    self.run("Changing the password...", ctx, move |_| core::set_password(&name, &p));
                }
                if ui.button("Cancel").clicked() { self.pass_for = None; }
            });
        }
        if ui.button("Refresh").clicked() { self.shared.lock().unwrap().refresh_lists = true; }
    }

    fn worlds_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, _st: &core::Status, idle: bool, sim: &str, worlds: &[core::SavedWorld]) {
        ui.heading("This world");
        if !sim.is_empty() { ui.label(egui::RichText::new(sim).weak()); } else { ui.label(egui::RichText::new("(start the world to see how it's doing)").weak()); }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("Save a copy as");
            ui.add(egui::TextEdit::singleline(&mut self.world_name).hint_text("a name").desired_width(180.0));
            if ui.add_enabled(idle, egui::Button::new("Save copy")).on_hover_text("As it was at its last save (every 10 minutes, and when it stops). Your characters aren't part of it.").clicked() {
                let n = if self.world_name.trim().is_empty() { format!("world-{}", worlds.len() + 1) } else { self.world_name.clone() };
                self.world_name.clear();
                self.run("Saving a copy of the world...", ctx, move |_| { if st_world_open() { core::soap(".sim save").ok(); } core::save_world(&n) });
            }
            if ui.add_enabled(idle, egui::Button::new("Start a new world...")).clicked() { self.confirm = Some(Confirm::NewWorld); }
        });
        ui.add_space(10.0);
        ui.heading("Saved worlds");
        if worlds.is_empty() { ui.label(egui::RichText::new("None yet.").weak()); }
        egui::Grid::new("worlds").num_columns(4).spacing([14.0, 6.0]).striped(true).show(ui, |ui| {
            for w in worlds {
                ui.label(&w.name);
                ui.label(core::fmt_age(w.saved));
                ui.label(format!("{:.1} MB", w.size_mb));
                ui.horizontal(|ui| {
                    if ui.add_enabled(idle, egui::Button::new("Load...")).clicked() { self.confirm = Some(Confirm::LoadWorld(w.name.clone())); }
                    if ui.add_enabled(idle, egui::Button::new("Delete...")).clicked() { self.confirm = Some(Confirm::DeleteWorld(w.name.clone())); }
                });
                ui.end_row();
            }
        });
        ui.add_space(6.0);
        ui.label(egui::RichText::new(format!("Saved worlds live in {}", core::worlds_dir().display())).weak());
    }

    fn settings_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, st: &core::Status, idle: bool) {
        ui.add(egui::Slider::new(&mut self.settings.wanderers, 0..=300).text("adventurers around you"));
        ui.add(egui::Slider::new(&mut self.settings.townsfolk, 0..=200).text("townsfolk around you"));
        ui.checkbox(&mut self.settings.stop_with_client, "Stop the world when I close WoW");
        ui.checkbox(&mut self.settings.stop_database_too, "...and the database too (nothing left running)");
        ui.checkbox(&mut self.settings.era, "Era preset").on_hover_text(
            "Slower levelling (half XP), no dungeon-finder teleports, no quest areas on the map, riding at vanilla/TBC levels and costs. Off = stock 3.3.5.");
        ui.checkbox(&mut self.settings.unload_grids, "Let go of map areas nobody is near").on_hover_text(
            "Keeps memory flat on long sessions. Turn it off if the world ever crashes right after you leave an area.");
        ui.horizontal(|ui| {
            ui.label("WoW folder:");
            ui.add(egui::TextEdit::singleline(&mut self.settings.client).hint_text("the usual places").desired_width(320.0));
            match core::find_client() { Some(p) => { ui.label(egui::RichText::new(format!("using {}", p.display())).weak()); } None => { ui.colored_label(egui::Color32::LIGHT_RED, "no Wow.exe found"); } }
        });
        ui.horizontal(|ui| {
            ui.label("Graphics:");
            let current = self.graphics.map(|i| core::GRAPHICS[i]).unwrap_or("your own settings");
            let mut pick: Option<usize> = None;
            ui.add_enabled_ui(!st.client, |ui| {
                egui::ComboBox::from_id_salt("graphics").selected_text(current).show_ui(ui, |ui| {
                    for (i, name) in core::GRAPHICS.iter().enumerate() { if ui.selectable_label(self.graphics == Some(i), *name).clicked() { pick = Some(i); } }
                });
            });
            if st.client { ui.label(egui::RichText::new("(close WoW to change)").weak()); }
            if let Some(i) = pick {
                match core::set_graphics_preset(i) {
                    Ok(()) => { self.graphics = Some(i); self.say(format!("Graphics set to {} - it applies the next time WoW starts.", core::GRAPHICS[i])); }
                    Err(e) => self.say(format!("Problem: {e}")),
                }
            }
        });
        ui.add_space(6.0);
        if ui.add_enabled(self.settings != self.saved && idle, egui::Button::new("Save")).clicked() {
            let s = self.settings.clone();
            let era_changed = s.era != self.saved.era;
            self.saved = s.clone();
            let up = st.world.open;
            self.run(if era_changed { "Saving (and switching the Era preset)..." } else { "Saving..." }, ctx, move |_| {
                core::save_settings(&s)?;
                Ok(if up { "Saved. Restart the world for it to apply.".into() } else { "Saved. It applies the next time the world starts.".into() })
            });
        }
    }

    fn confirm_window(&mut self, ctx: &egui::Context, c: Confirm, st: &core::Status) {
        let (title, lines, yes): (&str, Vec<String>, &str) = match &c {
            Confirm::NewWorld => ("Start a new world?", vec![
                "Everyone in the living world is replaced: new adventurers, townsfolk, bounties and requests.".into(),
                "The current world is saved under Worlds first, so you can load it back.".into(),
                "The new world lives a fortnight on its own before you arrive - its first start takes a few minutes.".into(),
                "Your characters, their bags, gold and mail stay as they are.".into()], "Start a new world"),
            Confirm::LoadWorld(n) => ("Load this world?", vec![format!("\"{n}\" comes back as it was saved."), "The current world is saved under Worlds first.".into(),
                "Your characters stay as they are.".into()], "Load it"),
            Confirm::DeleteWorld(n) => ("Delete this saved world?", vec![format!("\"{n}\" is gone for good.")], "Delete"),
            Confirm::DeleteAccount(n) => ("Delete this account?", vec![format!("{n} and all its characters are gone for good."), "(The world must be running.)".into()], "Delete"),
            Confirm::Close => ("Close the launcher?", vec!["The servers are still running.".into()], "Stop everything and close"),
        };
        egui::Window::new(title).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            for l in &lines { ui.label(l); }
            if matches!(c, Confirm::NewWorld | Confirm::LoadWorld(_)) && (st.world.running || st.client) {
                ui.colored_label(egui::Color32::LIGHT_YELLOW, "The world will be stopped first - you'll be disconnected if you're playing.");
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let go = ui.button(yes).clicked();
                let leave = matches!(c, Confirm::Close) && ui.button("Close, leave them running").clicked();
                if ui.button("Cancel").clicked() { self.confirm = None; }
                if leave { self.confirm = None; self.closing_ok = true; ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                if go {
                    self.confirm = None;
                    match c {
                        Confirm::NewWorld => self.run("Saving the old world and clearing the way...", ctx, |_| core::new_world()),
                        Confirm::LoadWorld(n) => self.run("Loading the world...", ctx, move |_| core::load_world(&n)),
                        Confirm::DeleteWorld(n) => self.run("Deleting...", ctx, move |_| core::delete_saved_world(&n)),
                        Confirm::DeleteAccount(n) => self.run("Deleting the account...", ctx, move |_| core::delete_account(&n)),
                        Confirm::Close => {
                            let ctx2 = ctx.clone();
                            self.closing_ok = true;
                            self.run("Saving and stopping everything...", ctx, move |_| {
                                let r = core::stop_all(true);
                                if r.is_ok() { ctx2.send_viewport_cmd(egui::ViewportCommand::Close); }
                                r.map(|_| "Everything is stopped.".into())
                            });
                        }
                    }
                }
            });
        });
    }
}

fn st_world_open() -> bool { core::status().world.open }

/// The window icon, drawn here (no image files to ship): a gold coin with a dark tree in it.
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
    if !core::single_instance() { return Ok(()); }   // one launcher at a time: two would fight over the same servers
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 680.0]).with_min_inner_size([760.0, 520.0]).with_title("SoloWoW").with_icon(icon()),
        ..Default::default()
    };
    eframe::run_native("SoloWoW", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
