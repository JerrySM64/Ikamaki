use crate::schedule::{Rotation, Schedule};
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::HashMap;

// Splatoon Map IDs
const MAP_IDS: &[i32] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

// Ranked modes
const RULES: &[&str] = &["cVar", "cVlf", "cVgl"];

// Duration of a rotation
const PHASE_DURATION: i64 = 2 * 60 * 60;

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
) -> Schedule {
    let mut rng = rand::rng();
    let mut phases = Vec::with_capacity(count);

    let mut prev_regular: Vec<i32> = Vec::new();
    let mut prev_gachi: Vec<i32> = Vec::new();

    for i in 0..count {
        let start = start_time + (i as i64 * PHASE_DURATION);
        let end = start + PHASE_DURATION;

        let override_opt = overrides.get(&(i + 1));

        let regular = match override_opt.and_then(|o| o.regular.as_ref()) {
            Some(maps) => {
                let mut sorted = maps.clone();
                sorted.sort_unstable();
                sorted
            }
            None => pick_maps(&mut rng, MAP_IDS, &prev_regular, &[]),
        };

        let gachi = match override_opt.and_then(|o| o.ranked.as_ref()) {
            Some(maps) => {
                let mut sorted = maps.clone();
                sorted.sort_unstable();
                sorted
            }
            None => pick_maps(&mut rng, MAP_IDS, &prev_gachi, &regular),
        };

        let gachi_rule = match override_opt.and_then(|o| o.rule.as_ref()) {
            Some(rule) => rule.clone(),
            None => RULES[i % RULES.len()].to_string(),
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
fn pick_maps<R: Rng>(rng: &mut R, pool: &[i32], prev: &[i32], exclude: &[i32]) -> Vec<i32> {
    let hard_filtered: Vec<i32> = pool
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
