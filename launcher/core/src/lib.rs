//! SoloWoW launcher logic - no UI, and no PowerShell (Nov 18: the PowerShell manager is gone; this is the one place that
//! runs the servers). The database (MySQL), the login server and the world server: start, stop, restart, status, RAM and
//! uptime, their logs, the world's console (SOAP); accounts (made here, the way the login server checks them: SRP6);
//! living worlds (start a new one, save a copy, load one back); the player's client (realmlist, account name, cache);
//! settings; the living world's feed. Reused by the desktop launcher (app) and, later, the Android one.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

pub const ROOT: &str = r"C:\SoloWoW";
/// The launcher's own account: game master 3, used for the world's console (SOAP). Not one to play on.
pub const SERVICE_ACCOUNT: &str = "SOLO";
const SERVICE_PASSWORD: &str = "solo";
const SOAP_PORT: u16 = 7879;
pub const DB_PORT: u16 = 3310;
pub const LOGIN_PORT: u16 = 3724;
pub const WORLD_PORT: u16 = 8085;
const MYSQL_BIN: &str = r"C:\Program Files\MySQL\MySQL Server 8.4\bin";

pub fn root() -> PathBuf { PathBuf::from(ROOT) }
fn server_dir() -> PathBuf { root().join("server") }
fn logs_dir() -> PathBuf { root().join("logs") }
pub fn worlds_dir() -> PathBuf { root().join("db").join("worlds") }

// ============================================================================================ processes

#[cfg(windows)]
fn hidden(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000) // CREATE_NO_WINDOW: nothing pops up on the desktop
}
#[cfg(not(windows))]
fn hidden(cmd: &mut Command) -> &mut Command { cmd }

fn port_open(port: u16) -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

/// (pid, memory in KB) of every process with this image name.
fn processes(image: &str) -> Vec<(u32, u64)> {
    let mut cmd = Command::new("tasklist");
    hidden(&mut cmd).args(["/FI", &format!("IMAGENAME eq {image}"), "/FO", "CSV", "/NH"]).stdout(Stdio::piped()).stderr(Stdio::null());
    let Ok(out) = cmd.output() else { return Vec::new() };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split("\",\"").map(|s| s.trim_matches('"')).collect();
            if f.len() < 5 || !f[0].eq_ignore_ascii_case(image) { return None; }
            let pid = f[1].parse().ok()?;
            let kb = f[4].chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
            Some((pid, kb))
        })
        .collect()
}

