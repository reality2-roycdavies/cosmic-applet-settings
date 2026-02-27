use cosmic::app::Core;
use cosmic::iced::widget::{column, container, horizontal_space, row, scrollable};
use cosmic::iced::{Alignment, Background, Length};
use cosmic::widget::button::{self, Style as ButtonStyle};
use cosmic::widget::{icon, text};
use cosmic::{Action, Application, Element, Task};
use serde::Deserialize;
use std::process::Command;

const APP_ID: &str = "io.github.reality2_roycdavies.cosmic-applet-settings";

/// A registered applet discovered from JSON files in the registry directory.
#[derive(Debug, Clone, Deserialize)]
pub struct AppletEntry {
    pub name: String,
    pub icon: String,
    pub applet_id: String,
    pub settings_cmd: String,
}

pub struct AppFlags {
    pub initial_applet_id: Option<String>,
    pub active_applets: Vec<AppletEntry>,
}

pub struct SettingsApp {
    core: Core,
    selected: usize,
    applets: Vec<AppletEntry>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectApplet(usize),
    OpenSettings(usize),
}

impl Application for SettingsApp {
    type Executor = cosmic::executor::Default;
    type Flags = AppFlags;
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        let selected = flags
            .initial_applet_id
            .as_ref()
            .and_then(|id| {
                flags
                    .active_applets
                    .iter()
                    .position(|a| a.applet_id == *id)
            })
            .unwrap_or(0);

        let app = Self {
            core,
            selected,
            applets: flags.active_applets,
        };

        (app, Task::none())
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![text::heading("Applet Settings").into()]
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if self.applets.is_empty() {
            return container(
                column![
                    text::title3("No Applets Detected"),
                    text::body(
                        "No custom applets are currently active on the panel or dock.\n\
                         Add an applet to the panel or dock, then reopen this settings app."
                    ),
                ]
                .spacing(12)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(32)
            .into();
        }

        let sidebar = self.sidebar_view();
        let page_content = self.page_view();

        row![
            container(sidebar).padding([8, 8, 8, 8]),
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
            Message::SelectApplet(idx) => {
                self.selected = idx;
            }
            Message::OpenSettings(idx) => {
                if let Some(entry) = self.applets.get(idx) {
                    launch_settings_cmd(&entry.settings_cmd);
                }
            }
        }
        Task::none()
    }
}

impl SettingsApp {
    fn sidebar_view(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::spacing();
        let space_xxs = spacing.space_xxs;
        let space_s = spacing.space_s;

        let mut sidebar_items = column![].spacing(space_xxs).padding(space_xxs);

        for (idx, entry) in self.applets.iter().enumerate() {
            let is_active = idx == self.selected;

            let item_content = row![
                icon::from_name(entry.icon.as_str()).size(20).symbolic(true),
                text::body(&entry.name),
                horizontal_space(),
            ]
            .spacing(space_xxs)
            .align_y(Alignment::Center);

            let btn = button::custom(item_content)
                .on_press(Message::SelectApplet(idx))
                .class(if is_active {
                    nav_active_style()
                } else {
                    nav_inactive_style()
                });

            sidebar_items = sidebar_items.push(
                btn.width(Length::Fill)
                    .padding([space_xxs, space_s, space_xxs, space_s]),
            );
        }

        container(sidebar_items)
            .width(Length::Fixed(260.0))
            .height(Length::Fill)
            .style(|theme: &cosmic::Theme| {
                let cosmic = theme.cosmic();
                container::Style {
                    background: Some(cosmic::iced::Background::Color(
                        cosmic::iced::Color::from(cosmic.primary_container_color()),
                    )),
                    border: cosmic::iced::Border {
                        radius: cosmic.corner_radii.radius_s.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .into()
    }

    fn page_view(&self) -> Element<'_, Message> {
        let Some(entry) = self.applets.get(self.selected) else {
            return text::body("No applet selected.").into();
        };

        let spacing = cosmic::theme::spacing();

        column![
            row![
                icon::from_name(entry.icon.as_str()).size(48).symbolic(true),
                column![
                    text::title3(&entry.name),
                    text::caption(&entry.applet_id),
                ]
                .spacing(4),
            ]
            .spacing(spacing.space_s)
            .align_y(Alignment::Center),
            cosmic::widget::button::standard("Open Settings")
                .on_press(Message::OpenSettings(self.selected)),
        ]
        .spacing(spacing.space_m)
        .into()
    }
}

fn launch_settings_cmd(cmd: &str) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if let Some((program, args)) = parts.split_first() {
        let _ = Command::new(program).args(args).spawn();
    }
}

pub fn run_app(flags: AppFlags) -> cosmic::iced::Result {
    let settings = cosmic::app::Settings::default()
        .size(cosmic::iced::Size::new(900.0, 700.0))
        .size_limits(
            cosmic::iced::Limits::NONE
                .min_width(600.0)
                .min_height(450.0),
        );
    cosmic::app::run::<SettingsApp>(settings, flags)
}

fn nav_active_style() -> cosmic::theme::Button {
    fn style(focused: bool, theme: &cosmic::Theme) -> ButtonStyle {
        let cosmic = theme.cosmic();
        let mut s = ButtonStyle::new();
        s.background = Some(Background::Color(
            cosmic.primary.component.hover.into(),
        ));
        s.text_color = Some(cosmic.accent_text_color().into());
        s.icon_color = Some(cosmic.accent_text_color().into());
        s.border_radius = cosmic.corner_radii.radius_xl.into();
        if focused {
            s.outline_width = 1.0;
            s.outline_color = cosmic.accent.base.into();
            s.border_width = 2.0;
            s.border_color = cosmic::iced::Color::TRANSPARENT;
        }
        s
    }
    cosmic::theme::Button::Custom {
        active: Box::new(style),
        disabled: Box::new(|theme| style(false, theme)),
        hovered: Box::new(style),
        pressed: Box::new(style),
    }
}

fn nav_inactive_style() -> cosmic::theme::Button {
    fn style(focused: bool, theme: &cosmic::Theme) -> ButtonStyle {
        let cosmic = theme.cosmic();
        let mut s = ButtonStyle::new();
        s.background = None;
        s.text_color = Some(cosmic.background.on.into());
        s.icon_color = Some(cosmic.background.on.into());
        s.border_radius = cosmic.corner_radii.radius_xl.into();
        if focused {
            s.outline_width = 1.0;
            s.outline_color = cosmic.accent.base.into();
            s.border_width = 2.0;
            s.border_color = cosmic::iced::Color::TRANSPARENT;
        }
        s
    }
    fn hovered(focused: bool, theme: &cosmic::Theme) -> ButtonStyle {
        let cosmic = theme.cosmic();
        let mut s = style(focused, theme);
        s.background = Some(Background::Color(
            cosmic.primary.component.base.into(),
        ));
        s
    }
    cosmic::theme::Button::Custom {
        active: Box::new(style),
        disabled: Box::new(|theme| style(false, theme)),
        hovered: Box::new(hovered),
        pressed: Box::new(hovered),
    }
}
