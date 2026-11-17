//! SoloWoW launcher logic - no UI. Starting and stopping the world (database, login server, world server), launching
//! the player's 3.3.5a client, a few settings, and the living world's feed (sim status over SOAP, the chronicle).
//! The heavy lifting of starting the world stays in scripts\play.ps1 (accounts, configs, realmlist); this crate drives it.

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

pub const ROOT: &str = r"C:\SoloWoW";
pub const ACCOUNT: &str = "SOLO";
const PASSWORD: &str = "solo";
const SOAP_PORT: u16 = 7879;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Status {
    pub database: bool,
    pub login: bool,
    pub world: bool,
    pub client: bool,
}

impl Status {
    pub fn world_up(&self) -> bool { self.database && self.login && self.world }
}

pub fn root() -> PathBuf { PathBuf::from(ROOT) }

/// The player's own client: the first folder in the usual places that has Wow.exe.
pub fn find_client() -> Option<PathBuf> {
    [r"C:\SoloWoW\WOTLK Client", r"C:\SoloWoW\client", r"C:\WOTLK Client", r"C:\Games\WOTLK Client", r"D:\WOTLK Client"]
        .iter()
        .map(PathBuf::from)
        .find(|p| p.join("Wow.exe").exists())
}

fn port_open(port: u16) -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

#[cfg(windows)]
fn hidden(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000) // CREATE_NO_WINDOW
}
#[cfg(not(windows))]
fn hidden(cmd: &mut Command) -> &mut Command { cmd }

fn process_running(image: &str) -> bool {
    let mut cmd = Command::new("tasklist");
    hidden(&mut cmd).args(["/FI", &format!("IMAGENAME eq {image}"), "/NH"]).stdout(Stdio::piped()).stderr(Stdio::null());
    match cmd.output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_lowercase().contains(&image.to_lowercase()),
        Err(_) => false,
    }
}

pub fn status() -> Status {
    Status {
        database: port_open(3310),
        login: port_open(3724),
        world: port_open(8085),
        client: process_running("Wow.exe"),
    }
}

fn script(name: &str, args: &[&str]) -> std::io::Result<Child> {
    let path = root().join("scripts").join(name);
    let mut cmd = Command::new("powershell");
    hidden(&mut cmd)
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(path)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.spawn()
}

/// Starts database, login and world (scripts\play.ps1 -NoClient). Returns once the script is done (the world is up or failed).
pub fn start_world() -> Result<String, String> {
    let child = script("play.ps1", &["-NoClient"]).map_err(|e| format!("could not run play.ps1: {e}"))?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if status().world_up() { Ok(last_line(&text)) } else { Err(if text.is_empty() { "the world did not come up".into() } else { last_line(&text) }) }
}

/// Stops world and login servers (the database keeps running, like stop.ps1).
pub fn stop_world() -> Result<(), String> {
    let child = script("stop.ps1", &[]).map_err(|e| format!("could not run stop.ps1: {e}"))?;
    child.wait_with_output().map(|_| ()).map_err(|e| e.to_string())
}

/// Starts the living world over (scripts\reset-sim.ps1): stops the world, sets the current one aside, archives its
/// chronicle. With `undo`, brings back the world that was set aside instead. Characters are never touched.
pub fn reset_world(undo: bool) -> Result<String, String> {
    let args: &[&str] = if undo { &["-Undo"] } else { &[] };
    let child = script("reset-sim.ps1", args).map_err(|e| format!("could not run reset-sim.ps1: {e}"))?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let text = last_line(&String::from_utf8_lossy(&out.stdout));
    if out.status.success() { Ok(text) } else { Err(if text.is_empty() { "the reset did not finish".into() } else { text }) }
}

/// Whether there is an earlier world set aside that `reset_world(true)` could bring back.
pub fn has_previous_world() -> bool {
    root().join("logs").read_dir().map(|d| d.flatten().any(|e| e.file_name().to_string_lossy().starts_with("chronicle-before-"))).unwrap_or(false)
}

fn last_line(s: &str) -> String { s.lines().filter(|l| !l.trim().is_empty()).last().unwrap_or("").trim().to_string() }

/// Opens the client. Clears its WDB cache first: generated names, quests and townsfolk live under reused ids.
pub fn launch_client() -> Result<(), String> {
    let client = find_client().ok_or("no WoW 3.3.5a client found (put it in C:\\SoloWoW\\WOTLK Client)")?;
    let wdb = client.join("Cache").join("WDB");
    if wdb.exists() { let _ = fs::remove_dir_all(&wdb); }
    Command::new(client.join("Wow.exe")).current_dir(&client).spawn().map(|_| ()).map_err(|e| format!("could not start Wow.exe: {e}"))
}

// ------------------------------------------------------------------------------------------- the living world's feed

fn base64(input: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

fn xml_escape(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }
fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&#xD;", "").replace("&amp;", "&")
}

