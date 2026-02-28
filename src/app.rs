use cosmic::app::Core;
use cosmic::iced::widget::{column, container, horizontal_space, row, scrollable};
use cosmic::iced::{Alignment, Background, ContentFit, Length, Subscription};
use cosmic::widget::button::{self, Style as ButtonStyle};
use cosmic::widget::{self, icon, settings, text, toggler};
use cosmic::{Action, Application, Element, Task};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{Read as _, Write as _};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const APP_ID: &str = "io.github.reality2_roycdavies.cosmic-applet-settings";

// ---------------------------------------------------------------------------
// Registry types
// ---------------------------------------------------------------------------

/// A registered applet discovered from JSON files in the registry directory.
#[derive(Debug, Clone, Deserialize)]
pub struct AppletEntry {
    pub name: String,
    pub icon: String,
    pub applet_id: String,
    pub settings_cmd: String,
}

// ---------------------------------------------------------------------------
// Schema types (parsed from --settings-describe JSON)
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, Debug)]
pub struct SettingsSchema {
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub refresh_interval: Option<u64>,
    pub sections: Vec<SchemaSection>,
    #[serde(default)]
    pub actions: Vec<SchemaAction>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SchemaSection {
    pub title: String,
    pub items: Vec<SchemaItem>,
    #[serde(default)]
    pub actions: Vec<SchemaAction>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SchemaItem {
    #[serde(rename = "type")]
    pub item_type: String,
    pub key: String,
    #[serde(default)]
    pub label: String,
    pub value: serde_json::Value,
    #[serde(default)]
    pub options: Option<Vec<SelectOption>>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub visible_when: Option<Condition>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub list_items: Option<Vec<ListItem>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ListItem {
    pub id: String,
    #[serde(default)]
    pub image: Option<String>,
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub actions: Vec<SchemaAction>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Condition {
    pub key: String,
    pub equals: serde_json::Value,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SchemaAction {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub confirm: Option<String>,
}

// ---------------------------------------------------------------------------
// Application state & messages
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct PendingConfirm {
    action_id: String,
    item_id: Option<String>,
    message: String,
}

pub struct AppFlags {
    pub initial_applet_id: Option<String>,
    pub active_applets: Vec<AppletEntry>,
    pub socket_path: PathBuf,
}

pub struct SettingsApp {
    core: Core,
    selected: usize,
    applets: Vec<AppletEntry>,
    ipc_rx: std::sync::mpsc::Receiver<String>,
    // Inline settings state
    current_schema: Option<SettingsSchema>,
    schema_loading: bool,
    schema_error: Option<String>,
    local_values: HashMap<String, serde_json::Value>,
    text_edits: HashMap<String, String>,
    status_message: String,
    last_loaded_idx: Option<usize>,
    // Pre-computed display data for widgets that borrow (dropdown, text_input)
    dropdown_labels: Vec<Vec<String>>,
    text_displays: Vec<String>,
    // Confirmation state for destructive actions
    pending_confirm: Option<PendingConfirm>,
    // Debounce: auto-commit text edits after a pause in typing
    last_text_edit_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectApplet(usize),
    OpenSettings(usize),
    CheckIpc,
    // Inline settings protocol
    LoadSchema(usize),
    SchemaLoaded(usize, Result<SettingsSchema, String>),
    RefreshSchema,
    RefreshSchemaLoaded(Result<SettingsSchema, String>),
    SettingChanged(String, serde_json::Value),
    SettingApplied(Result<String, String>),
    ActionTriggered(String),
    ItemActionTriggered(String, String),
    ConfirmAction,
    CancelConfirm,
    ActionCompleted(Result<String, String>),
    TextEditing(String, String),
    TextUnfocused(String),
    AutoFlushTextEdits,
    SliderChanged(String, f64),
    DropdownSelected(String, usize, Vec<SelectOption>),
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

        let (ipc_tx, ipc_rx) = std::sync::mpsc::channel();
        let sock_path = flags.socket_path;

        // Listen for requests from other instances
        std::thread::spawn(move || {
            let listener = match UnixListener::bind(&sock_path) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to bind IPC socket: {e}");
                    return;
                }
            };

            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    let mut buf = String::new();
                    let _ = stream.read_to_string(&mut buf);
                    let _ = ipc_tx.send(buf);
                    let _ = stream.write_all(b"k");
                }
            }
        });

        let has_applets = !flags.active_applets.is_empty();

        let app = Self {
            core,
            selected,
            applets: flags.active_applets,
            ipc_rx,
            current_schema: None,
            schema_loading: false,
            schema_error: None,
            local_values: HashMap::new(),
            text_edits: HashMap::new(),
            status_message: String::new(),
            last_loaded_idx: None,
            dropdown_labels: Vec::new(),
            text_displays: Vec::new(),
            pending_confirm: None,
            last_text_edit_time: None,
        };

        let task = if has_applets {
            Task::done(Action::App(Message::LoadSchema(selected)))
        } else {
            Task::none()
        };

        (app, task)
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

    fn subscription(&self) -> Subscription<Self::Message> {
        let ipc_sub =
            cosmic::iced::time::every(Duration::from_millis(250)).map(|_| Message::CheckIpc);

        let flush_sub = cosmic::iced::time::every(Duration::from_millis(500))
            .map(|_| Message::AutoFlushTextEdits);

        let mut subs = vec![ipc_sub, flush_sub];

        // Add refresh subscription if current schema has refresh_interval
        if let Some(ref schema) = self.current_schema {
            if let Some(interval) = schema.refresh_interval {
                if interval > 0 {
                    subs.push(
                        cosmic::iced::time::every(Duration::from_secs(interval))
                            .map(|_| Message::RefreshSchema),
                    );
                }
            }
        }

        Subscription::batch(subs)
    }

    fn update(&mut self, message: Self::Message) -> Task<Action<Self::Message>> {
        match message {
            Message::SelectApplet(idx) => {
                if self.selected != idx {
                    self.selected = idx;
                    return Task::done(Action::App(Message::LoadSchema(idx)));
                }
            }

            Message::OpenSettings(idx) => {
                if let Some(entry) = self.applets.get(idx) {
                    launch_settings_cmd(&entry.settings_cmd);
                }
            }

            Message::CheckIpc => {
                while let Ok(applet_id) = self.ipc_rx.try_recv() {
                    if !applet_id.is_empty() {
                        if let Some(idx) =
                            self.applets.iter().position(|a| a.applet_id == applet_id)
                        {
                            self.selected = idx;
                            return Task::done(Action::App(Message::LoadSchema(idx)));
                        }
                    }
                }
            }

            Message::LoadSchema(idx) => {
                if let Some(entry) = self.applets.get(idx).cloned() {
                    self.schema_loading = true;
                    self.schema_error = None;
                    self.current_schema = None;
                    self.local_values.clear();
                    self.text_edits.clear();
                    self.status_message.clear();
                    self.last_loaded_idx = Some(idx);
                    self.pending_confirm = None;

                    let binary = extract_binary(&entry.settings_cmd);
                    return Task::perform(
                        async move { run_settings_describe(&binary).await },
                        move |result| Action::App(Message::SchemaLoaded(idx, result)),
                    );
                }
            }

            Message::SchemaLoaded(idx, result) => {
                self.schema_loading = false;
                if idx == self.selected {
                    match result {
                        Ok(schema) => {
                            // Populate local_values from schema defaults
                            for section in &schema.sections {
                                for item in &section.items {
                                    self.local_values
                                        .insert(item.key.clone(), item.value.clone());
                                }
                            }
                            self.current_schema = Some(schema);
                            self.schema_error = None;
                            self.rebuild_display_cache();
                        }
                        Err(e) => {
                            self.schema_error = Some(e);
                            self.current_schema = None;
                        }
                    }
                }
            }

            Message::RefreshSchema => {
                if let Some(entry) = self.applets.get(self.selected).cloned() {
                    let binary = extract_binary(&entry.settings_cmd);
                    return Task::perform(
                        async move { run_settings_describe(&binary).await },
                        |result| Action::App(Message::RefreshSchemaLoaded(result)),
                    );
                }
            }

            Message::RefreshSchemaLoaded(result) => {
                if let Ok(schema) = result {
                    // Update local_values from refreshed schema
                    for section in &schema.sections {
                        for item in &section.items {
                            self.local_values
                                .insert(item.key.clone(), item.value.clone());
                        }
                    }
                    self.current_schema = Some(schema);
                    self.rebuild_display_cache();
                    // Note: pending_confirm is preserved across refreshes
                }
            }

            Message::SettingChanged(key, value) => {
                self.local_values.insert(key.clone(), value.clone());
                self.text_edits.remove(&key);
                self.status_message = format!("Applying {}...", key);

                if let Some(entry) = self.applets.get(self.selected).cloned() {
                    let binary = extract_binary(&entry.settings_cmd);
                    let key_clone = key.clone();
                    let value_json = value.to_string();
                    return Task::perform(
                        async move {
                            run_settings_set(&binary, &key_clone, &value_json).await
                        },
                        move |result| Action::App(Message::SettingApplied(result)),
                    );
                }
            }

            Message::SettingApplied(result) => {
                match &result {
                    Ok(msg) => self.status_message = msg.clone(),
                    Err(e) => self.status_message = format!("Error: {e}"),
                }
                // Re-describe to get fresh values
                let idx = self.selected;
                return Task::done(Action::App(Message::LoadSchema(idx)));
            }

            Message::ActionTriggered(action_id) => {
                // Check if this action has a confirm prompt
                if let Some(ref schema) = self.current_schema {
                    if let Some(action) = schema.actions.iter().find(|a| a.id == action_id) {
                        if let Some(ref confirm_msg) = action.confirm {
                            self.pending_confirm = Some(PendingConfirm {
                                action_id,
                                item_id: None,
                                message: confirm_msg.clone(),
                            });
                            return Task::none();
                        }
                    }
                }
                // No confirmation needed, dispatch immediately
                return self.dispatch_action(&action_id, None);
            }

            Message::ItemActionTriggered(action_id, item_id) => {
                // Check if this per-item action has a confirm prompt
                if let Some(ref schema) = self.current_schema {
                    'outer: for section in &schema.sections {
                        for item in &section.items {
                            if let Some(ref list_items) = item.list_items {
                                for li in list_items {
                                    if li.id == item_id {
                                        if let Some(action) =
                                            li.actions.iter().find(|a| a.id == action_id)
                                        {
                                            if let Some(ref confirm_msg) = action.confirm {
                                                self.pending_confirm = Some(PendingConfirm {
                                                    action_id,
                                                    item_id: Some(item_id),
                                                    message: confirm_msg.clone(),
                                                });
                                                return Task::none();
                                            }
                                        }
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }
                }
                // No confirmation needed
                return self.dispatch_action(&action_id, Some(&item_id));
            }

            Message::ConfirmAction => {
                if let Some(pending) = self.pending_confirm.take() {
                    return self
                        .dispatch_action(&pending.action_id, pending.item_id.as_deref());
                }
            }

            Message::CancelConfirm => {
                self.pending_confirm = None;
            }

            Message::ActionCompleted(result) => {
                match &result {
                    Ok(msg) => self.status_message = msg.clone(),
                    Err(e) => self.status_message = format!("Error: {e}"),
                }
                let idx = self.selected;
                return Task::done(Action::App(Message::LoadSchema(idx)));
            }

            Message::TextEditing(key, value) => {
                self.text_edits.insert(key, value);
                self.last_text_edit_time = Some(Instant::now());
                self.rebuild_display_cache();
            }

            Message::TextUnfocused(key) => {
                // Commit this text field immediately when it loses focus
                if let Some(value) = self.text_edits.remove(&key) {
                    self.last_text_edit_time = None;
                    return Task::done(Action::App(Message::SettingChanged(
                        key,
                        serde_json::Value::String(value),
                    )));
                }
            }

            Message::AutoFlushTextEdits => {
                // Debounce: commit pending text edits after 1s of no typing
                if let Some(last_edit) = self.last_text_edit_time {
                    if last_edit.elapsed() > Duration::from_secs(1)
                        && !self.text_edits.is_empty()
                    {
                        self.last_text_edit_time = None;
                        if let Some(entry) = self.applets.get(self.selected).cloned() {
                            let binary = extract_binary(&entry.settings_cmd);
                            let pending: Vec<(String, String)> = self
                                .text_edits
                                .drain()
                                .map(|(k, v)| {
                                    (k, serde_json::Value::String(v).to_string())
                                })
                                .collect();
                            self.status_message = "Saving...".to_string();
                            return Task::perform(
                                async move {
                                    let mut last_result =
                                        Ok("Settings applied".to_string());
                                    for (key, value) in pending {
                                        last_result =
                                            run_settings_set(&binary, &key, &value).await;
                                    }
                                    last_result
                                },
                                move |result| Action::App(Message::SettingApplied(result)),
                            );
                        }
                    }
                }
            }

            Message::SliderChanged(key, value) => {
                self.local_values
                    .insert(key.clone(), serde_json::Value::from(value));
                // Apply slider changes immediately
                return Task::done(Action::App(Message::SettingChanged(
                    key,
                    serde_json::Value::from(value),
                )));
            }

            Message::DropdownSelected(key, idx, options) => {
                if let Some(opt) = options.get(idx) {
                    return Task::done(Action::App(Message::SettingChanged(
                        key,
                        serde_json::Value::String(opt.value.clone()),
                    )));
                }
            }
        }
        Task::none()
    }
}

impl SettingsApp {
    /// Dispatch an action to the applet CLI.
    /// Flushes any pending text edits first so unsaved typing is committed
    /// before the action runs.
    fn dispatch_action(
        &mut self,
        action_id: &str,
        item_id: Option<&str>,
    ) -> Task<Action<Message>> {
        self.status_message = format!("Running {}...", action_id);

        if let Some(entry) = self.applets.get(self.selected).cloned() {
            let binary = extract_binary(&entry.settings_cmd);
            let id = action_id.to_string();
            let iid = item_id.map(|s| s.to_string());

            // Flush pending text edits before dispatching the action
            let pending: Vec<(String, String)> = self
                .text_edits
                .drain()
                .map(|(k, v)| (k, serde_json::Value::String(v).to_string()))
                .collect();

            return Task::perform(
                async move {
                    for (key, value) in pending {
                        let _ = run_settings_set(&binary, &key, &value).await;
                    }
                    run_settings_action(&binary, &id, iid.as_deref()).await
                },
                |result| Action::App(Message::ActionCompleted(result)),
            );
        }
        Task::none()
    }

    /// Rebuild cached dropdown labels and text display values.
    /// Called after schema load or text edits to keep widget data in sync.
    fn rebuild_display_cache(&mut self) {
        self.dropdown_labels.clear();
        self.text_displays.clear();

        if let Some(ref schema) = self.current_schema {
            for section in &schema.sections {
                for item in &section.items {
                    match item.item_type.as_str() {
                        "select" => {
                            let labels: Vec<String> = item
                                .options
                                .as_ref()
                                .map(|opts| opts.iter().map(|o| o.label.clone()).collect())
                                .unwrap_or_default();
                            self.dropdown_labels.push(labels);
                        }
                        "text" => {
                            let stored = self
                                .local_values
                                .get(&item.key)
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let display = self
                                .text_edits
                                .get(&item.key)
                                .cloned()
                                .unwrap_or(stored);
                            self.text_displays.push(display);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

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

        // If loading, show spinner text
        if self.schema_loading {
            return settings::view_column(vec![
                text::title1(&entry.name).into(),
                text::body("Loading settings...").into(),
            ])
            .into();
        }

        // If schema failed or not available, show fallback with "Open Settings" button
        if self.schema_error.is_some() || self.current_schema.is_none() {
            let mut items: Vec<Element<'_, Message>> = vec![text::title1(&entry.name).into()];
            if let Some(desc) = self.current_schema.as_ref().and_then(|s| s.description.clone())
            {
                items.push(text::caption(desc).into());
            }
            items.push(
                cosmic::widget::button::standard("Open Settings")
                    .on_press(Message::OpenSettings(self.selected))
                    .into(),
            );
            return settings::view_column(items).into();
        }

        let schema = self.current_schema.as_ref().unwrap();
        let mut content_items: Vec<Element<'_, Message>> =
            vec![text::title1(&schema.title).into()];

        // Description
        if let Some(ref desc) = schema.description {
            content_items.push(text::caption(desc.as_str()).into());
        }

        // Render sections using settings::section() with pre-computed display data
        let mut dd_idx = 0usize;
        let mut txt_idx = 0usize;

        for section in &schema.sections {
            // Render section-level actions before items (e.g. "Fetch" above history list)
            if !section.actions.is_empty() {
                let section_action_widgets: Vec<Element<'_, Message>> = section
                    .actions
                    .iter()
                    .map(|action| {
                        let is_pending = self
                            .pending_confirm
                            .as_ref()
                            .map(|p| p.action_id == action.id && p.item_id.is_none())
                            .unwrap_or(false);

                        if is_pending {
                            let msg = self.pending_confirm.as_ref().unwrap().message.clone();
                            row![
                                text::body(msg),
                                cosmic::widget::button::suggested("Confirm")
                                    .on_press(Message::ConfirmAction),
                                cosmic::widget::button::standard("Cancel")
                                    .on_press(Message::CancelConfirm),
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .into()
                        } else {
                            let btn: Element<'_, Message> = match action.style.as_deref() {
                                Some("destructive") => {
                                    cosmic::widget::button::destructive(&action.label)
                                        .on_press(Message::ActionTriggered(action.id.clone()))
                                        .into()
                                }
                                Some("suggested") => {
                                    cosmic::widget::button::suggested(&action.label)
                                        .on_press(Message::ActionTriggered(action.id.clone()))
                                        .into()
                                }
                                _ => cosmic::widget::button::standard(&action.label)
                                    .on_press(Message::ActionTriggered(action.id.clone()))
                                    .into(),
                            };
                            btn
                        }
                    })
                    .collect();

                content_items.push(
                    settings::section()
                        .add(settings::item_row(section_action_widgets))
                        .into(),
                );
            }

            let mut sec = settings::section().title(&section.title);
            let mut has_items = false;

            for item in &section.items {
                // Check visibility condition
                if let Some(ref cond) = item.visible_when {
                    let current_val = self.local_values.get(&cond.key);
                    let visible = match current_val {
                        Some(v) => values_equal(v, &cond.equals),
                        None => false,
                    };
                    if !visible {
                        // Still advance indices for skipped items
                        match item.item_type.as_str() {
                            "select" => dd_idx += 1,
                            "text" => txt_idx += 1,
                            _ => {}
                        }
                        continue;
                    }
                }

                match item.item_type.as_str() {
                    "toggle" => {
                        let value = self
                            .local_values
                            .get(&item.key)
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let key = item.key.clone();
                        sec = sec.add(settings::item(
                            &item.label,
                            toggler(value).on_toggle(move |v| {
                                Message::SettingChanged(
                                    key.clone(),
                                    serde_json::Value::Bool(v),
                                )
                            }),
                        ));
                        has_items = true;
                    }

                    "select" => {
                        if let Some(options) = item.options.as_ref() {
                            let current_value = self
                                .local_values
                                .get(&item.key)
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            let selected =
                                options.iter().position(|o| o.value == current_value);
                            let key = item.key.clone();
                            let opts = options.clone();
                            let labels = &self.dropdown_labels[dd_idx];

                            sec = sec.add(settings::item(
                                &item.label,
                                cosmic::widget::dropdown(labels, selected, move |idx| {
                                    Message::DropdownSelected(
                                        key.clone(),
                                        idx,
                                        opts.clone(),
                                    )
                                })
                                .width(Length::Fixed(200.0)),
                            ));
                            has_items = true;
                        }
                        dd_idx += 1;
                    }

                    "slider" => {
                        let value = self
                            .local_values
                            .get(&item.key)
                            .and_then(|v| v.as_f64())
                            .unwrap_or(item.min.unwrap_or(0.0));
                        let min = item.min.unwrap_or(0.0);
                        let max = item.max.unwrap_or(100.0);
                        let step = item.step.unwrap_or(1.0);
                        let unit = item.unit.as_deref().unwrap_or("");
                        let key = item.key.clone();
                        let value_label = if step >= 1.0 {
                            format!("{}{}", value as i64, unit)
                        } else {
                            format!("{:.1}{}", value, unit)
                        };

                        sec = sec.add(settings::flex_item(
                            &item.label,
                            widget::row()
                                .spacing(8)
                                .align_y(Alignment::Center)
                                .push(text::body(value_label))
                                .push(
                                    cosmic::widget::slider(min..=max, value, move |v| {
                                        let rounded = (v / step).round() * step;
                                        Message::SliderChanged(key.clone(), rounded)
                                    })
                                    .step(step)
                                    .width(Length::Fill),
                                ),
                        ));
                        has_items = true;
                    }

                    "text" => {
                        let placeholder = item.placeholder.as_deref().unwrap_or("");
                        let key = item.key.clone();
                        let key2 = item.key.clone();
                        let key3 = item.key.clone();
                        let display = &self.text_displays[txt_idx];

                        sec = sec.add(settings::item(
                            &item.label,
                            cosmic::widget::text_input(placeholder, display)
                                .on_input(move |v| {
                                    Message::TextEditing(key.clone(), v)
                                })
                                .on_submit(move |v| {
                                    Message::SettingChanged(
                                        key2.clone(),
                                        serde_json::Value::String(v),
                                    )
                                })
                                .on_unfocus(Message::TextUnfocused(key3))
                                .width(Length::Fixed(250.0)),
                        ));
                        txt_idx += 1;
                        has_items = true;
                    }

                    "info" => {
                        let value = self
                            .local_values
                            .get(&item.key)
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        sec = sec.add(settings::item(&item.label, text::body(value)));
                        has_items = true;
                    }

                    "image" => {
                        let height = item.height.unwrap_or(280.0);
                        let image_widget: Element<'_, Message> =
                            if let Some(path) = item.value.as_str().filter(|p| !p.is_empty()) {
                                container(
                                    widget::image(path)
                                        .content_fit(ContentFit::Contain)
                                        .height(Length::Fixed(height)),
                                )
                                .width(Length::Fill)
                                .center_x(Length::Fill)
                                .style(|theme: &cosmic::Theme| {
                                    let cosmic = theme.cosmic();
                                    container::Style {
                                        background: Some(Background::Color(
                                            cosmic.primary.component.base.into(),
                                        )),
                                        border: cosmic::iced::Border {
                                            radius: cosmic.corner_radii.radius_s.into(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }
                                })
                                .into()
                            } else {
                                container(text::caption("No image available"))
                                    .padding(40)
                                    .width(Length::Fill)
                                    .center_x(Length::Fill)
                                    .into()
                            };
                        // Image widgets go directly into the content column, not into a
                        // settings::section item, for full-width display
                        content_items.push(image_widget);
                        continue;
                    }

                    "list" => {
                        // List items rendered as individual card rows below the section
                        if let Some(ref list_items) = item.list_items {
                            // Push the current section if it has items, then render list
                            if has_items {
                                content_items.push(sec.into());
                                sec = settings::section().title(&section.title);
                                has_items = false;
                            }

                            content_items
                                .push(self.render_list_widget(list_items));
                        }
                        continue;
                    }

                    _ => {}
                }
            }

            if has_items {
                content_items.push(sec.into());
            }
        }

        // Render global actions section
        if !schema.actions.is_empty() {
            let action_widgets: Vec<Element<'_, Message>> = schema
                .actions
                .iter()
                .map(|action| {
                    // Check if this action is pending confirmation
                    let is_pending = self
                        .pending_confirm
                        .as_ref()
                        .map(|p| p.action_id == action.id && p.item_id.is_none())
                        .unwrap_or(false);

                    if is_pending {
                        let msg = self.pending_confirm.as_ref().unwrap().message.clone();
                        row![
                            text::body(msg),
                            cosmic::widget::button::suggested("Confirm")
                                .on_press(Message::ConfirmAction),
                            cosmic::widget::button::standard("Cancel")
                                .on_press(Message::CancelConfirm),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .into()
                    } else {
                        let btn: Element<'_, Message> = match action.style.as_deref() {
                            Some("destructive") => {
                                cosmic::widget::button::destructive(&action.label)
                                    .on_press(Message::ActionTriggered(action.id.clone()))
                                    .into()
                            }
                            Some("suggested") => {
                                cosmic::widget::button::suggested(&action.label)
                                    .on_press(Message::ActionTriggered(action.id.clone()))
                                    .into()
                            }
                            _ => cosmic::widget::button::standard(&action.label)
                                .on_press(Message::ActionTriggered(action.id.clone()))
                                .into(),
                        };
                        btn
                    }
                })
                .collect();

            content_items.push(
                settings::section()
                    .add(settings::item_row(action_widgets))
                    .into(),
            );
        }

        // Status message
        if !self.status_message.is_empty() {
            content_items.push(text::caption(&self.status_message).into());
        }

        settings::view_column(content_items).into()
    }

    /// Render a list widget as a column of card rows.
    fn render_list_widget<'a>(&'a self, list_items: &'a [ListItem]) -> Element<'a, Message> {
        if list_items.is_empty() {
            return container(text::caption("No items"))
                .padding(16)
                .width(Length::Fill)
                .center_x(Length::Fill)
                .into();
        }

        let mut col = column![].spacing(4);

        for li in list_items {
            let mut row_content = row![].spacing(12).align_y(Alignment::Center);

            // Optional thumbnail
            if let Some(ref img_path) = li.image {
                row_content = row_content.push(
                    container(
                        widget::image(img_path.clone())
                            .content_fit(ContentFit::Cover)
                            .width(Length::Fixed(120.0))
                            .height(Length::Fixed(68.0)),
                    )
                    .style(|theme: &cosmic::Theme| {
                        let cosmic = theme.cosmic();
                        container::Style {
                            border: cosmic::iced::Border {
                                radius: cosmic.corner_radii.radius_xs.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
                );
            }

            // Title + subtitle column
            let mut info_col = column![text::body(&li.title)].spacing(2);
            if let Some(ref subtitle) = li.subtitle {
                info_col = info_col.push(text::caption(subtitle));
            }
            row_content = row_content.push(info_col.width(Length::Fill));

            // Per-item action buttons
            let is_pending_item = self
                .pending_confirm
                .as_ref()
                .map(|p| p.item_id.as_deref() == Some(li.id.as_str()))
                .unwrap_or(false);

            if is_pending_item {
                let msg = self.pending_confirm.as_ref().unwrap().message.clone();
                row_content = row_content.push(
                    row![
                        text::caption(msg),
                        cosmic::widget::button::suggested("Confirm")
                            .on_press(Message::ConfirmAction),
                        cosmic::widget::button::standard("Cancel")
                            .on_press(Message::CancelConfirm),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                );
            } else {
                let mut btn_row = row![].spacing(6);
                for action in &li.actions {
                    let aid = action.id.clone();
                    let iid = li.id.clone();
                    let btn: Element<'_, Message> = match action.style.as_deref() {
                        Some("destructive") => {
                            cosmic::widget::button::destructive(&action.label)
                                .on_press(Message::ItemActionTriggered(aid, iid))
                                .into()
                        }
                        Some("suggested") => {
                            cosmic::widget::button::suggested(&action.label)
                                .on_press(Message::ItemActionTriggered(aid, iid))
                                .into()
                        }
                        _ => cosmic::widget::button::standard(&action.label)
                            .on_press(Message::ItemActionTriggered(aid, iid))
                            .into(),
                    };
                    btn_row = btn_row.push(btn);
                }
                row_content = row_content.push(btn_row);
            }

            // Wrap in a card-styled container
            let card = container(row_content.padding(8))
                .width(Length::Fill)
                .style(|theme: &cosmic::Theme| {
                    let cosmic = theme.cosmic();
                    container::Style {
                        background: Some(Background::Color(
                            cosmic.primary.component.base.into(),
                        )),
                        border: cosmic::iced::Border {
                            radius: cosmic.corner_radii.radius_s.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                });

            col = col.push(card);
        }

        col.into()
    }
}

// ---------------------------------------------------------------------------
// CLI protocol helpers
// ---------------------------------------------------------------------------

/// Extract the binary name from settings_cmd (first word).
fn extract_binary(settings_cmd: &str) -> String {
    settings_cmd
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

/// Run `<binary> --settings-describe` and parse the JSON output.
async fn run_settings_describe(binary: &str) -> Result<SettingsSchema, String> {
    let output = tokio::process::Command::new(binary)
        .arg("--settings-describe")
        .output()
        .await
        .map_err(|e| format!("Failed to run {binary} --settings-describe: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "{binary} --settings-describe failed (exit {}): {stderr}",
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse settings JSON from {binary}: {e}"))
}

/// Run `<binary> --settings-set <key> <json_value>` and return the message.
async fn run_settings_set(binary: &str, key: &str, json_value: &str) -> Result<String, String> {
    let output = tokio::process::Command::new(binary)
        .args(["--settings-set", key, json_value])
        .output()
        .await
        .map_err(|e| format!("Failed to run {binary} --settings-set: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_response(&stdout, binary)
}

/// Run `<binary> --settings-action <action_id> [<item_id>]` and return the message.
async fn run_settings_action(
    binary: &str,
    action_id: &str,
    item_id: Option<&str>,
) -> Result<String, String> {
    let mut args = vec!["--settings-action", action_id];
    if let Some(iid) = item_id {
        args.push(iid);
    }

    let output = tokio::process::Command::new(binary)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("Failed to run {binary} --settings-action: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_response(&stdout, binary)
}

/// Parse `{"ok": bool, "message": "..."}` response.
fn parse_response(stdout: &str, binary: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct Response {
        ok: bool,
        message: String,
    }
    match serde_json::from_str::<Response>(stdout) {
        Ok(resp) if resp.ok => Ok(resp.message),
        Ok(resp) => Err(resp.message),
        Err(e) => Err(format!("Bad response from {binary}: {e}")),
    }
}

/// Compare two serde_json::Values for equality (handles string/number/bool).
fn values_equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    match (a, b) {
        (serde_json::Value::String(sa), serde_json::Value::String(sb)) => sa == sb,
        (serde_json::Value::Bool(ba), serde_json::Value::Bool(bb)) => ba == bb,
        (serde_json::Value::Number(na), serde_json::Value::Number(nb)) => {
            na.as_f64() == nb.as_f64()
        }
        _ => a == b,
    }
}

fn launch_settings_cmd(cmd: &str) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if let Some((program, args)) = parts.split_first() {
        let _ = std::process::Command::new(program).args(args).spawn();
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
