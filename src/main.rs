use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_FPS: u32 = 144;

#[derive(Debug)]
struct Args {
    fps: u32,
    revert: bool,
    detect: bool,
    launch: bool,
    ini_dir: Option<PathBuf>,
    game_dir: Option<PathBuf>,
    yes: bool,
}

fn parse_args() -> Args {
    let mut a = Args {
        fps: DEFAULT_FPS,
        revert: false,
        detect: false,
        launch: false,
        ini_dir: None,
        game_dir: None,
        yes: false,
    };
    let mut it = env::args().skip(1);
    while let Some(tok) = it.next() {
        match tok.as_str() {
            "--fps" => {
                if let Some(v) = it.next() {
                    a.fps = v.parse().unwrap_or(DEFAULT_FPS);
                }
            }
            "--revert" => a.revert = true,
            "--detect" => a.detect = true,
            "--launch" => a.launch = true,
            "--yes" | "-y" => a.yes = true,
            "--ini-dir" => {
                if let Some(v) = it.next() {
                    a.ini_dir = Some(PathBuf::from(v));
                }
            }
            "--game-dir" => {
                if let Some(v) = it.next() {
                    a.game_dir = Some(PathBuf::from(v));
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
    }
    if a.fps < 30 {
        a.fps = 30;
    }
    if a.fps > 1000 {
        a.fps = 1000;
    }
    a
}

fn print_help() {
    println!("Skyrim FPS Unlocker");
    println!();
    println!("Usage: skyrim-fps-unlocker [options]");
    println!();
    println!("Options:");
    println!("  --fps <n>       Target framerate (default: 144). Used for Havok fMaxTime.");
    println!("  --revert        Restore ini files from backups.");
    println!("  --detect        Print current ini state and exit.");
    println!("  --launch        Launch SkyrimSE.exe after applying tweaks.");
    println!("  --ini-dir <p>   Directory containing Skyrim.ini / SkyrimPrefs.ini.");
    println!("  --game-dir <p>  Skyrim SE install directory (for --launch).");
    println!("  --yes, -y       Skip confirmation prompt.");
    println!("  --help, -h      Show this help.");
    println!();
    println!("Ini files are located in:");
    println!("  %USERPROFILE%\\Documents\\My Games\\Skyrim Special Edition\\");
}

fn ini_dir(args: &Args) -> PathBuf {
    if let Some(d) = &args.ini_dir {
        return d.clone();
    }
    let home = env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Documents")
        .join("My Games")
        .join("Skyrim Special Edition")
}

fn game_dir(args: &Args) -> PathBuf {
    if let Some(d) = &args.game_dir {
        return d.clone();
    }
    // Common Steam install locations
    let candidates = [
        "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Skyrim Special Edition",
        "C:\\Program Files\\Steam\\steamapps\\common\\Skyrim Special Edition",
        "D:\\Steam\\steamapps\\common\\Skyrim Special Edition",
        "D:\\SteamLibrary\\steamapps\\common\\Skyrim Special Edition",
        "E:\\SteamLibrary\\steamapps\\common\\Skyrim Special Edition",
    ];
    for c in candidates {
        if Path::new(c).join("SkyrimSE.exe").exists() {
            return PathBuf::from(c);
        }
    }
    PathBuf::from("C:\\Program Files (x86)\\Steam\\steamapps\\common\\Skyrim Special Edition")
}

// ---------- INI helpers ----------

fn read_lines(path: &Path) -> Vec<String> {
    match fs::read_to_string(path) {
        Ok(s) => s.lines().map(|l| l.to_string()).collect(),
        Err(_) => Vec::new(),
    }
}

fn write_lines(path: &Path, lines: &[String]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut f = fs::File::create(path)?;
    for l in lines {
        writeln!(f, "{}", l)?;
    }
    Ok(())
}

/// Ensure a key=value exists in the given section. If section missing, append it.
fn set_ini(path: &Path, section: &str, key: &str, value: &str) {
    let mut lines = read_lines(path);
    let mut key_idx: Option<usize> = None;
    // find section range
    let mut start = None;
    let mut end = lines.len();
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim();
        if t == format!("[{}]", section) {
            start = Some(i);
        } else if start.is_some() && t.starts_with('[') && t.ends_with(']') {
            end = i;
            break;
        }
    }
    if let Some(s) = start {
        // search key within [s+1, end)
        for i in (s + 1)..end {
            let t = lines[i].trim();
            if t.is_empty() || t.starts_with(';') || t.starts_with('#') {
                continue;
            }
            if let Some(eq) = t.find('=') {
                let k = t[..eq].trim();
                if k.eq_ignore_ascii_case(key) {
                    key_idx = Some(i);
                    break;
                }
            }
        }
        if let Some(i) = key_idx {
            lines[i] = format!("{}={}", key, value);
        } else {
            // insert before end (or at end of section)
            lines.insert(end, format!("{}={}", key, value));
        }
    } else {
        // append section
        if !lines.is_empty() && !lines.last().unwrap().trim().is_empty() {
            lines.push(String::new());
        }
        lines.push(format!("[{}]", section));
        lines.push(format!("{}={}", key, value));
    }
    let _ = write_lines(path, &lines);
}

fn get_ini(path: &Path, section: &str, key: &str) -> Option<String> {
    let lines = read_lines(path);
    let mut in_sec = false;
    for l in lines {
        let t = l.trim();
        if t.starts_with('[') && t.ends_with(']') {
            in_sec = t == format!("[{}]", section);
            continue;
        }
        if in_sec {
            if let Some(eq) = t.find('=') {
                let k = t[..eq].trim();
                if k.eq_ignore_ascii_case(key) {
                    return Some(t[eq + 1..].trim().to_string());
                }
            }
        }
    }
    None
}

fn backup(path: &Path) {
    let bak = PathBuf::from(format!("{}.bak", path.display()));
    if path.exists() && !bak.exists() {
        let _ = fs::copy(path, &bak);
    }
}

fn restore(path: &Path) -> bool {
    let bak = PathBuf::from(format!("{}.bak", path.display()));
    if bak.exists() {
        let _ = fs::copy(&bak, path);
        let _ = fs::remove_file(&bak);
        true
    } else {
        false
    }
}

fn fmt_f(v: f64) -> String {
    // trim trailing zeros but keep reasonable precision
    let s = format!("{:.6}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    if s.is_empty() { "0".to_string() } else { s }
}

fn detect_state(dir: &Path) {
    let skyrim_ini = dir.join("Skyrim.ini");
    let prefs_ini = dir.join("SkyrimPrefs.ini");
    println!("Ini directory: {}", dir.display());
    println!();
    println!("Skyrim.ini exists: {}", skyrim_ini.exists());
    if skyrim_ini.exists() {
        println!("  [Display] bLockFramerate = {}", get_ini(&skyrim_ini, "Display", "bLockFramerate").unwrap_or_else(|| "<unset>".into()));
        println!("  [Display] iVSyncPresentInterval = {}", get_ini(&skyrim_ini, "Display", "iVSyncPresentInterval").unwrap_or_else(|| "<unset>".into()));
        println!("  [HAVOK] fMaxTime = {}", get_ini(&skyrim_ini, "HAVOK", "fMaxTime").unwrap_or_else(|| "<unset>".into()));
        println!("  [HAVOK] fMaxTimeComplex = {}", get_ini(&skyrim_ini, "HAVOK", "fMaxTimeComplex").unwrap_or_else(|| "<unset>".into()));
    }
    println!();
    println!("SkyrimPrefs.ini exists: {}", prefs_ini.exists());
    if prefs_ini.exists() {
        println!("  [Display] iVSyncPresentInterval = {}", get_ini(&prefs_ini, "Display", "iVSyncPresentInterval").unwrap_or_else(|| "<unset>".into()));
    }
}

fn apply(dir: &Path, fps: u32) {
    let skyrim_ini = dir.join("Skyrim.ini");
    let prefs_ini = dir.join("SkyrimPrefs.ini");

    // Ensure ini dir & files exist (Skyrim creates them on first run; create minimal if missing)
    if !dir.exists() {
        let _ = fs::create_dir_all(dir);
    }
    if !skyrim_ini.exists() {
        let _ = write_lines(&skyrim_ini, &Vec::new());
    }
    if !prefs_ini.exists() {
        let _ = write_lines(&prefs_ini, &Vec::new());
    }

    backup(&skyrim_ini);
    backup(&prefs_ini);

    let inv = 1.0 / fps as f64;
    let inv_complex = 2.0 / fps as f64;

    set_ini(&skyrim_ini, "Display", "bLockFramerate", "0");
    set_ini(&skyrim_ini, "Display", "iVSyncPresentInterval", "0");
    set_ini(&skyrim_ini, "HAVOK", "fMaxTime", &fmt_f(inv));
    set_ini(&skyrim_ini, "HAVOK", "fMaxTimeComplex", &fmt_f(inv_complex));
    set_ini(&prefs_ini, "Display", "iVSyncPresentInterval", "0");

    println!("Applied FPS unlock for target {} FPS.", fps);
    println!("  Skyrim.ini: bLockFramerate=0, iVSyncPresentInterval=0, fMaxTime={}, fMaxTimeComplex={}",
        fmt_f(inv), fmt_f(inv_complex));
    println!("  SkyrimPrefs.ini: iVSyncPresentInterval=0");
    println!("Backups saved as Skyrim.ini.bak / SkyrimPrefs.ini.bak");
}

fn revert(dir: &Path) {
    let skyrim_ini = dir.join("Skyrim.ini");
    let prefs_ini = dir.join("SkyrimPrefs.ini");
    let mut ok = false;
    if restore(&skyrim_ini) { ok = true; println!("Restored Skyrim.ini"); }
    if restore(&prefs_ini) { ok = true; println!("Restored SkyrimPrefs.ini"); }
    if !ok {
        println!("No backups found to restore.");
    }
}

fn launch(game: &Path) {
    let exe = game.join("SkyrimSE.exe");
    if !exe.exists() {
        println!("SkyrimSE.exe not found in {}", game.display());
        return;
    }
    println!("Launching {}", exe.display());
    let _ = Command::new(&exe).spawn();
}

fn confirm(msg: &str) -> bool {
    print!("{}", msg);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
    let t = buf.trim().to_lowercase();
    t == "y" || t == "yes"
}

fn main() {
    let args = parse_args();

    if args.detect {
        detect_state(&ini_dir(&args));
        return;
    }

    if args.revert {
        revert(&ini_dir(&args));
        return;
    }

    let dir = ini_dir(&args);
    if !args.yes {
        println!("This will modify ini files in: {}", dir.display());
        println!("Target FPS: {}", args.fps);
        if !confirm("Continue? [y/N] ") {
            println!("Aborted.");
            return;
        }
    }

    apply(&dir, args.fps);

    if args.launch {
        launch(&game_dir(&args));
    }
}