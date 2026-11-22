//! Exact 1:1 Rust translation of Smooth Scrolling
//! (SmajloSlovakian/Minecraft-Smooth-Scrolling, GPL-3.0-only).
//!
//! Upstream package io.github.smajloslovakian.smoothscroll, @639d45b.
//! Translation by sparticle. Carries the upstream GPL-3.0 license and credit.
//!
//! Module map (upstream -> here):
//!   SmoothScrolling.java, SmoothSc.java, Globals.java -> smooth_sc
//!   cfg/CfgValue.java          -> cfg_value
//!   cfg/CfgValueBuilder.java   -> cfg_value_builder
//!   cfg/NewConfig.java         -> new_config
//!   cfg/SmScCfg.java           -> sm_sc_cfg
//!   menu/*                     -> menu
//!   mixin/client/*             -> mixins
//!   duck/*                     -> duck
//!   TransformationAccess.java  -> transformation_access
//! Host seams for Minecraft / platform types live in seams.
//!
//! The hooks register the original's config fields as Horizon settings,
//! same defaults and ranges (the touchpad defaults follow the original's
//! per-OS values). /smoothscroll reads and writes them. Honest status:
//! the scrolling itself is not in game yet - the client mod API has no
//! mouse-wheel / scroll-event surface for container, creative, chat or
//! entry-list screens, and no text-field cursor surface, so the 1:1 logic
//! (cfg_value, new_config, sm_sc_cfg and the mixins' math) runs its tests
//! and stays unwired until that lands. The seams stay recorded in the
//! README.

use mod_api::{
    self, ClientHost, Command, CommandEvent, Flow, RegistrationContext, RegisterClientCommandsEvent,
    Registrar, SettingSpec,
};

pub mod seams;
pub mod smooth_sc;
pub mod cfg_value;
pub mod cfg_value_builder;
pub mod new_config;
pub mod sm_sc_cfg;
pub mod menu;
pub mod duck;
pub mod transformation_access;
pub mod mixins;
#[cfg(test)]
mod tests;

/// Upstream SmoothScrolling.MOD_ID.
pub const MOD_ID: &str = "smoothscroll";

/// The mod's id on Horizon: its settings' namespace and its command.
const HORIZON_ID: &str = "smooth_scroll";

#[derive(Default)]
struct Mod;

/// A registered field: (setting key, default), kept beside the
/// registration so the command reads and validates against one table.
const FIELDS: &[(&str, Field)] = &[
    ("hotbar_smoothness", Field::Num(20.0)),
    ("hotbar_rollover", Field::Bool(true)),
    ("hotbar_static_selector", Field::Bool(false)),
    ("chat_smoothness", Field::Num(50.0)),
    ("chat_opening_smoothness", Field::Num(50.0)),
    ("chat_scrolling_speed", Field::Num(0.0)),
    ("chat_suggestion_smoothness", Field::Num(50.0)),
    ("chat_suggestion_speed", Field::Num(30.0)),
    ("creative_screen_smoothness", Field::Num(50.0)),
    ("creative_screen_speed", Field::Num(30.0)),
    ("creative_screen_rp_compat", Field::Bool(false)),
    ("entry_list_smoothness", Field::Num(50.0)),
    ("entry_list_speed", Field::Num(30.0)),
    ("text_input_smoothness", Field::Num(50.0)),
    ("text_input_speed", Field::Num(100.0)),
    ("text_input_cursor_margin", Field::Num(10.0)),
    ("text_input_custom_cursor_update", Field::Bool(true)),
    ("touchpad_inertia_deceleration", Field::NumOs { windows: 0.0, other: 90.0 }),
    ("touchpad_inertia_stop", Field::Num(0.005)),
    ("touchpad_inertia_start", Field::Num(0.05)),
    ("touchpad_threshold", Field::Num(0.25)),
    ("touchpad_time_threshold", Field::Num(0.02)),
    ("touchpad_expect_merged", Field::BoolOs { windows: true, other: false }),
    ("misc_enable_mask_debug", Field::Bool(false)),
];

