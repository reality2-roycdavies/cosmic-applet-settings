mod app;
mod detection;

use app::{AppFlags, AppletEntry};
use std::fs;
use std::io::{Read as _, Write as _};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

fn main() -> cosmic::iced::Result {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help(&args[0]);
                return Ok(());
            }
            "--version" | "-v" => {
                println!("cosmic-applet-settings {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
    }

    let initial_applet_id = args.get(1).filter(|a| !a.starts_with('-')).cloned();
    let sock = socket_path();

    // Try to connect to an existing instance
    if let Ok(mut stream) = UnixStream::connect(&sock) {
        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let msg = initial_applet_id.as_deref().unwrap_or("");
        let _ = stream.write_all(msg.as_bytes());
        let _ = stream.shutdown(std::net::Shutdown::Write);
        // Wait for acknowledgment from the existing instance
        let mut buf = [0u8; 1];
        let _ = stream.read(&mut buf);
        return Ok(());
    }

    // No existing instance — clean up stale socket and proceed
    let _ = fs::remove_file(&sock);

    let all_applets = scan_registry();
    let active_applets = detection::filter_active_applets(&all_applets);

    // If an applet_id was given on the command line, auto-select it.
    // If the requested applet is registered but not on the panel, include it anyway.
    let mut applets = active_applets;
    if let Some(ref id) = initial_applet_id {
        if !applets.iter().any(|a| a.applet_id == *id) {
            if let Some(entry) = all_applets.iter().find(|a| a.applet_id == *id) {
                applets.insert(0, entry.clone());
            } else {
                eprintln!("Unknown applet: {id}");
                eprintln!("Registered applets:");
                for a in &all_applets {
                    eprintln!("  {}", a.applet_id);
                }
                std::process::exit(1);
            }
        }
    }

    app::run_app(AppFlags {
        initial_applet_id,
        active_applets: applets,
        socket_path: sock,
    })
}

/// Scan the registry directory for applet registration JSON files.
fn scan_registry() -> Vec<AppletEntry> {
    let registry_dir = registry_dir();
    let mut entries = Vec::new();

    let read_dir = match fs::read_dir(&registry_dir) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };

    for dir_entry in read_dir.flatten() {
        let path = dir_entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            match fs::read_to_string(&path) {
                Ok(contents) => match serde_json::from_str::<AppletEntry>(&contents) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => eprintln!("Warning: invalid registration {}: {e}", path.display()),
                },
                Err(e) => eprintln!("Warning: cannot read {}: {e}", path.display()),
            }
        }
    }

    // Sort by name for consistent ordering
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

fn socket_path() -> PathBuf {
    let runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("cosmic-applet-settings.sock")
}

fn registry_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("cosmic-applet-settings/applets")
}

fn print_help(program: &str) {
    println!("Unified settings hub for custom COSMIC applets\n");
    println!("Usage: {} [APPLET_ID]\n", program);
    println!("If APPLET_ID is given, that applet's settings are launched.");
    println!("Otherwise, the first active applet is shown.\n");
    println!("Applets self-register by placing JSON descriptors in:");
    println!("  ~/.local/share/cosmic-applet-settings/applets/\n");
    println!("Options:");
    println!("  --version, -v      Show version information");
    println!("  --help, -h         Show this help message");
}
