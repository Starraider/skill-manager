use dioxus::prelude::*;

use crate::components::copy_modal::CopyModal;
use crate::components::language_modal::LanguageModal;
use crate::components::tab_harnesses::TabHarnesses;
use crate::components::tab_sources::TabSources;
use crate::components::tab_targets::TabTargets;
use crate::models::skill::Skill;
use crate::models::state::{ActiveTab, AppState};
use crate::services::config_store::save_config;
use crate::services::fs_scanner::scan_skills;
use crate::services::i18n::{min_left_pane_width, t, Language};

#[component]
pub fn AppLayout() -> Element {
    let mut state: AppState = use_context();
    let current_lang = (state.current_language)();
    let active_tab = (state.active_tab)();
    let status_msg = (state.status_message)();
    let is_copy_modal_open = (state.is_copy_modal_open)();
    let is_language_dialog_open = (state.is_language_dialog_open)();
    let mut is_lang_menu_open = use_signal(|| false);

    let config = (state.config)();
    let selected_source_id = (state.selected_source_id)();

    let mut set_language = move |lang: Language| {
        state.current_language.set(lang);
        let min_w = min_left_pane_width(lang);
        if (state.left_pane_width)() < min_w {
            state.left_pane_width.set(min_w);
        }
        let mut cfg = (state.config)();
        cfg.language = lang;
        cfg.left_pane_width = Some((state.left_pane_width)());
        let _ = save_config(&cfg);
        state.config.set(cfg);
        is_lang_menu_open.set(false);
    };

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
                    img {
                        class: "brand-icon",
                        src: asset!("/assets/icons/app-icon.png"),
                        alt: "Skill-Manager",
                    }
                    span { "{t(current_lang, \"app.title\")}" }
                }

                div { class: "header-right",
                    nav { class: "tab-nav",
                        button {
                            class: if active_tab == ActiveTab::Sources { "tab-btn active" } else { "tab-btn" },
                            onclick: move |_| state.active_tab.set(ActiveTab::Sources),
                            "{t(current_lang, \"tabs.sources\")}"
                        }
                        button {
                            class: if active_tab == ActiveTab::Targets { "tab-btn active" } else { "tab-btn" },
                            onclick: move |_| state.active_tab.set(ActiveTab::Targets),
                            "{t(current_lang, \"tabs.targets\")}"
                        }
                        button {
                            class: if active_tab == ActiveTab::Harnesses { "tab-btn active" } else { "tab-btn" },
                            onclick: move |_| state.active_tab.set(ActiveTab::Harnesses),
                            "{t(current_lang, \"tabs.harnesses\")}"
                        }
                    }

                    div { class: "header-menu-divider" }

                    // Top-level menu item: Language
                    div { class: "menu-dropdown-container",
                        button {
                            class: if is_lang_menu_open() { "menu-top-btn active" } else { "menu-top-btn" },
                            onclick: move |_| is_lang_menu_open.set(!is_lang_menu_open()),
                            span { "🌐" }
                            span { "{t(current_lang, \"menu.language\")}" }
                            span { class: "menu-chevron", "▾" }
                        }

                        if is_lang_menu_open() {
                            div {
                                class: "dropdown-backdrop-transparent",
                                onclick: move |_| is_lang_menu_open.set(false),
                            }
                            div { class: "dropdown-menu",
                                button {
                                    class: if current_lang == Language::Deutsch { "dropdown-item selected" } else { "dropdown-item" },
                                    onclick: move |_| set_language(Language::Deutsch),
                                    span { class: "dropdown-check", if current_lang == Language::Deutsch { "✓" } else { " " } }
                                    span { "Deutsch" }
                                }
                                button {
                                    class: if current_lang == Language::English { "dropdown-item selected" } else { "dropdown-item" },
                                    onclick: move |_| set_language(Language::English),
                                    span { class: "dropdown-check", if current_lang == Language::English { "✓" } else { " " } }
                                    span { "English" }
                                }
                                div { class: "dropdown-divider" }
                                button {
                                    class: "dropdown-item dropdown-action",
                                    onclick: move |_| {
                                        is_lang_menu_open.set(false);
                                        state.is_language_dialog_open.set(true);
                                    },
                                    span { class: "dropdown-check", "⚙" }
                                    span { "{t(current_lang, \"menu.dialog_title\")}..." }
                                }
                            }
                        }
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
                        "{t(current_lang, \"common.close\")}"
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

            // Language Dialog Modal
            if is_language_dialog_open {
                LanguageModal {}
            }
        }
    }
}

