use cosmic::app::Core;
use cosmic::iced::widget::{column, container, horizontal_space, row, scrollable, Space};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, icon, text};
use cosmic::{Action, Application, Element, Task};

use cosmic_tailscale::settings_page as tailscale_page;
use cosmic_runkat::settings_page as runkat_page;
use cosmic_bing_wallpaper::settings_page as bing_wallpaper_page;
use cosmic_pie_menu::settings_page as pie_menu_page;
use cosmic_hotspot::settings_page as hotspot_page;

use crate::pages::Page;

const APP_ID: &str = "io.github.reality2_roycdavies.cosmic-applet-settings";

pub struct SettingsApp {
    core: Core,
    active_page: Page,
    tailscale: Option<tailscale_page::State>,
    runkat: Option<runkat_page::State>,
    bing_wallpaper: Option<bing_wallpaper_page::State>,
    pie_menu: Option<pie_menu_page::State>,
    hotspot: Option<hotspot_page::State>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectPage(Page),
    Tailscale(tailscale_page::Message),
    RunKat(runkat_page::Message),
    BingWallpaper(bing_wallpaper_page::Message),
    PieMenu(pie_menu_page::Message),
    Hotspot(hotspot_page::Message),
}

impl Application for SettingsApp {
    type Executor = cosmic::executor::Default;
    type Flags = Page;
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, initial_page: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        let app = Self {
            core,
            active_page: initial_page,
            tailscale: None,
            runkat: None,
            bing_wallpaper: None,
            pie_menu: None,
            hotspot: None,
        };

        // Eagerly init the initial page
        let mut app = app;
        app.ensure_page_init(initial_page);

        (app, Task::none())
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![text::heading("Applet Settings").into()]
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let sidebar = self.sidebar_view();
        let page_content = self.page_view();

        let divider = container(Space::new(Length::Fixed(1.0), Length::Fill)).style(
            |theme: &cosmic::Theme| {
                let cosmic = theme.cosmic();
                container::Style {
                    background: Some(cosmic::iced::Background::Color(cosmic::iced::Color::from(
                        cosmic.palette.neutral_5,
                    ))),
                    ..Default::default()
                }
            },
        );

        row![
            sidebar,
            divider,
            scrollable(
                container(container(page_content).max_width(800))
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                    .padding(16),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Action<Self::Message>> {
        match message {
            Message::SelectPage(page) => {
                self.active_page = page;
                self.ensure_page_init(page);
            }
            Message::Tailscale(msg) => {
                if let Some(ref mut state) = self.tailscale {
                    tailscale_page::update(state, msg);
                }
            }
            Message::RunKat(msg) => {
                if let Some(ref mut state) = self.runkat {
                    runkat_page::update(state, msg);
                }
            }
            Message::BingWallpaper(msg) => {
                if let Some(ref mut state) = self.bing_wallpaper {
                    bing_wallpaper_page::update(state, msg);
                }
            }
            Message::PieMenu(msg) => {
                if let Some(ref mut state) = self.pie_menu {
                    pie_menu_page::update(state, msg);
                }
            }
            Message::Hotspot(msg) => {
                if let Some(ref mut state) = self.hotspot {
                    hotspot_page::update(state, msg);
                }
            }
        }
        Task::none()
    }
}

impl SettingsApp {
    fn ensure_page_init(&mut self, page: Page) {
        match page {
            Page::Tailscale => {
                if self.tailscale.is_none() {
                    self.tailscale = Some(tailscale_page::init());
                }
            }
            Page::RunKat => {
                if self.runkat.is_none() {
                    self.runkat = Some(runkat_page::init());
                }
            }
            Page::BingWallpaper => {
                if self.bing_wallpaper.is_none() {
                    self.bing_wallpaper = Some(bing_wallpaper_page::init());
                }
            }
            Page::PieMenu => {
                if self.pie_menu.is_none() {
                    self.pie_menu = Some(pie_menu_page::init());
                }
            }
            Page::Hotspot => {
                if self.hotspot.is_none() {
                    self.hotspot = Some(hotspot_page::init());
                }
            }
        }
    }

    fn sidebar_view(&self) -> Element<'_, Message> {
        let mut sidebar_items = column![].spacing(4).padding(8);

        for &page in Page::ALL {
            let is_active = page == self.active_page;

            let item_content = row![
                icon::from_name(page.icon_name()).size(20).symbolic(true),
                text::body(page.title()),
                horizontal_space(),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            let btn = if is_active {
                button::custom(item_content)
                    .on_press(Message::SelectPage(page))
                    .class(cosmic::theme::Button::Suggested)
            } else {
                button::custom(item_content)
                    .on_press(Message::SelectPage(page))
                    .class(cosmic::theme::Button::MenuItem)
            };

            sidebar_items = sidebar_items.push(btn.width(Length::Fill).padding([8, 12]));
        }

        container(sidebar_items)
            .width(Length::Fixed(240.0))
            .height(Length::Fill)
            .into()
    }

    fn page_view(&self) -> Element<'_, Message> {
        match self.active_page {
            Page::Tailscale => {
                if let Some(ref state) = self.tailscale {
                    tailscale_page::view(state).map(Message::Tailscale)
                } else {
                    text::body("Loading...").into()
                }
            }
            Page::RunKat => {
                if let Some(ref state) = self.runkat {
                    runkat_page::view(state).map(Message::RunKat)
                } else {
                    text::body("Loading...").into()
                }
            }
            Page::BingWallpaper => {
                if let Some(ref state) = self.bing_wallpaper {
                    bing_wallpaper_page::view(state).map(Message::BingWallpaper)
                } else {
                    text::body("Loading...").into()
                }
            }
            Page::PieMenu => {
                if let Some(ref state) = self.pie_menu {
                    pie_menu_page::view(state).map(Message::PieMenu)
                } else {
                    text::body("Loading...").into()
                }
            }
            Page::Hotspot => {
                if let Some(ref state) = self.hotspot {
                    hotspot_page::view(state).map(Message::Hotspot)
                } else {
                    text::body("Loading...").into()
                }
            }
        }
    }
}

pub fn run_app(initial_page: Page) -> cosmic::iced::Result {
    let settings = cosmic::app::Settings::default()
        .size(cosmic::iced::Size::new(900.0, 700.0))
        .size_limits(
            cosmic::iced::Limits::NONE
                .min_width(600.0)
                .min_height(450.0),
        );
    cosmic::app::run::<SettingsApp>(settings, initial_page)
}