fn kill(pid: u32) {
    let mut cmd = Command::new("taskkill");
    let _ = hidden(&mut cmd).args(["/PID", &pid.to_string(), "/F"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

fn wait_until(seconds: u64, mut cond: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(seconds);
    while Instant::now() < end {
        if cond() { return true; }
        std::thread::sleep(Duration::from_millis(400));
    }
    cond()
}

/// When we started each server this session (for its uptime).
static STARTED: Mutex<Option<HashMap<&'static str, Instant>>> = Mutex::new(None);
fn mark_started(what: &'static str) { STARTED.lock().unwrap().get_or_insert_with(HashMap::new).insert(what, Instant::now()); }
fn started(what: &str) -> Option<Duration> { STARTED.lock().unwrap().as_ref()?.get(what).map(|t| t.elapsed()) }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Proc {
    pub running: bool,    // the process is there
    pub open: bool,       // its port answers
    pub mem_kb: u64,
    pub uptime: Option<Duration>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Status {
    pub database: Proc,
    pub login: Proc,
    pub world: Proc,
    pub client: bool,
}

impl Status {
    pub fn world_up(&self) -> bool { self.database.open && self.login.open && self.world.open }
    pub fn anything_running(&self) -> bool { self.database.running || self.login.running || self.world.running }
}

fn proc_status(image: &str, port: u16, key: &str) -> Proc {
    let ps = processes(image);
    let open = port_open(port);
    Proc { running: !ps.is_empty() || open, open, mem_kb: ps.iter().map(|p| p.1).sum(), uptime: if ps.is_empty() { None } else { started(key) } }
}

pub fn status() -> Status {
    Status {
        database: proc_status("mysqld.exe", DB_PORT, "db"),
        login: proc_status("authserver.exe", LOGIN_PORT, "login"),
        world: proc_status("worldserver.exe", WORLD_PORT, "world"),
        client: !processes("Wow.exe").is_empty(),
    }
}

/// Another AzerothCore (AzerothManager, say) on our ports would take the login.
fn foreign_server() -> bool {
    let mut cmd = Command::new("wmic");
    hidden(&mut cmd).args(["process", "where", "name='worldserver.exe' or name='authserver.exe'", "get", "ExecutablePath"]).stdout(Stdio::piped()).stderr(Stdio::null());
    match cmd.output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim()).filter(|l| l.to_lowercase().ends_with(".exe")).any(|l| !l.to_lowercase().starts_with(&ROOT.to_lowercase())),
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------------------- the database

pub fn start_database() -> Result<(), String> {
    if port_open(DB_PORT) { return Ok(()); }
    let mut cmd = Command::new(Path::new(MYSQL_BIN).join("mysqld.exe"));
    hidden(&mut cmd).arg(format!("--defaults-file={}", root().join("db").join("my.ini").display())).stdout(Stdio::null()).stderr(Stdio::null());
    cmd.spawn().map_err(|e| format!("could not start MySQL: {e}"))?;
    mark_started("db");
    if wait_until(60, || port_open(DB_PORT)) { Ok(()) } else { Err(format!("the database did not start - see {}\\db\\data\\*.err", ROOT)) }
}

pub fn stop_database() -> Result<(), String> {
    if !port_open(DB_PORT) && processes("mysqld.exe").is_empty() { return Ok(()); }
    if !processes("worldserver.exe").is_empty() || !processes("authserver.exe").is_empty() { return Err("stop the login and world servers first".into()); }
    let _ = sql_raw("", "SHUTDOWN");
    if wait_until(30, || !port_open(DB_PORT) && processes("mysqld.exe").is_empty()) { return Ok(()); }
    for (pid, _) in processes("mysqld.exe") { kill(pid); }
    if wait_until(15, || !port_open(DB_PORT)) { Ok(()) } else { Err(format!("the database is still running on port {DB_PORT}")) }
}

/// One query through MySQL's own client (tab-separated rows, no header).
fn sql_raw(db: &str, query: &str) -> Result<String, String> {
    let mut cmd = Command::new(Path::new(MYSQL_BIN).join("mysql.exe"));
    hidden(&mut cmd)
        .env("MYSQL_PWD", "acore")
        .args(["--host=127.0.0.1", &format!("--port={DB_PORT}"), "--user=acore", "--batch", "--skip-column-names", "--default-character-set=utf8mb4", "-e", query])
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    if !db.is_empty() { cmd.arg(db); }
    let out = cmd.output().map_err(|e| format!("could not run mysql: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).lines().filter(|l| !l.contains("[Warning]")).collect::<Vec<_>>().join(" ");
        return Err(if err.is_empty() { "the database said no".into() } else { err });
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn sql(db: &str, query: &str) -> Result<Vec<Vec<String>>, String> {
    Ok(sql_raw(db, query)?.lines().filter(|l| !l.is_empty()).map(|l| l.split('\t').map(|s| s.to_string()).collect()).collect())
}

fn sql_quote(s: &str) -> String { s.replace('\\', "\\\\").replace('\'', "''") }

// ---------------------------------------------------------------------------------------- the servers

fn spawn_server(exe: &str, conf: &str, out: &str) -> Result<(), String> {
    let bin = server_dir();
    let log = fs::File::create(logs_dir().join(out)).map_err(|e| e.to_string())?;
    let mut cmd = Command::new(bin.join(exe));
    hidden(&mut cmd)
        .env("AC_DISABLE_INTERACTIVE", "1")
        .args(["-c", &bin.join("configs").join(conf).to_string_lossy()])
        .current_dir(&bin)
        .stdin(Stdio::null()).stdout(log.try_clone().map_err(|e| e.to_string())?).stderr(log);
    cmd.spawn().map(|_| ()).map_err(|e| format!("could not start {exe}: {e}"))
}

/// The login server won't list a realm the world flagged offline when it stopped (flag 3).
fn reset_realm_flags() { let _ = sql_raw("acore_auth", "UPDATE realmlist SET flag = 2 WHERE flag = 3"); }

pub fn start_login() -> Result<(), String> {
    if !processes("authserver.exe").is_empty() { return Ok(()); }
    if foreign_server() { return Err("another AzerothCore server (AzerothManager?) is running - stop it first".into()); }
    start_database()?;
    reset_realm_flags();
    ensure_service_account()?;
    spawn_server("authserver.exe", "authserver.conf", "auth.out")?;
    mark_started("login");
    if wait_until(30, || port_open(LOGIN_PORT)) { Ok(()) } else { Err("the login server did not open its port - see the Login log".into()) }
}

pub fn stop_login() -> Result<(), String> {
    for (pid, _) in processes("authserver.exe") { kill(pid); }
    if wait_until(10, || processes("authserver.exe").is_empty()) { Ok(()) } else { Err("the login server would not stop".into()) }
}

/// Starts the world server and returns once its port is open (it may still be getting the living world ready:
/// see `world_ready`).
pub fn start_world_server() -> Result<(), String> {
    if !processes("worldserver.exe").is_empty() { return Ok(()); }
    if foreign_server() { return Err("another AzerothCore server (AzerothManager?) is running - stop it first".into()); }
    start_database()?;
    reset_realm_flags();
    let _ = sql_raw("acore_auth", "UPDATE realmlist SET icon = 8 WHERE id = 1");   // RP-PvP, as GameType in worldserver.conf
    spawn_server("worldserver.exe", "worldserver.conf", "world.out")?;
    mark_started("world");
    if wait_until(300, || port_open(WORLD_PORT) || processes("worldserver.exe").is_empty()) && port_open(WORLD_PORT) { Ok(()) }
    else { Err("the world server stopped while starting - see the World log".into()) }
}

/// The world answers: it has finished getting the living world ready (a new world lives a fortnight first; a restart
/// catches up on the time it was away) and lets players in. Until then a client sits at "Connected" (Nov 18).
pub fn world_ready() -> bool { port_open(WORLD_PORT) && soap_timeout("server info", Duration::from_secs(4)).is_ok() }

/// What the world is busy with while it gets ready: the last SoloWoW line of its log.
pub fn world_progress() -> String {
    log_tail("Server.log", 200).into_iter().rev().find(|l| l.starts_with("SoloWoW") || l.starts_with("SoloSim")).unwrap_or_default()
}

/// Saves and shuts the world down ('server shutdown' saves the players and the living world). Waits up to 4 minutes;
/// Err if it is still running then (force_stop_world can end it, losing what happened since the last save).
pub fn stop_world_server() -> Result<(), String> {
    if processes("worldserver.exe").is_empty() { return Ok(()); }
    if port_open(WORLD_PORT) { let _ = soap_timeout("server shutdown 1", Duration::from_secs(10)); }
    else { return Err("the world server is running but not answering".into()); }
    if wait_until(240, || processes("worldserver.exe").is_empty()) { reset_realm_flags(); Ok(()) }
    else { Err("the world server did not shut down within 4 minutes".into()) }
}

pub fn force_stop_world() {
    for (pid, _) in processes("worldserver.exe") { kill(pid); }
    wait_until(15, || processes("worldserver.exe").is_empty());
    reset_realm_flags();
}

pub fn start_all() -> Result<(), String> { start_database()?; start_login()?; start_world_server() }

/// World, login, then (with `database`) MySQL too: nothing left running.
pub fn stop_all(database: bool) -> Result<(), String> {
    stop_world_server()?;
    stop_login()?;
    if database { stop_database()?; }
    Ok(())
}

// ---------------------------------------------------------------------------------------- the world's console (SOAP)

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

fn soap_timeout(command: &str, wait: Duration) -> Result<String, String> {
    let command = command.trim().trim_start_matches('.');
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><SOAP-ENV:Envelope xmlns:SOAP-ENV=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:ns1=\"urn:AC\">\
         <SOAP-ENV:Body><ns1:executeCommand><command>{}</command></ns1:executeCommand></SOAP-ENV:Body></SOAP-ENV:Envelope>",
        xml_escape(command)
    );
    let addr: SocketAddr = ([127, 0, 0, 1], SOAP_PORT).into();
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    s.set_read_timeout(Some(wait)).ok();
    let req = format!(
        "POST / HTTP/1.1\r\nHost: 127.0.0.1:{SOAP_PORT}\r\nAuthorization: Basic {}\r\nContent-Type: text/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        base64(format!("{SERVICE_ACCOUNT}:{SERVICE_PASSWORD}").as_bytes()), body.len(), body
    );
    s.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut resp = String::new();
    s.read_to_string(&mut resp).map_err(|_| "the world is busy".to_string())?;
    for tag in ["result", "faultstring"] {
        if let (Some(a), Some(b)) = (resp.find(&format!("<{tag}>")), resp.find(&format!("</{tag}>"))) {
            return Ok(xml_unescape(&resp[a + tag.len() + 2..b]).trim().to_string());
        }
    }
    Err("no answer from the world".into())
}

/// Runs a server command (".sim status", "server info", "account create ..." - with or without the dot).
pub fn soap(command: &str) -> Result<String, String> { soap_timeout(command, Duration::from_secs(60)) }

/// Players online, from 'server info' ("Connected players: 1. Characters in world: 1.").
pub fn players_online() -> Option<u32> {
    let info = soap_timeout("server info", Duration::from_secs(4)).ok()?;
    let at = info.find("Connected players:")?;
    info[at + 18..].trim_start().split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()
}

// ============================================================================================ accounts

// SHA-1 (accounts only: the login server checks a password by SRP6 over SHA-1)
fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let mut msg = data.to_vec();
    let bits = (data.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&bits.to_be_bytes());
    for block in msg.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..16 { w[i] = u32::from_be_bytes([block[4 * i], block[4 * i + 1], block[4 * i + 2], block[4 * i + 3]]); }
        for i in 16..80 { w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1); }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let t = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(*wi);
            e = d; d = c; c = b.rotate_left(30); b = a; a = t;
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b); h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d); h[4] = h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for i in 0..5 { out[4 * i..4 * i + 4].copy_from_slice(&h[i].to_be_bytes()); }
    out
}

// 320-bit unsigned numbers, little-endian 64-bit limbs: enough for SRP6's 256-bit modulus
type Big = [u64; 5];
fn big_from_le(bytes: &[u8]) -> Big {
    let mut b = [0u64; 5];
    for (i, byte) in bytes.iter().enumerate().take(40) { b[i / 8] |= u64::from(*byte) << (8 * (i % 8)); }
    b
}
fn big_to_le(b: &Big, len: usize) -> Vec<u8> { (0..len).map(|i| (b[i / 8] >> (8 * (i % 8))) as u8).collect() }
fn big_ge(a: &Big, b: &Big) -> bool {
    for i in (0..5).rev() { if a[i] != b[i] { return a[i] > b[i]; } }
    true
}
fn big_sub(a: &mut Big, b: &Big) {
    let mut borrow = 0u64;
    for i in 0..5 {
        let (d1, o1) = a[i].overflowing_sub(b[i]);
        let (d2, o2) = d1.overflowing_sub(borrow);
        a[i] = d2; borrow = u64::from(o1 || o2);
    }
}
fn big_add_mod(a: &Big, b: &Big, n: &Big) -> Big {
    let mut r = [0u64; 5]; let mut carry = 0u64;
    for i in 0..5 {
        let (s1, o1) = a[i].overflowing_add(b[i]);
        let (s2, o2) = s1.overflowing_add(carry);
        r[i] = s2; carry = u64::from(o1 || o2);
    }
    if big_ge(&r, n) { big_sub(&mut r, n); }
    r
}
fn big_bit(a: &Big, i: usize) -> bool { (a[i / 64] >> (i % 64)) & 1 == 1 }
fn big_mul_mod(a: &Big, b: &Big, n: &Big) -> Big {
    let mut r = [0u64; 5];
    for i in (0..320).rev() {
        r = big_add_mod(&r, &r, n);
        if big_bit(b, i) { r = big_add_mod(&r, a, n); }
    }
    r
}
fn big_pow_mod(base: &Big, exp: &Big, n: &Big) -> Big {
    let mut r: Big = [1, 0, 0, 0, 0];
    for i in (0..320).rev() {
        r = big_mul_mod(&r, &r, n);
        if big_bit(exp, i) { r = big_mul_mod(&r, base, n); }
    }
    r
}

/// The login server's check value for a name and password with this salt (AzerothCore's SRP6: v = g^x mod N,
/// x = SHA1(salt | SHA1(NAME:PASSWORD)), all little-endian).
fn srp_verifier(user: &str, pass: &str, salt: &[u8; 32]) -> [u8; 32] {
    const N_HEX: &str = "894B645E89E1535BBDAD5B8B290650530801B18EBFBF5E8FAB3C82872A3E9BB7";
    let n_be: Vec<u8> = (0..32).map(|i| u8::from_str_radix(&N_HEX[2 * i..2 * i + 2], 16).unwrap()).collect();
    let n_le: Vec<u8> = n_be.into_iter().rev().collect();
    let n = big_from_le(&n_le);
    let h1 = sha1(format!("{}:{}", user.to_uppercase(), pass.to_uppercase()).as_bytes());
    let mut s = salt.to_vec(); s.extend_from_slice(&h1);
    let x = big_from_le(&sha1(&s));
    let v = big_pow_mod(&[7, 0, 0, 0, 0], &x, &n);
    let mut out = [0u8; 32];
    out.copy_from_slice(&big_to_le(&v, 32));
    out
}

fn random_bytes<const N: usize>() -> [u8; N] {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut out = [0u8; N];
    for (i, chunk) in out.chunks_mut(8).enumerate() {
        let mut h = RandomState::new().build_hasher();   // seeded by the OS each time
        h.write_usize(i);
        h.write_u128(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
        chunk.copy_from_slice(&h.finish().to_le_bytes()[..chunk.len()]);
    }
    out
}

fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02X}")).collect() }

#[derive(Clone, Debug, Default)]
pub struct Account {
    pub id: u32,
    pub name: String,
    pub gm: u8,
    pub characters: u32,
    pub last_login: String,
    pub online: bool,
}

pub fn valid_account_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 16 { return Err("an account name is 1 to 16 letters or digits".into()); }
    if !name.chars().all(|c| c.is_ascii_alphanumeric()) { return Err("an account name is letters and digits only".into()); }
    Ok(())
}
fn valid_password(pass: &str) -> Result<(), String> {
    if pass.is_empty() || pass.len() > 16 { return Err("a password is 1 to 16 characters (the 3.3.5 client allows no more)".into()); }
    if !pass.chars().all(|c| c.is_ascii_graphic()) { return Err("a password is plain letters, digits and symbols, no spaces".into()); }
    Ok(())
}

pub fn accounts() -> Result<Vec<Account>, String> {
    start_database()?;
    let rows = sql("acore_auth", "SELECT a.id, a.username, IFNULL(MAX(aa.gmlevel), 0), IFNULL(a.last_login, ''), a.online, \
        (SELECT COUNT(*) FROM acore_characters.characters c WHERE c.account = a.id AND c.deleteInfos_Account IS NULL) \
        FROM account a LEFT JOIN account_access aa ON aa.id = a.id GROUP BY a.id ORDER BY a.id")?;
    Ok(rows.into_iter().filter(|r| r.len() >= 6).map(|r| Account {
        id: r[0].parse().unwrap_or(0), name: r[1].clone(), gm: r[2].parse().unwrap_or(0), last_login: r[3].clone(),
        online: r[4] != "0", characters: r[5].parse().unwrap_or(0),
    }).collect())
}

/// A new account the 3.3.5 client can log in with. `gm`: 0 player, 1-3 game master levels.
pub fn create_account(name: &str, pass: &str, gm: u8) -> Result<String, String> {
    valid_account_name(name)?; valid_password(pass)?;
    start_database()?;
    let name = name.to_uppercase();
    if !sql("acore_auth", &format!("SELECT id FROM account WHERE username = '{}'", sql_quote(&name)))?.is_empty() {
        return Err(format!("there is already an account called {name}"));
    }
    let salt = random_bytes::<32>();
    let v = srp_verifier(&name, pass, &salt);
    sql_raw("acore_auth", &format!("INSERT INTO account (username, salt, verifier, expansion) VALUES ('{}', X'{}', X'{}', 2)", sql_quote(&name), hex(&salt), hex(&v)))?;
    if gm > 0 { set_gm(&name, gm)?; }
    Ok(format!("Account {name} is ready."))
}

pub fn set_password(name: &str, pass: &str) -> Result<String, String> {
    valid_password(pass)?;
    let name = name.to_uppercase();
    let salt = random_bytes::<32>();
    let v = srp_verifier(&name, pass, &salt);
    sql_raw("acore_auth", &format!("UPDATE account SET salt = X'{}', verifier = X'{}', session_key = NULL WHERE username = '{}'", hex(&salt), hex(&v), sql_quote(&name)))?;
    Ok(format!("{name}'s password is changed."))
}

pub fn set_gm(name: &str, gm: u8) -> Result<String, String> {
    let rows = sql("acore_auth", &format!("SELECT id FROM account WHERE username = '{}'", sql_quote(&name.to_uppercase())))?;
    let id = rows.first().and_then(|r| r.first()).ok_or("no such account")?.clone();
    if gm == 0 { sql_raw("acore_auth", &format!("DELETE FROM account_access WHERE id = {id}"))?; }
    else { sql_raw("acore_auth", &format!("REPLACE INTO account_access (id, gmlevel, RealmID) VALUES ({id}, {}, -1)", gm.min(3)))?; }
    Ok(format!("{} is now {}.", name.to_uppercase(), match gm { 0 => "a player", 1 => "a moderator (GM 1)", 2 => "a game master (GM 2)", _ => "an administrator (GM 3)" }))
}

/// Deletes an account and its characters - through the world itself, so nothing is left behind (it must be running).
pub fn delete_account(name: &str) -> Result<String, String> {
    if name.eq_ignore_ascii_case(SERVICE_ACCOUNT) { return Err("that is the launcher's own account (the world's console needs it)".into()); }
    if !port_open(WORLD_PORT) { return Err("start the world first - it removes the account's characters too".into()); }
    soap(&format!("account delete {}", name.to_uppercase()))
}

/// The SOAP account the launcher talks to the world with.
fn ensure_service_account() -> Result<(), String> {
    if sql("acore_auth", &format!("SELECT id FROM account WHERE username = '{SERVICE_ACCOUNT}'"))?.is_empty() {
        create_account(SERVICE_ACCOUNT, SERVICE_PASSWORD, 3)?;
    }
    Ok(())
}

// ============================================================================================ living worlds

const SIM_PREFIX: &str = "sim\\_";

fn sim_tables() -> Result<Vec<String>, String> {
    Ok(sql("acore_characters", &format!("SELECT table_name FROM information_schema.tables WHERE table_schema = 'acore_characters' AND table_name LIKE '{SIM_PREFIX}%'"))?
        .into_iter().filter_map(|r| r.into_iter().next()).collect())
}

#[derive(Clone, Debug)]
pub struct SavedWorld { pub name: String, pub file: PathBuf, pub size_mb: f64, pub saved: SystemTime }

pub fn saved_worlds() -> Vec<SavedWorld> {
    let Ok(dir) = fs::read_dir(worlds_dir()) else { return Vec::new() };
    let mut v: Vec<SavedWorld> = dir.flatten().filter_map(|e| {
        let p = e.path();
        if p.extension()?.to_str()? != "sql" { return None; }
        let m = e.metadata().ok()?;
        Some(SavedWorld { name: p.file_stem()?.to_string_lossy().to_string(), size_mb: m.len() as f64 / 1_048_576.0, saved: m.modified().ok()?, file: p })
    }).collect();
    v.sort_by(|a, b| b.saved.cmp(&a.saved));
    v
}

fn stamp() -> String {
    // local-ish timestamp for file names without a time crate: seconds since epoch is unreadable, so YYYYMMDD-HHMM UTC
    let s = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (days, rem) = (s / 86400, s % 86400);
    let (mut y, mut d) = (1970u64, days);
    loop { let len = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 }; if d < len { break; } d -= len; y += 1; }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let months = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0; while d >= months[m] { d -= months[m]; m += 1; }
    format!("{y}{:02}{:02}-{:02}{:02}", m + 1, d + 1, rem / 3600, rem % 3600 / 60)
}

fn clean_world_name(name: &str) -> String {
    let n: String = name.trim().chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' }).collect();
    if n.is_empty() { format!("world-{}", stamp()) } else { n }
}

/// Saves a copy of the living world as it was last saved (the world saves every 10 minutes and when it stops - stop it
/// first for the very latest). Your characters aren't part of it.
pub fn save_world(name: &str) -> Result<String, String> {
    start_database()?;
    let tables = sim_tables()?;
    if tables.is_empty() { return Err("there is no living world yet".into()); }
    fs::create_dir_all(worlds_dir()).map_err(|e| e.to_string())?;
    let name = clean_world_name(name);
    let file = worlds_dir().join(format!("{name}.sql"));
    let mut cmd = Command::new(Path::new(MYSQL_BIN).join("mysqldump.exe"));
    hidden(&mut cmd).env("MYSQL_PWD", "acore")
        .args(["--host=127.0.0.1", &format!("--port={DB_PORT}"), "--user=acore", "--single-transaction", "acore_characters"])
        .args(&tables).arg(format!("--result-file={}", file.display()))
        .stdout(Stdio::null()).stderr(Stdio::piped());
    let out = cmd.output().map_err(|e| format!("could not run mysqldump: {e}"))?;
    if !out.status.success() { return Err(String::from_utf8_lossy(&out.stderr).trim().to_string()); }
    let chron = logs_dir().join("chronicle.txt");
    if chron.exists() { let _ = fs::copy(&chron, worlds_dir().join(format!("{name}.chronicle.txt"))); }
    Ok(format!("Saved the world as \"{name}\"."))
}

/// A brand-new living world (the old one is saved first, so it can be loaded back). The world is stopped; on its next
/// start it builds the new world and lets it live a fortnight before you arrive (a few minutes).
pub fn new_world() -> Result<String, String> {
    stop_world_server()?;
    start_database()?;
    let tables = sim_tables()?;
    let mut kept = String::new();
    if !tables.is_empty() {
        save_world(&format!("before-new-world-{}", stamp()))?;
        kept = " The old one is saved under Worlds.".into();
        for t in &tables { sql_raw("acore_characters", &format!("DROP TABLE `{t}`"))?; }
    }
    let chron = logs_dir().join("chronicle.txt");
    if chron.exists() { let _ = fs::rename(&chron, logs_dir().join(format!("chronicle-before-{}.txt", stamp()))); }
    Ok(format!("A new world will be made when the world next starts.{kept}"))
}

/// Puts a saved world back (the current one is saved first). Stops the world.
pub fn load_world(name: &str) -> Result<String, String> {
    let file = worlds_dir().join(format!("{name}.sql"));
    if !file.exists() { return Err(format!("no saved world called {name}")); }
    stop_world_server()?;
    start_database()?;
    let tables = sim_tables()?;
    if !tables.is_empty() {
        save_world(&format!("before-loading-{}", stamp()))?;
        for t in &tables { sql_raw("acore_characters", &format!("DROP TABLE `{t}`"))?; }
    }
    let input = fs::File::open(&file).map_err(|e| e.to_string())?;
    let mut cmd = Command::new(Path::new(MYSQL_BIN).join("mysql.exe"));
    hidden(&mut cmd).env("MYSQL_PWD", "acore")
        .args(["--host=127.0.0.1", &format!("--port={DB_PORT}"), "--user=acore", "acore_characters"])
        .stdin(input).stdout(Stdio::null()).stderr(Stdio::piped());
    let out = cmd.output().map_err(|e| format!("could not run mysql: {e}"))?;
    if !out.status.success() { return Err(String::from_utf8_lossy(&out.stderr).trim().to_string()); }
    let saved_chron = worlds_dir().join(format!("{name}.chronicle.txt"));
    let chron = logs_dir().join("chronicle.txt");
    if chron.exists() { let _ = fs::rename(&chron, logs_dir().join(format!("chronicle-before-{}.txt", stamp()))); }
    if saved_chron.exists() { let _ = fs::copy(&saved_chron, &chron); }
    Ok(format!("\"{name}\" is back. Start the world to carry on where it was."))
}

pub fn delete_saved_world(name: &str) -> Result<String, String> {
    let file = worlds_dir().join(format!("{name}.sql"));
    fs::remove_file(&file).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(worlds_dir().join(format!("{name}.chronicle.txt")));
    Ok(format!("Deleted the saved world \"{name}\"."))
}

// ============================================================================================ the client

/// The player's own client: the first folder in the usual places that has Wow.exe (or the one set in launcher.cfg).
pub fn find_client() -> Option<PathBuf> {
    let mine = fs::read_to_string(launcher_conf()).unwrap_or_default();
    if let Some(p) = conf_get(&mine, "Client").map(PathBuf::from) { if p.join("Wow.exe").exists() { return Some(p); } }
    [r"C:\SoloWoW\WOTLK Client", r"C:\SoloWoW\client", r"C:\WOTLK Client", r"C:\Games\WOTLK Client", r"D:\WOTLK Client"]
        .iter().map(PathBuf::from).find(|p| p.join("Wow.exe").exists())
}

/// Points the client at our world, fills in the account name, clears its cache (generated names, quests, items and
/// townsfolk live under re-used ids), and opens it.
pub fn launch_client(account: &str) -> Result<(), String> {
    let client = find_client().ok_or("no WoW 3.3.5a client found (put it in C:\\SoloWoW\\WOTLK Client, or pick it under Settings)")?;
    let _ = fs::write(client.join("Data").join("enUS").join("realmlist.wtf"), "set realmlist 127.0.0.1\r\n");
    let cfg = client.join("WTF").join("Config.wtf");
    let text = fs::read_to_string(&cfg).unwrap_or_default();
    let mut lines: Vec<String> = text.lines().filter(|l| {
        let k = l.trim().strip_prefix("SET ").and_then(|r| r.split_whitespace().next()).unwrap_or("").to_lowercase();
        k != "accountname" && k != "realmlist" && k != "realmname"
    }).map(|l| l.to_string()).collect();
    if !account.is_empty() { lines.push(format!("SET accountName \"{}\"", account.to_uppercase())); }
    if let Some(dir) = cfg.parent() { let _ = fs::create_dir_all(dir); }
    let _ = fs::write(&cfg, lines.join("\r\n") + "\r\n");
    let wdb = client.join("Cache").join("WDB");
    if wdb.exists() { let _ = fs::remove_dir_all(&wdb); }
    Command::new(client.join("Wow.exe")).current_dir(&client).spawn().map(|_| ()).map_err(|e| format!("could not start Wow.exe: {e}"))
}

// ============================================================================================ logs and the chronicle

fn tail_file(path: &Path, n: usize, bytes: u64) -> Vec<String> {
    let Ok(mut f) = fs::File::open(path) else { return Vec::new() };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    if f.seek(std::io::SeekFrom::Start(len.saturating_sub(bytes))).is_err() { return Vec::new(); }
    let mut buf = Vec::new();
    if f.read_to_end(&mut buf).is_err() { return Vec::new(); }
    let text = String::from_utf8_lossy(&buf);
    let lines: Vec<String> = text.lines().map(|l| l.trim_end().to_string()).filter(|l| !l.is_empty()).collect();
    lines[lines.len().saturating_sub(n)..].to_vec()
}

/// The last lines of one of the servers' logs (Server.log, Auth.log, Errors.log).
pub fn log_tail(name: &str, n: usize) -> Vec<String> { tail_file(&logs_dir().join(name), n, 96 * 1024) }

/// The last `n` lines of the chronicle (notable happenings, newest last).
pub fn chronicle_tail(n: usize) -> Vec<String> {
    tail_file(&logs_dir().join("chronicle.txt"), n + 40, 64 * 1024).into_iter().filter(|l| !l.starts_with("######")).collect::<Vec<_>>()
        .split_off(0).into_iter().rev().take(n).collect::<Vec<_>>().into_iter().rev().collect()
}

// ============================================================================================ settings

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub wanderers: u32,        // NpcBot.Bubble.Count: adventurers around you
    pub townsfolk: u32,        // NpcBot.Bubble.Townsfolk
    pub stop_with_client: bool,
    pub stop_database_too: bool,
    pub era: bool,             // the "Era" preset: half XP, no dungeon-finder teleport, quest map areas off, vanilla/TBC riding
    pub unload_grids: bool,    // SoloWoW.UnloadIdleGrids: let go of map areas nobody is near
    pub account: String,       // the account Play fills in
    pub client: String,        // the client's folder ("" = look in the usual places)
}

