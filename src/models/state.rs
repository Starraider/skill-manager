use crate::models::config::AppConfig;
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ActiveTab {
    #[default]
    Sources,
    Targets,
    Harnesses,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub config: Signal<AppConfig>,
    pub active_tab: Signal<ActiveTab>,
    pub selected_source_id: Signal<Option<String>>,
    pub selected_target_id: Signal<Option<String>>,
    pub selected_skill_names: Signal<HashSet<String>>,
    pub is_copy_modal_open: Signal<bool>,
    pub status_message: Signal<Option<String>>,
}