/// A field's kind and default (the original's config template's types).
#[derive(Clone, Copy)]
enum Field {
    Num(f64),
    Bool(bool),
    /// The original sets these per OS at config build time, so the default
    /// resolves to the build host's OS.
    NumOs { windows: f64, other: f64 },
    BoolOs { windows: bool, other: bool },
}

/// (min, max, step) for the numeric fields, exactly the template's
/// min_max/step values; bools take None.
const RANGES: &[(&str, (f64, f64, f64))] = &[
    ("hotbar_smoothness", (0.0, 100.0, 1.0)),
    ("chat_smoothness", (0.0, 100.0, 1.0)),
    ("chat_opening_smoothness", (0.0, 100.0, 1.0)),
    ("chat_scrolling_speed", (0.0, 100.0, 1.0)),
    ("chat_suggestion_smoothness", (0.0, 100.0, 1.0)),
    ("chat_suggestion_speed", (0.0, 100.0, 1.0)),
    ("creative_screen_smoothness", (0.0, 100.0, 1.0)),
    ("creative_screen_speed", (0.0, 100.0, 1.0)),
    ("entry_list_smoothness", (0.0, 100.0, 1.0)),
    ("entry_list_speed", (0.0, 100.0, 1.0)),
    ("text_input_smoothness", (0.0, 100.0, 1.0)),
    ("text_input_speed", (0.0, 300.0, 1.0)),
    ("text_input_cursor_margin", (0.0, 100.0, 1.0)),
    ("touchpad_inertia_deceleration", (0.0, 100.0, 1.0)),
    ("touchpad_inertia_stop", (0.0, 0.5, 0.001)),
    ("touchpad_inertia_start", (0.0, 1.0, 0.001)),
    ("touchpad_threshold", (0.0, 5.0, 0.01)),
    ("touchpad_time_threshold", (0.0, 0.25, 0.001)),
];

fn range_of(key: &str) -> Option<(f64, f64, f64)> {
    RANGES.iter().find(|(k, _)| *k == key).map(|(_, r)| *r)
}

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {
    /// Register the original's config fields as settings, same defaults and
    /// ranges (the touchpad defaults follow the original's per-OS values).
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        for (key, field) in FIELDS {
            let label = format!("setting.smooth_scroll.{}", key);
            match field {
                Field::Num(default) => {
                    let (min, max, step) = range_of(key).unwrap_or((0.0, 100.0, 1.0));
                    let _ = registrar.register_setting(SettingSpec::range(*key, label, min, max, step, *default));
                }
                Field::Bool(default) => {
                    let _ = registrar.register_setting(SettingSpec::toggle(*key, label, *default));
                }
                Field::NumOs { windows, other } => {
                    let (min, max, step) = range_of(key).unwrap_or((0.0, 100.0, 1.0));
                    let default = if smooth_sc::SmoothSc::is_windows() { *windows } else { *other };
                    let _ = registrar.register_setting(SettingSpec::range(*key, label, min, max, step, default));
                }
                Field::BoolOs { windows, other } => {
                    let default = if smooth_sc::SmoothSc::is_windows() { *windows } else { *other };
                    let _ = registrar.register_setting(SettingSpec::toggle(*key, label, default));
                }
            }
        }
    }

    /// `/smoothscroll` and its `ss` short form, answered locally and consumed.
    fn register_client_commands(
        &self,
        _host: &mut dyn ClientHost,
        event: &mut RegisterClientCommandsEvent,
    ) {
        event.commands.push(Command::new("smoothscroll <status|set> [field] [value]"));
        event.commands.push(Command::new("ss <status|set> [field] [value]"));
    }

    /// `/smoothscroll`: read the live values, or write one field back
    /// (changes persist through the settings surface, like the original's
    /// config file did under F3+T).
    fn command(&self, host: &mut dyn ClientHost, event: &mut CommandEvent) -> Flow {
        let raw = event.command.trim();
        let bare = raw.strip_prefix('/').unwrap_or(raw);
        let body = bare
            .strip_prefix("smoothscroll")
            .or_else(|| bare.strip_prefix("ss"));
        let Some(body) = body else {
            return Flow::Continue;
        };
        if !(bare.starts_with("smoothscroll") || bare.starts_with("ss")) {
            return Flow::Continue;
        }
        // The command word must be followed by a boundary so `/ssx` and
        // other commands stay theirs.
        let rest = body.trim_start();
        if !(rest.is_empty() || rest.starts_with(' ')) {
            return Flow::Continue;
        }
        let words: Vec<&str> = rest.split_whitespace().collect();
        let reply: String = match words.as_slice() {
            [] | ["status"] => status(host),
            ["set", key, value] => set_field(host, key, value),
            ["set"] => format!("usage: /smoothscroll set <field> <value> (fields: {})",
                FIELDS.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(" ")),
            _ => "usage: /smoothscroll <status|set> [field] [value]".to_owned(),
        };
        let _ = host.chat_line(mod_api::Text::literal(reply));
        Flow::Cancel
    }
}