/// Runs a server command over SOAP (the same account the scripts use) and returns its text.
pub fn soap(command: &str) -> Result<String, String> {
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><SOAP-ENV:Envelope xmlns:SOAP-ENV=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:ns1=\"urn:AC\">\
         <SOAP-ENV:Body><ns1:executeCommand><command>{}</command></ns1:executeCommand></SOAP-ENV:Body></SOAP-ENV:Envelope>",
        xml_escape(command)
    );
    let addr: SocketAddr = ([127, 0, 0, 1], SOAP_PORT).into();
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    s.set_read_timeout(Some(Duration::from_secs(20))).ok();
    let req = format!(
        "POST / HTTP/1.1\r\nHost: 127.0.0.1:{SOAP_PORT}\r\nAuthorization: Basic {}\r\nContent-Type: text/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        base64(format!("{ACCOUNT}:{PASSWORD}").as_bytes()), body.len(), body
    );
    s.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut resp = String::new();
    s.read_to_string(&mut resp).map_err(|e| e.to_string())?;
    for tag in ["result", "faultstring"] {
        if let (Some(a), Some(b)) = (resp.find(&format!("<{tag}>")), resp.find(&format!("</{tag}>"))) {
            return Ok(xml_unescape(&resp[a + tag.len() + 2..b]).trim().to_string());
        }
    }
    Err("no answer from the world".into())
}

/// The last `n` lines of the chronicle (notable happenings, newest last), without the "=== Day" headers' blank lines.
pub fn chronicle_tail(n: usize) -> Vec<String> {
    let path = root().join("logs").join("chronicle.txt");
    let Ok(mut f) = fs::File::open(&path) else { return Vec::new() };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    let start = len.saturating_sub(64 * 1024);
    use std::io::Seek;
    if f.seek(std::io::SeekFrom::Start(start)).is_err() { return Vec::new(); }
    let mut buf = Vec::new();
    if f.read_to_end(&mut buf).is_err() { return Vec::new(); }
    let text = String::from_utf8_lossy(&buf);
    let lines: Vec<String> = text.lines().filter(|l| !l.trim().is_empty() && !l.starts_with("######")).map(|l| l.to_string()).collect();
    lines[lines.len().saturating_sub(n)..].to_vec()
}

// ------------------------------------------------------------------------------------------- settings

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub wanderers: u32,   // NpcBot.Bubble.Count: adventurers around you
    pub townsfolk: u32,   // NpcBot.Bubble.Townsfolk
    pub stop_with_client: bool,
    pub era: bool,            // the "Era" preset: half XP, no dungeon-finder teleport, quest map areas off, vanilla/TBC riding
    pub unload_grids: bool,   // SoloWoW.UnloadIdleGrids: let go of map areas nobody is near
}

fn world_conf() -> PathBuf { root().join("server").join("configs").join("worldserver.conf") }
fn launcher_conf() -> PathBuf { root().join("launcher.cfg") }

fn conf_get(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let l = l.trim_start();
        let (k, v) = l.split_once('=')?;
        (k.trim() == key).then(|| v.trim().trim_matches('"').to_string())
    })
}

fn conf_set(text: &str, key: &str, value: &str) -> String {
    let mut found = false;
    let mut out: Vec<String> = text
        .lines()
        .map(|l| match l.trim_start().split_once('=') {
            Some((k, _)) if k.trim() == key => { found = true; format!("{key} = {value}") }
            _ => l.to_string(),
        })
        .collect();
    if !found { out.push(format!("{key} = {value}")); }
    out.join("\r\n") + "\r\n"
}

pub fn load_settings() -> Settings {
    let conf = fs::read_to_string(world_conf()).unwrap_or_default();
    let mine = fs::read_to_string(launcher_conf()).unwrap_or_default();
    Settings {
        wanderers: conf_get(&conf, "NpcBot.Bubble.Count").and_then(|v| v.parse().ok()).unwrap_or(40),
        townsfolk: conf_get(&conf, "NpcBot.Bubble.Townsfolk").and_then(|v| v.parse().ok()).unwrap_or(30),
        stop_with_client: conf_get(&mine, "StopWithClient").map(|v| v != "0").unwrap_or(true),
        era: conf_get(&conf, "Rate.XP.Kill").map(|v| v.parse::<f32>().map(|x| x < 0.99).unwrap_or(true)).unwrap_or(true),
        unload_grids: conf_get(&conf, "SoloWoW.UnloadIdleGrids").map(|v| v != "0").unwrap_or(true),
    }
}

