use crate::core::schedule::{Rotation, Schedule};
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::HashMap;

// Splatoon Map IDs
const MAP_IDS: &[i32] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

// Ranked modes
const RULES: &[&str] = &["cVar", "cVlf", "cVgl"];

// Officially banned map/rule combinations (MapID, GachiRule)
const BANNED_COMBOS: &[(i32, &str)] = &[
    (2, "cVlf"),  // Saltspray Rig - Tower Control
    (6, "cVlf"),  // Port Mackerel - Tower Control
    (4, "cVgl"),  // Blackbelly Skatepark - Rainmaker
    (14, "cVgl"), // Piranha Pit - Rainmaker
    (2, "cVgl"),  // Saltspray Rig - Rainmaker
];

#[derive(Default)]
pub struct RotationOverride {
    pub regular: Option<Vec<i32>>,
    pub ranked: Option<Vec<i32>>,
    pub rule: Option<String>,
}

// Generates the rotation schedule
pub fn generate(
    count: usize,
    start_time: i64,
    overrides: &HashMap<usize, RotationOverride>,
    duration_seconds: i64,
    allow_banned: bool,
) -> Schedule {
    let mut rng = rand::rng();
    let mut phases = Vec::with_capacity(count);

    let mut prev_regular: Vec<i32> = Vec::new();
    let mut prev_gachi: Vec<i32> = Vec::new();

    for i in 0..count {
        let start = start_time + (i as i64 * duration_seconds);
        let end = start + duration_seconds;

        let override_opt = overrides.get(&(i + 1));

        let gachi_rule = match override_opt.and_then(|o| o.rule.as_ref()) {
            Some(rule) => rule.clone(),
            None => RULES[i % RULES.len()].to_string(),
        };

        let regular = match override_opt.and_then(|o| o.regular.as_ref()) {
            Some(maps) => {
                let mut sorted = maps.clone();
                sorted.sort_unstable();
                sorted
            }
            None => pick_maps(&mut rng, MAP_IDS, &prev_regular, &[], "cPnt", allow_banned),
        };

        let gachi = match override_opt.and_then(|o| o.ranked.as_ref()) {
            Some(maps) => {
                let mut sorted = maps.clone();
                sorted.sort_unstable();
                sorted
            }
            None => pick_maps(&mut rng, MAP_IDS, &prev_gachi, &regular, &gachi_rule, allow_banned),
        };

        prev_regular = regular.clone();
        prev_gachi = gachi.clone();

        phases.push(Rotation {
            start,
            end,
            regular,
            gachi,
            regular_rule: "cPnt".to_string(),
            gachi_rule,
        });
    }

    Schedule { phases }
}

// Pick two maps from the pool, avoid any maps present in the previous rotation
fn pick_maps<R: Rng>(
    rng: &mut R,
    pool: &[i32],
    prev: &[i32],
    exclude: &[i32],
    current_rule: &str,
    allow_banned: bool,
) -> Vec<i32> {
    let banned_filtered: Vec<i32> = if allow_banned {
        pool.to_vec()
    } else {
        pool.iter()
            .filter(|&&m| !BANNED_COMBOS.contains(&(m, current_rule)))
            .copied()
            .collect()
    };

    let hard_filtered: Vec<i32> = banned_filtered
        .iter()
        .filter(|&&m| !exclude.contains(&m))
        .copied()
        .collect();

    let soft_filtered: Vec<i32> = hard_filtered
        .iter()
        .filter(|&&m| !prev.contains(&m))
        .copied()
        .collect();

    let source = if soft_filtered.len() >= 2 {
        soft_filtered
    } else {
        hard_filtered
    };

    let mut shuffled = source;
    shuffled.shuffle(rng);
    let mut picked: Vec<i32> = shuffled.into_iter().take(2).collect();
    picked.sort_unstable();
    picked
}
