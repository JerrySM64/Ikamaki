use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use chrono::{Datelike, Timelike};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::core::generator::RotationOverride;
use crate::core::schedule::Schedule;

slint::include_modules!();

// Rule IDs in the order of the rule ComboBox, where index 0 is "Default"
const RULES: [&str; 3] = ["cVar", "cVlf", "cVgl"];

pub struct IkamakiApp {
    window: MainWindow,
}

impl IkamakiApp {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let window = MainWindow::new()?;
        let start_time = Rc::new(Cell::new(crate::core::default_timestamp()));
        let entries = Rc::new(VecModel::<OverrideRow>::default());

        let (year, month, day, hour, minute) = local_parts(start_time.get());
        window.set_start_year(year);
        window.set_start_month(i32::try_from(month).unwrap_or(1));
        window.set_start_day(i32::try_from(day).unwrap_or(1));
        window.set_start_hour(i32::try_from(hour).unwrap_or(0));
        window.set_start_minute(i32::try_from(minute).unwrap_or(0));
        window.set_overrides(ModelRc::from(entries.clone()));
        window.set_yaml_supported(cfg!(not(target_arch = "wasm32")));        window.set_status("Configure the schedule, then press \"Generate & Save\".".into());
        window.set_end_time(end_time_string(start_time.get(), count(&window), duration_hours(&window)).into());

        window.on_config_changed({
            let window = window.as_weak();
            let start_time = start_time.clone();
            let entries = entries.clone();
            move || {
                let Some(window) = window.upgrade() else {
                    return;
                };
                if let Some(timestamp) = local_timestamp(
                    window.get_start_year(),
                    u32::try_from(window.get_start_month()).unwrap_or(1),
                    u32::try_from(window.get_start_day()).unwrap_or(1),
                    u32::try_from(window.get_start_hour()).unwrap_or(0),
                    u32::try_from(window.get_start_minute()).unwrap_or(0),
                ) {
                    start_time.set(timestamp);
                }
                window.set_end_time(
                    end_time_string(start_time.get(), count(&window), duration_hours(&window))
                        .into(),
                );
                sync_overrides(&window, &entries, start_time.get());
            }
        });

        window.on_generate({
            let window = window.as_weak();
            let entries = entries.clone();
            let start_time = start_time.clone();
            move || {
                let Some(window) = window.upgrade() else {
                    return;
                };
                let status = generate_and_save(&window, start_time.get(), &entries);
                window.set_status(status.into());
            }
        });

        window.on_add_override({
            let window = window.as_weak();
            let entries = entries.clone();
            let start_time = start_time.clone();
            move || {
                let Some(window) = window.upgrade() else {
                    return;
                };
                // First rotation that does not have an override yet
                let taken: Vec<i32> = entries.iter().map(|row| row.interval).collect();
                let free = (1..=i32::try_from(count(&window)).unwrap_or(i32::MAX))
                    .find(|interval| !taken.contains(interval));
                let Some(interval) = free else {
                    window.set_status("Every rotation already has an override.".into());
                    return;
                };
                entries.push(OverrideRow {
                    interval,
                    year: 0,
                    month: 0,
                    day: 0,
                    hour: 0,
                    minute: 0,
                    regular: SharedString::new(),
                    ranked: SharedString::new(),
                    rule: 0,
                });
                // Clamps the interval into the schedule and fills in its time
                sync_overrides(&window, &entries, start_time.get());
            }
        });