/// Saves settings. World settings take effect the next time the world starts. Switching the Era preset also changes
/// the world database (scripts\era-db.ps1, reversible), which starts MySQL if it has to.
pub fn save_settings(s: &Settings) -> Result<(), String> {
    let path = world_conf();
    let mut conf = fs::read_to_string(&path).map_err(|e| format!("can't read worldserver.conf: {e}"))?;
    let era_was = conf_get(&conf, "Rate.XP.Kill").map(|v| v.parse::<f32>().map(|x| x < 0.99).unwrap_or(true)).unwrap_or(true);
    conf = conf_set(&conf, "NpcBot.Bubble.Count", &s.wanderers.to_string());
    conf = conf_set(&conf, "NpcBot.Bubble.Townsfolk", &s.townsfolk.to_string());
    conf = conf_set(&conf, "SoloWoW.UnloadIdleGrids", if s.unload_grids { "1" } else { "0" });
    let xp = if s.era { "0.5" } else { "1" };
    for k in ["Rate.XP.Kill", "Rate.XP.Quest", "Rate.XP.Explore"] { conf = conf_set(&conf, k, xp); }
    conf = conf_set(&conf, "DungeonFinder.OptionsMask", if s.era { "0" } else { "7" });
    if s.era != era_was {
        let args: &[&str] = if s.era { &[] } else { &["-Off"] };
        let out = script("era-db.ps1", args).map_err(|e| format!("could not run era-db.ps1: {e}"))?.wait_with_output().map_err(|e| e.to_string())?;
        if !out.status.success() { return Err(format!("the Era database change failed: {}", last_line(&String::from_utf8_lossy(&out.stdout)))); }
    }
    fs::write(&path, conf).map_err(|e| format!("can't write worldserver.conf: {e}"))?;
    let mine = conf_set(&fs::read_to_string(launcher_conf()).unwrap_or_default(), "StopWithClient", if s.stop_with_client { "1" } else { "0" });
    fs::write(launcher_conf(), mine).map_err(|e| e.to_string())
}

// ------------------------------------------------------------------------------------------- the client's graphics

/// Graphics presets written into the client's WTF\Config.wtf (only while WoW is closed - it rewrites the file on exit).
pub const GRAPHICS: [&str; 4] = ["Low", "Medium", "High", "Ultra"];
const GRAPHICS_KEYS: [(&str, [&str; 4]); 8] = [
    ("farclip",             ["300", "600", "1000", "1277"]),
    ("environmentDetail",   ["0.5", "1", "1.25", "1.5"]),
    ("groundEffectDensity", ["16", "64", "128", "256"]),
    ("groundEffectDist",    ["40", "70", "100", "140"]),
    ("extShadowQuality",    ["0", "2", "4", "5"]),
    ("particleDensity",     ["0.5", "0.75", "1", "1"]),
    ("weatherDensity",      ["0", "1", "2", "3"]),
    ("textureFilteringMode", ["0", "2", "4", "5"]),
];

fn wtf() -> Option<PathBuf> { find_client().map(|c| c.join("WTF").join("Config.wtf")) }

fn wtf_get(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let rest = l.trim().strip_prefix("SET ")?;
        let (k, v) = rest.split_once(' ')?;
        (k.eq_ignore_ascii_case(key)).then(|| v.trim().trim_matches('"').to_string())
    })
}

/// Which preset the client is on now (None = its own mix of settings).
pub fn graphics_preset() -> Option<usize> {
    let text = fs::read_to_string(wtf()?).ok()?;
    (0..4).find(|&i| GRAPHICS_KEYS.iter().all(|(k, v)| wtf_get(&text, k).as_deref() == Some(v[i])))
}

pub fn set_graphics_preset(i: usize) -> Result<(), String> {
    if i >= 4 { return Err("no such preset".into()); }
    if status().client { return Err("close WoW first - it rewrites its settings when it exits".into()); }
    let path = wtf().ok_or("no WoW client found")?;
    let text = fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    for (k, v) in GRAPHICS_KEYS.iter() {
        let line = format!("SET {k} \"{}\"", v[i]);
        match lines.iter().position(|l| l.trim().strip_prefix("SET ").and_then(|r| r.split_once(' ')).map(|(kk, _)| kk.eq_ignore_ascii_case(k)).unwrap_or(false)) {
            Some(p) => lines[p] = line,
            None => lines.push(line),
        }
    }
    if let Some(dir) = path.parent() { let _ = fs::create_dir_all(dir); }
    fs::write(&path, lines.join("\r\n") + "\r\n").map_err(|e| format!("can't write Config.wtf: {e}"))
}

pub fn exists(p: &Path) -> bool { p.exists() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn b64() { assert_eq!(base64(b"SOLO:solo"), "U09MTzpzb2xv"); }
    #[test]
    fn conf() {
        let t = "# x\nNpcBot.Bubble.Count = 40\n";
        assert_eq!(conf_get(t, "NpcBot.Bubble.Count").as_deref(), Some("40"));
        assert!(conf_set(t, "NpcBot.Bubble.Count", "25").contains("NpcBot.Bubble.Count = 25"));
        assert!(conf_set(t, "New.Key", "1").contains("New.Key = 1"));
    }
    #[test]
    fn wtf_keys() {
        let t = "SET farclip \"397\"\r\nSET gxWindow \"1\"\r\n";
        assert_eq!(wtf_get(t, "farclip").as_deref(), Some("397"));
        assert_eq!(wtf_get(t, "FARCLIP").as_deref(), Some("397"));
        assert_eq!(wtf_get(t, "nothere"), None);
    }
}
