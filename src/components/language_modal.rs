use crate::models::state::AppState;
use crate::services::config_store::save_config;
use crate::services::i18n::{min_left_pane_width, t, Language};
use dioxus::prelude::*;

#[component]
pub fn LanguageModal() -> Element {
    let mut state: AppState = use_context();
    let current_lang = (state.current_language)();

    let close_dialog = move |_| {
        state.is_language_dialog_open.set(false);
    };

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
        state.is_language_dialog_open.set(false);
    };

    rsx! {
        div {
            class: "modal-backdrop",
            onclick: close_dialog,
            div {
                class: "modal-content",
                style: "max-width: 440px;",
                onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h3 { class: "modal-title", "{t(current_lang, \"menu.dialog_title\")}" }
                    button {
                        class: "btn btn-secondary btn-sm",
                        onclick: close_dialog,
                        "✕"
                    }
                }
                div { class: "modal-body",
                    p { style: "font-size: 13px; color: var(--text-muted); margin-bottom: 16px;",
                        "{t(current_lang, \"menu.dialog_desc\")}"
                    }

                    div { style: "display: flex; flex-direction: column; gap: 10px;",
                        div {
                            class: if current_lang == Language::Deutsch { "language-option-card selected" } else { "language-option-card" },
                            onclick: move |_| set_language(Language::Deutsch),
                            div { style: "display: flex; align-items: center; justify-content: space-between; width: 100%;",
                                div { style: "display: flex; align-items: center; gap: 10px;",
                                    span { style: "font-size: 18px;", "🇩🇪" }
                                    span { style: "font-weight: 600; font-size: 14px;", "Deutsch" }
                                }
                                if current_lang == Language::Deutsch {
                                    span { class: "badge badge-active", "✓" }
                                }
                            }
                        }

                        div {
                            class: if current_lang == Language::English { "language-option-card selected" } else { "language-option-card" },
                            onclick: move |_| set_language(Language::English),
                            div { style: "display: flex; align-items: center; justify-content: space-between; width: 100%;",
                                div { style: "display: flex; align-items: center; gap: 10px;",
                                    span { style: "font-size: 18px;", "🇬🇧" }
                                    span { style: "font-weight: 600; font-size: 14px;", "English" }
                                }
                                if current_lang == Language::English {
                                    span { class: "badge badge-active", "✓" }
                                }
                            }
                        }
                    }
                }
                div { class: "modal-footer",
                    button {
                        class: "btn btn-secondary",
                        onclick: close_dialog,
                        "{t(current_lang, \"common.cancel\")}"
                    }
                }
            }
        }
    }
}