        window.on_override_time_picked({
            let window = window.as_weak();
            let entries = entries.clone();
            let start_time = start_time.clone();
            move |index, year, month, day, hour, minute| {
                let Some(window) = window.upgrade() else {
                    return;
                };
                let Some(requested) = local_timestamp(
                    year,
                    u32::try_from(month).unwrap_or(1),
                    u32::try_from(day).unwrap_or(1),
                    u32::try_from(hour).unwrap_or(0),
                    u32::try_from(minute).unwrap_or(0),
                ) else {
                    return;
                };
                let Ok(index) = usize::try_from(index) else {
                    return;
                };
                let Some(mut row) = entries.row_data(index) else {
                    return;
                };

                let start = start_time.get();
                let duration = duration_hours(&window) * 3600;
                let interval = snap_interval(requested, start, duration, count(&window));
                let taken = entries
                    .iter()
                    .enumerate()
                    .any(|(other, entry)| other != index && i64::from(entry.interval) == interval);
                if taken {
                    window.set_status(
                        format!("Rotation {interval} already has an override, pick another time")
                            .into(),
                    );
                    return;
                }
                row.interval = i32::try_from(interval).unwrap_or(1);
                entries.set_row_data(index, row);
                sync_overrides(&window, &entries, start);

                let snapped = start + (interval - 1) * duration;
                if snapped != requested {
                    window.set_status(
                        format!(
                            "Override moved to rotation {interval}, the nearest one inside the schedule ({})",
                            time_string(snapped)
                        )
                        .into(),
                    );
                }
            }
        });

        window.on_remove_override(move |index| {
            if let Ok(index) = usize::try_from(index) {
                entries.remove(index);
            }
        });

        Ok(Self { window })
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.window.run()
    }
}

fn count(window: &MainWindow) -> usize {
    usize::try_from(window.get_count()).unwrap_or(1)
}

fn duration_hours(window: &MainWindow) -> i64 {
    i64::from(window.get_duration_hours())
}

fn local_parts(timestamp: i64) -> (i32, u32, u32, u32, u32) {
    match chrono::DateTime::from_timestamp(timestamp, 0) {
        Some(dt) => {
            let local = dt.with_timezone(&chrono::Local);
            (
                local.year(),
                local.month(),
                local.day(),
                local.hour(),
                local.minute(),
            )
        }
        None => (1970, 1, 1, 0, 0),
    }
}

fn local_timestamp(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Option<i64> {
    chrono::NaiveDate::from_ymd_opt(year, month, day)?
        .and_hms_opt(hour, minute, 0)?
        .and_local_timezone(chrono::Local)
        .earliest()
        .map(|dt| dt.timestamp())
}

fn end_time_string(start_time: i64, count: usize, duration_hours: i64) -> String {
    time_string(start_time + (count as i64 * duration_hours * 3600))
}

fn time_string(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "unknown".to_string())
}

// Snaps to the nearest rotation start
fn snap_interval(requested: i64, start_time: i64, duration_seconds: i64, count: usize) -> i64 {
    let nearest = (requested - start_time + duration_seconds / 2).div_euclid(duration_seconds);
    (nearest + 1).clamp(1, (count as i64).max(1))
}

fn sync_overrides(window: &MainWindow, entries: &VecModel<OverrideRow>, start_time: i64) {
    let duration = duration_hours(window) * 3600;
    let last = (count(window) as i64).max(1);
    for index in 0..entries.row_count() {
        let Some(mut row) = entries.row_data(index) else {
            continue;
        };
        let interval = i64::from(row.interval).clamp(1, last);
        let (year, month, day, hour, minute) = local_parts(start_time + (interval - 1) * duration);
        row.interval = i32::try_from(interval).unwrap_or(1);
        row.year = year;
        row.month = i32::try_from(month).unwrap_or(1);
        row.day = i32::try_from(day).unwrap_or(1);
        row.hour = i32::try_from(hour).unwrap_or(0);
        row.minute = i32::try_from(minute).unwrap_or(0);
        entries.set_row_data(index, row);
    }
}