/// One field's current value, the registered default filling any gap.
fn read_field(host: &dyn ClientHost, key: &str, field: &Field) -> String {
    match field {
        Field::Num(default) => num_text(host, key, *default),
        Field::Bool(default) => bool_text(host, key, *default),
        Field::NumOs { windows, other } => {
            num_text(host, key, if smooth_sc::SmoothSc::is_windows() { *windows } else { *other })
        }
        Field::BoolOs { windows, other } => {
            bool_text(host, key, if smooth_sc::SmoothSc::is_windows() { *windows } else { *other })
        }
    }
}

fn num_text(host: &dyn ClientHost, key: &str, default: f64) -> String {
    host.setting_f64(key)
        .map(format_number)
        .unwrap_or_else(|| format_number(default))
}

fn bool_text(host: &dyn ClientHost, key: &str, default: bool) -> String {
    host.setting_bool(key)
        .map(|v| v.to_string())
        .unwrap_or_else(|| default.to_string())
}

/// Format like the config template does: whole numbers without a fraction.
fn format_number(v: f64) -> String {
    if (v - v.round()).abs() < f64::EPSILON && v.abs() < 1e10 {
        format!("{}", v.round() as i64)
    } else {
        format!("{}", v)
    }
}

/// `/smoothscroll status`: every field, grouped the way the template groups.
fn status(host: &dyn ClientHost) -> String {
    let mut out = String::from("smoothscroll:");
    let mut first = true;
    for (key, field) in FIELDS {
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str(&format!(" {}={}", key, read_field(host, key, field)));
    }
    out
}

/// `/smoothscroll set <field> <value>`: validate against the registration,
/// write through the settings surface, echo the applied value.
fn set_field(host: &mut dyn ClientHost, key: &str, value: &str) -> String {
    let Some((_, field)) = FIELDS.iter().find(|(k, _)| *k == key) else {
        return format!(
            "unknown field {}. fields: {}",
            key,
            FIELDS.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(" ")
        );
    };
    match field {
        Field::Num(default) | Field::NumOs { windows: default, .. } => {
            let Ok(number) = value.parse::<f64>() else {
                return format!("{} takes a number", key);
            };
            let Some((min, max, _)) = range_of(key) else {
                return format!("{} is not writable here", key);
            };
            if !(min..=max).contains(&number) {
                return format!("{} takes {} to {}", key, format_number(min), format_number(max));
            }
            let _ = default; // the write persists; the default only fills reads
            host.set_setting(HORIZON_ID, key, &format_number(number));
            format!("{}={}", key, format_number(number))
        }
        Field::Bool(_) | Field::BoolOs { .. } => match value {
            "true" | "on" | "yes" => {
                host.set_setting(HORIZON_ID, key, "true");
                format!("{}=true", key)
            }
            "false" | "off" | "no" => {
                host.set_setting(HORIZON_ID, key, "false");
                format!("{}=false", key)
            }
            _ => format!("{} takes true or false", key),
        },
    }
}
