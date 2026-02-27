use crate::app::AppletEntry;
use std::fs;
use std::path::PathBuf;

/// Returns only the applets whose applet_id appears in the COSMIC panel or dock config.
pub fn filter_active_applets(applets: &[AppletEntry]) -> Vec<AppletEntry> {
    let combined = read_panel_configs();

    applets
        .iter()
        .filter(|entry| combined.contains(&entry.applet_id))
        .cloned()
        .collect()
}

fn read_panel_configs() -> String {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => return String::new(),
    };

    let paths: Vec<PathBuf> = [
        "cosmic/com.system76.CosmicPanel.Panel/v1/plugins_center",
        "cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings",
        "cosmic/com.system76.CosmicPanel.Dock/v1/plugins_center",
        "cosmic/com.system76.CosmicPanel.Dock/v1/plugins_wings",
    ]
    .iter()
    .map(|p| config_dir.join(p))
    .collect();

    let mut combined = String::new();
    for path in paths {
        if let Ok(contents) = fs::read_to_string(&path) {
            combined.push_str(&contents);
            combined.push('\n');
        }
    }
    combined
}