fn world_conf() -> PathBuf { server_dir().join("configs").join("worldserver.conf") }
fn launcher_conf() -> PathBuf { root().join("launcher.cfg") }

fn conf_get(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let l = l.trim_start();
        if l.starts_with('#') { return None; }
        let (k, v) = l.split_once('=')?;
        (k.trim() == key).then(|| v.trim().trim_matches('"').to_string())
    })
}

fn conf_set(text: &str, key: &str, value: &str) -> String {
    let mut found = false;
    let mut out: Vec<String> = text.lines().map(|l| match l.trim_start().split_once('=') {
        Some((k, _)) if !l.trim_start().starts_with('#') && k.trim() == key => { found = true; format!("{key} = {value}") }
        _ => l.to_string(),
    }).collect();
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
        stop_database_too: conf_get(&mine, "StopDatabase").map(|v| v != "0").unwrap_or(true),
        era: conf_get(&conf, "Rate.XP.Kill").map(|v| v.parse::<f32>().map(|x| x < 0.99).unwrap_or(true)).unwrap_or(true),
        unload_grids: conf_get(&conf, "SoloWoW.UnloadIdleGrids").map(|v| v != "0").unwrap_or(true),
        account: conf_get(&mine, "Account").unwrap_or_default(),
        client: conf_get(&mine, "Client").unwrap_or_default(),
    }
}