fn build_overrides(
    entries: &VecModel<OverrideRow>,
) -> Result<HashMap<usize, RotationOverride>, String> {
    let mut overrides = HashMap::new();
    for entry in entries.iter() {
        let interval = usize::try_from(entry.interval)
            .ok()
            .filter(|interval| *interval != 0)
            .ok_or_else(|| "override intervals start at 1".to_string())?;
        let rule = usize::try_from(entry.rule)
            .ok()
            .and_then(|rule| rule.checked_sub(1))
            .and_then(|rule| RULES.get(rule))
            .map(|rule| (*rule).to_string());
        let replaced = overrides.insert(
            interval,
            RotationOverride {
                regular: parse_map_ids(&entry.regular)?,
                ranked: parse_map_ids(&entry.ranked)?,
                rule,
            },
        );
        if replaced.is_some() {
            return Err(format!("more than one override targets rotation {interval}"));
        }
    }
    Ok(overrides)
}

// Returns the message for the status bar
fn generate_and_save(
    window: &MainWindow,
    start_time: i64,
    entries: &VecModel<OverrideRow>,
) -> String {
    let overrides = match build_overrides(entries) {
        Ok(overrides) => overrides,
        Err(err) => return format!("Error: {err}"),
    };

    let count = count(window);
    let duration_hours = duration_hours(window);
    let schedule = crate::core::generator::generate(
        count,
        start_time,
        &overrides,
        duration_hours * 3600,
        window.get_allow_banned(),
    );

    let mut status = format!(
        "Successfully generated {} rotation(s). Schedule ends at: {}",
        count,
        end_time_string(start_time, count, duration_hours)
    );
    save_schedule(
        &mut status,
        &schedule,
        window.get_hash_kick(),
        window.get_save_yml(),
    );
    status
}

#[cfg(not(target_arch = "wasm32"))]
fn save_schedule(status: &mut String, schedule: &Schedule, hash_kick: bool, save_yml: bool) {
    let Some(path) = rfd::FileDialog::new()
        .set_file_name("VSSetting.byaml")
        .add_filter("BYAML", &["byaml"])
        .save_file()
    else {
        status.push_str(" (save cancelled)");
        return;
    };

    let bytes = crate::core::exporter::export_bytes(schedule, hash_kick);
    if let Err(err) = std::fs::write(&path, bytes) {
        *status = format!("Failed to write {}: {err}", path.display());
        return;
    }

    if save_yml {
        let yml_path = path.with_extension("yml");
        let yaml = crate::core::exporter::export_yaml_string(schedule, hash_kick);
        if let Err(err) = std::fs::write(&yml_path, yaml) {
            *status = format!(
                "Saved {}, but failed to write {}: {err}",
                path.display(),
                yml_path.display()
            );
            return;
        }
    }

    status.push_str(&format!(" -> Saved to {}", path.display()));
}

// The YAML export is native only, see the roead dependency in Cargo.toml
#[cfg(target_arch = "wasm32")]
fn save_schedule(status: &mut String, schedule: &Schedule, hash_kick: bool, _save_yml: bool) {
    let bytes = crate::core::exporter::export_bytes(schedule, hash_kick);
    if let Err(err) = download_bytes(&bytes, "VSSetting.byaml") {
        *status = format!("Download failed: {err:?}");
        return;
    }

    status.push_str(" -> Download started");
}

fn parse_map_ids(raw: &str) -> Result<Option<Vec<i32>>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let mut ids = Vec::new();
    for part in trimmed.split(',') {
        let part = part.trim();
        match part.parse::<i32>() {
            Ok(id) => ids.push(id),
            Err(_) => return Err(format!("invalid map ID '{part}'")),
        }
    }
    if ids.len() != 2 {
        return Err(format!("expected exactly 2 map IDs, got {}", ids.len()));
    }
    Ok(Some(ids))
}

#[cfg(target_arch = "wasm32")]
fn download_bytes(bytes: &[u8], filename: &str) -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::{JsCast, JsValue};

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;

    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    let part: JsValue = array.into();
    parts.push(&part);

    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;

    let anchor = document
        .create_element("a")?
        .dyn_into::<web_sys::HtmlAnchorElement>()?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.set_hidden(true);
    anchor.click();

    web_sys::Url::revoke_object_url(&url)?;
    Ok(())
}
