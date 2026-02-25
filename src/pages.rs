#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Tailscale,
    RunKat,
    BingWallpaper,
    PieMenu,
    Hotspot,
}

impl Page {
    pub const ALL: &'static [Page] = &[
        Page::Tailscale,
        Page::RunKat,
        Page::BingWallpaper,
        Page::PieMenu,
        Page::Hotspot,
    ];

    pub fn cli_name(&self) -> &'static str {
        match self {
            Page::Tailscale => "tailscale",
            Page::RunKat => "runkat",
            Page::BingWallpaper => "bing-wallpaper",
            Page::PieMenu => "pie-menu",
            Page::Hotspot => "hotspot",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Page::Tailscale => "Tailscale VPN",
            Page::RunKat => "RunKat",
            Page::BingWallpaper => "Bing Wallpaper",
            Page::PieMenu => "Pie Menu",
            Page::Hotspot => "WiFi Hotspot",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Page::Tailscale => "network-vpn-symbolic",
            Page::RunKat => "utilities-system-monitor-symbolic",
            Page::BingWallpaper => "preferences-desktop-wallpaper-symbolic",
            Page::PieMenu => "input-touchpad-symbolic",
            Page::Hotspot => "network-wireless-hotspot-symbolic",
        }
    }

    pub fn from_cli_name(s: &str) -> Option<Page> {
        match s {
            "tailscale" => Some(Page::Tailscale),
            "runkat" => Some(Page::RunKat),
            "bing-wallpaper" => Some(Page::BingWallpaper),
            "pie-menu" => Some(Page::PieMenu),
            "hotspot" => Some(Page::Hotspot),
            _ => None,
        }
    }
}