/// Saves settings. World settings take effect the next time the world starts. Switching the Era preset also changes
/// the world database (scripts\era-db.ps1, reversible - run hidden), which starts MySQL if it has to.
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
        start_database()?;
        let mut cmd = Command::new("powershell");
        hidden(&mut cmd).args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]).arg(root().join("scripts").join("era-db.ps1"));
        if !s.era { cmd.arg("-Off"); }
        let out = cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).output().map_err(|e| format!("could not switch the Era preset: {e}"))?;
        if !out.status.success() { return Err("the Era database change failed".into()); }
    }
    fs::write(&path, conf).map_err(|e| format!("can't write worldserver.conf: {e}"))?;
    let mut mine = fs::read_to_string(launcher_conf()).unwrap_or_default();
    mine = conf_set(&mine, "StopWithClient", if s.stop_with_client { "1" } else { "0" });
    mine = conf_set(&mine, "StopDatabase", if s.stop_database_too { "1" } else { "0" });
    mine = conf_set(&mine, "Account", &s.account);
    mine = conf_set(&mine, "Client", &s.client);
    fs::write(launcher_conf(), mine).map_err(|e| e.to_string())
}

// ============================================================================================ the client's graphics

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

// ============================================================================================ one launcher at a time

#[cfg(windows)]
pub fn single_instance() -> bool {
    extern "system" {
        fn CreateMutexW(attrs: *mut core::ffi::c_void, owner: i32, name: *const u16) -> *mut core::ffi::c_void;
        fn GetLastError() -> u32;
    }
    let name: Vec<u16> = "Local\\SoloWoWLauncher\0".encode_utf16().collect();
    // the handle is kept for the life of the process on purpose
    unsafe { let h = CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr()); !h.is_null() && GetLastError() != 183 }
}
#[cfg(not(windows))]
pub fn single_instance() -> bool { true }

