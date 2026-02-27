mod app;
mod detection;

use app::{AppFlags, AppletEntry};
use std::fs;
use std::path::PathBuf;

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

    let all_applets = scan_registry();
    let active_applets = detection::filter_active_applets(&all_applets);

    // If an applet_id was given on the command line, auto-select it.
    // If the requested applet is registered but not on the panel, include it anyway.
    let initial_applet_id = args.get(1).filter(|a| !a.starts_with('-')).cloned();

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
