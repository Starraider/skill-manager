use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use std::collections::HashSet;

use skill_manager::components::AppLayout;
use skill_manager::models::state::{ActiveTab, AppState};
use skill_manager::services::config_store::load_config;

#[component]
fn App() -> Element {
    let initial_config = use_signal(|| load_config().unwrap_or_default());
    let active_tab = use_signal(|| ActiveTab::Sources);
    let selected_source_id = use_signal(|| initial_config().sources.first().map(|s| s.id.clone()));
    let selected_target_id = use_signal(|| initial_config().targets.first().map(|t| t.id.clone()));
    let selected_skill_names = use_signal(HashSet::new);
    let is_copy_modal_open = use_signal(|| false);
    let status_message = use_signal(|| None);

    let state = AppState {
        config: initial_config,
        active_tab,
        selected_source_id,
        selected_target_id,
        selected_skill_names,
        is_copy_modal_open,
        status_message,
    };

    use_context_provider(|| state);

    rsx! {
        AppLayout {}
    }
}

fn main() {
    let window = WindowBuilder::new()
        .with_title("Skill-Manager")
        .with_inner_size(LogicalSize::new(1100.0, 750.0))
        .with_min_inner_size(LogicalSize::new(900.0, 600.0));

    let desktop_cfg = Config::default().with_window(window);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(desktop_cfg)
        .launch(App);
}
