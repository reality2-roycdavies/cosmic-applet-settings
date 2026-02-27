use crate::pages::Page;
use std::fs;
use std::path::PathBuf;

/// Returns the list of pages whose applets are currently active on the COSMIC panel or dock.
pub fn detect_active_pages() -> Vec<Page> {
    let combined = read_panel_configs();

    Page::ALL
        .iter()
        .copied()
        .filter(|page| combined.contains(page.applet_id()))
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
