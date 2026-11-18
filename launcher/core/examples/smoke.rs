// A run through the launcher's server control without the window: start everything, wait until the world lets players
// in, make / change / delete an account, save and delete a world copy, stop everything. cargo run --example smoke
use solowow_core as core;
use std::time::{Duration, Instant};

fn main() {
    let t0 = Instant::now();
    let say = |s: &str| println!("[{:>4}s] {s}", t0.elapsed().as_secs());
    say(&format!("status before: {:?}", core::status()));
    if let Err(e) = core::start_database() { say(&format!("database: {e}")); return; }
    if let Err(e) = core::start_login() { say(&format!("login: {e}")); return; }
    if let Err(e) = core::start_world_server() { say(&format!("world: {e}")); return; }
    say("world port open; waiting until it lets players in");
    let mut last = String::new();
    while !core::world_ready() {
        let p = core::world_progress();
        if p != last { say(&format!("  getting ready: {p}")); last = p; }
        if t0.elapsed() > Duration::from_secs(1500) { say("gave up waiting"); break; }
        std::thread::sleep(Duration::from_secs(3));
    }
    say(&format!("ready: {} - players online {:?}", core::world_ready(), core::players_online()));
    say(&format!("sim: {:?}", core::soap(".sim status").map(|s| s.chars().take(160).collect::<String>())));
    say(&format!("create: {:?}", core::create_account("SmokeTest", "pass123", 0)));
    say(&format!("again: {:?}", core::create_account("SmokeTest", "pass123", 0)));
    say(&format!("password: {:?}", core::set_password("SmokeTest", "other1")));
    say(&format!("gm: {:?}", core::set_gm("SmokeTest", 2)));
    say(&format!("accounts: {:?}", core::accounts().map(|v| v.iter().map(|a| format!("{}:gm{}:{}ch", a.name, a.gm, a.characters)).collect::<Vec<_>>())));
    say(&format!("delete: {:?}", core::delete_account("SmokeTest")));
    say(&format!("save world: {:?}", core::save_world("smoke-test")));
    say(&format!("saved: {:?}", core::saved_worlds().iter().map(|w| format!("{} {:.1}MB", w.name, w.size_mb)).collect::<Vec<_>>()));
    say(&format!("delete copy: {:?}", core::delete_saved_world("smoke-test")));
    say(&format!("status: {:?}", core::status()));
    say(&format!("stop all: {:?}", core::stop_all(true)));
    say(&format!("status after: {:?}", core::status()));
}