pub fn fmt_mem(kb: u64) -> String { if kb >= 1_048_576 { format!("{:.2} GB", kb as f64 / 1_048_576.0) } else { format!("{} MB", kb / 1024) } }
pub fn fmt_uptime(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 86400 { format!("{}d {}h", s / 86400, s % 86400 / 3600) } else { format!("{:02}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60) }
}
pub fn fmt_age(t: SystemTime) -> String {
    let s = SystemTime::now().duration_since(t).map(|d| d.as_secs()).unwrap_or(0);
    if s < 3600 { format!("{} min ago", s / 60) } else if s < 86400 { format!("{} h ago", s / 3600) } else { format!("{} days ago", s / 86400) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn b64() { assert_eq!(base64(b"SOLO:solo"), "U09MTzpzb2xv"); }
    #[test]
    fn sha1_known() {
        assert_eq!(hex(&sha1(b"abc")), "A9993E364706816ABA3E25717850C26C9CD0D89D");
        assert_eq!(hex(&sha1(b"")), "DA39A3EE5E6B4B0D3255BFEF95601890AFD80709");
    }
    #[test]
    fn srp_matches_the_login_server() {
        // the SOLO account as the old scripts made it (salt and verifier from acore_auth)
        let salt_hex = "6E60969CCEF24D8D0777E231E1B0565F26685BD10FEE743D360D8F695A9168CE";
        let mut salt = [0u8; 32];
        for i in 0..32 { salt[i] = u8::from_str_radix(&salt_hex[2 * i..2 * i + 2], 16).unwrap(); }
        assert_eq!(hex(&srp_verifier("SOLO", "solo", &salt)), "946098702607ACFB6B43BE20FA6D23BA23A2235B0F7AD0487374A5CDE4E90B82");
    }
    #[test]
    fn conf() {
        let t = "# x\nNpcBot.Bubble.Count = 40\n";
        assert_eq!(conf_get(t, "NpcBot.Bubble.Count").as_deref(), Some("40"));
        assert!(conf_set(t, "NpcBot.Bubble.Count", "25").contains("NpcBot.Bubble.Count = 25"));
        assert!(conf_set(t, "New.Key", "1").contains("New.Key = 1"));
    }
    #[test]
    fn names() {
        assert!(valid_account_name("Arkada64").is_ok());
        assert!(valid_account_name("bad name").is_err());
        assert!(valid_account_name("").is_err());
    }
    #[test]
    fn stamps() { assert_eq!(stamp().len(), 13); }
}
