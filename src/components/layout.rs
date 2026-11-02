use dioxus::prelude::*;

use crate::components::copy_modal::CopyModal;
use crate::components::tab_harnesses::TabHarnesses;
use crate::components::tab_sources::TabSources;
use crate::components::tab_targets::TabTargets;
use crate::models::skill::Skill;
use crate::models::state::{ActiveTab, AppState};
use crate::services::fs_scanner::scan_skills;

#[component]
pub fn AppLayout() -> Element {
    let mut state: AppState = use_context();
    let active_tab = (state.active_tab)();
    let status_msg = (state.status_message)();
    let is_copy_modal_open = (state.is_copy_modal_open)();

    let config = (state.config)();
    let selected_source_id = (state.selected_source_id)();

    // Collect skills for modal if open
    let modal_skills: Vec<Skill> = if is_copy_modal_open {
        if let Some(src) = config
            .sources
            .iter()
            .find(|s| Some(&s.id) == selected_source_id.as_ref())
        {
            scan_skills(&src.path).unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    rsx! {
        style { {include_str!("../../assets/style.css")} }

        div { class: "app-container",
            // Header
            header { class: "app-header",
                div { class: "brand-title",
                    span { "⚡" }
                    span { "Skill-Manager" }
                }

                nav { class: "tab-nav",
                    button {
                        class: if active_tab == ActiveTab::Sources { "tab-btn active" } else { "tab-btn" },
                        onclick: move |_| state.active_tab.set(ActiveTab::Sources),
                        "1. Sources"
                    }
                    button {
                        class: if active_tab == ActiveTab::Targets { "tab-btn active" } else { "tab-btn" },
                        onclick: move |_| state.active_tab.set(ActiveTab::Targets),
                        "2. Targets"
                    }
                    button {
                        class: if active_tab == ActiveTab::Harnesses { "tab-btn active" } else { "tab-btn" },
                        onclick: move |_| state.active_tab.set(ActiveTab::Harnesses),
                        "3. Harnesses"
                    }
                }
            }

            // Notification / Status banner
            if let Some(msg) = status_msg {
                div { class: "status-banner",
                    span { "{msg}" }
                    button {
                        class: "status-close-btn",
                        onclick: move |_| state.status_message.set(None),
                        "✕"
                    }
                }
            }

            // Main Tab Content
            main { class: "content-area",
                match active_tab {
                    ActiveTab::Sources => rsx! { TabSources {} },
                    ActiveTab::Targets => rsx! { TabTargets {} },
                    ActiveTab::Harnesses => rsx! { TabHarnesses {} },
                }
            }

            // Modal overlay
            if is_copy_modal_open {
                CopyModal { skills: modal_skills }
            }
        }
    }
}
