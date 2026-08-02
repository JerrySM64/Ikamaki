use crate::schedule::Schedule;
use roead::Endian;
use roead::byml::Byml;

/*
   Fixes the formatting in the human readable YAML.
   Not the prettiest, but it works.
*/
fn fix_yaml(byml: &Byml) -> Byml {
    match byml {
        Byml::I32(v) => Byml::I64(i64::from(*v)),
        Byml::Array(arr) => Byml::Array(arr.iter().map(fix_yaml).collect()),
        Byml::Map(map) => {
            let mut new_map = roead::byml::Map::default();
            new_map.reserve(map.len());
            for (k, v) in map {
                new_map.insert(k.clone(), fix_yaml(v));
            }
            Byml::Map(new_map)
        }
        _ => byml.clone(),
    }
}

const MAP_FIRST_APPEAR: [(&str, i32); 16] = [
    ("2015-04-17", 0),
    ("2015-04-17", 1),
    ("2015-04-17", 2),
    ("2015-04-17", 3),
    ("2015-04-17", 4),
    ("2015-06-02", 6),
    ("2015-06-11", 7),
    ("2015-06-20", 9),
    ("2015-07-11", 8),
    ("2015-07-25", 5),
    ("2015-08-21", 11),
    ("2015-09-18", 10),
    ("2015-11-14", 12),
    ("2015-12-04", 15),
    ("2015-12-29", 14),
    ("2016-01-22", 13),
];

const RULE_FIRST_APPEAR: [(&str, &str); 3] = [
    ("2015-06-02", "cVar"),
    ("2015-07-02", "cVlf"),
    ("2015-08-15", "cVgl"),
];

const WEAPON_UNLOCK: [(&str, i32); 58] = [
    ("2015-06-02", 1060),
    ("2015-06-06", 2030),
    ("2015-06-13", 1020),
    ("2015-06-17", 1160),
    ("2015-06-17", 4031),
    ("2015-06-24", 1110),
    ("2015-06-27", 2000),
    ("2015-06-27", 1091),
    ("2015-07-08", 1000),
    ("2015-07-08", 1081),
    ("2015-07-18", 2040),
    ("2015-07-18", 1061),
    ("2015-07-22", 4040),
    ("2015-07-22", 1021),
    ("2015-08-01", 1130),
    ("2015-08-01", 2031),
    ("2015-08-06", 3000),
    ("2015-08-06", 5010),
    ("2015-08-29", 4050),
    ("2015-08-29", 1161),
    ("2015-09-05", 5000),
    ("2015-09-12", 1170),
    ("2015-09-26", 3010),
    ("2015-10-03", 2001),
    ("2015-10-10", 1131),
    ("2015-10-10", 4041),
    ("2015-10-17", 1150),
    ("2015-10-30", 1111),
    ("2015-10-30", 1171),
    ("2015-11-07", 5011),
    ("2015-11-11", 1001),
    ("2015-11-18", 4051),
    ("2015-11-21", 5020),
    ("2015-11-25", 3001),
    ("2015-11-28", 3020),
    ("2015-12-12", 5001),
    ("2015-12-19", 3011),
    ("2015-12-25", 1151),
    ("'2016-01-01T19:00:00+09:00'", 2041),
    ("2016-01-09", 3021),
    ("2016-01-16", 5021),
    ("'2016-04-13T11:00:00+09:00'", 1042),
    ("'2016-04-13T11:00:00+09:00'", 1072),
    ("'2016-04-13T11:00:00+09:00'", 2022),
    ("'2016-04-13T11:00:00+09:00'", 2032),
    ("'2016-04-13T11:00:00+09:00'", 3002),
    ("'2016-04-13T11:00:00+09:00'", 4002),
    ("'2016-04-13T11:00:00+09:00'", 4052),
    ("'2016-04-13T11:00:00+09:00'", 5002),
    ("'2016-06-08T11:00:00+09:00'", 1002),
    ("'2016-06-08T11:00:00+09:00'", 1032),
    ("'2016-06-08T11:00:00+09:00'", 1062),
    ("'2016-06-08T11:00:00+09:00'", 1132),
    ("'2016-06-08T11:00:00+09:00'", 1172),
    ("'2016-06-08T11:00:00+09:00'", 2012),
    ("'2016-06-08T11:00:00+09:00'", 4012),
    ("'2016-06-08T11:00:00+09:00'", 4022),
    ("'2016-06-08T11:00:00+09:00'", 5012),
];

const HEADER: &str = "Version: 1\n\
IsBigEndian: True\n\
SupportPaths: False\n\
HasReferenceNodes: False\n\
root:\n";

fn build_map_first_appear() -> Byml {
    Byml::Array(
        MAP_FIRST_APPEAR
            .iter()
            .map(|(date, id)| {
                let mut map = roead::byml::Map::default();
                map.insert("Date".into(), Byml::String((*date).into()));
                map.insert("MapID".into(), Byml::I32(*id));
                Byml::Map(map)
            })
            .collect(),
    )
}

fn build_rule_first_appear() -> Byml {
    Byml::Array(
        RULE_FIRST_APPEAR
            .iter()
            .map(|(date, rule)| {
                let mut map = roead::byml::Map::default();
                map.insert("Date".into(), Byml::String((*date).into()));
                map.insert("GachiRule".into(), Byml::String((*rule).into()));
                Byml::Map(map)
            })
            .collect(),
    )
}

fn build_weapon_unlock() -> Byml {
    Byml::Array(
        WEAPON_UNLOCK
            .iter()
            .map(|(date, id)| {
                let mut map = roead::byml::Map::default();
                map.insert("Date".into(), Byml::String((*date).into()));
                map.insert("WeaponSetID".into(), Byml::I32(*id));
                Byml::Map(map)
            })
            .collect(),
    )
}

fn build_byml(schedule: &Schedule, hash_kick: bool) -> Byml {
    let mut root = roead::byml::Map::default();

    root.insert("AddFirstMatchingTime".into(), Byml::I32(30));
    root.insert("AddMatchingTime".into(), Byml::I32(0));
    root.insert(
        "AfterFesBonusStart".into(),
        Byml::String("2016-07-24T19:00:00+09:00".into()),
    );
    root.insert("BottleneckThreasholdFrame".into(), Byml::I32(480));
    let aligned = crate::default_timestamp();
    let date_time = chrono::DateTime::from_timestamp(aligned, 0)
        .unwrap()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    root.insert("DateTime".into(), Byml::String(date_time.into()));
    root.insert("DisconnectByMemoryHash".into(), Byml::Bool(hash_kick));
    root.insert("TimeoutAfterJoin".into(), Byml::I32(120));
    root.insert("Version".into(), Byml::I32(3));
    root.insert("WaitMatchingTime".into(), Byml::I32(25));
    root.insert("WebPost".into(), Byml::Bool(true));

    root.insert("MapFirstAppear".into(), build_map_first_appear());
    root.insert("RuleFirstAppear".into(), build_rule_first_appear());
    root.insert("WeaponUnlock".into(), build_weapon_unlock());

    let phases_array: Vec<Byml> = schedule
        .phases
        .iter()
        .map(|phase| {
            let mut dict = roead::byml::Map::default();

            let duration_hours = ((phase.end - phase.start) / 3600) as i32;
            dict.insert("Time".into(), Byml::I32(duration_hours));

            let regular_stages: Vec<Byml> = phase
                .regular
                .iter()
                .map(|&m| {
                    let mut stage_map = roead::byml::Map::default();
                    stage_map.insert("MapID".into(), Byml::I32(m));
                    Byml::Map(stage_map)
                })
                .collect();
            dict.insert("RegularStages".into(), Byml::Array(regular_stages));

            let gachi_stages: Vec<Byml> = phase
                .gachi
                .iter()
                .map(|&m| {
                    let mut stage_map = roead::byml::Map::default();
                    stage_map.insert("MapID".into(), Byml::I32(m));
                    Byml::Map(stage_map)
                })
                .collect();
            dict.insert("GachiStages".into(), Byml::Array(gachi_stages));

            dict.insert(
                "RegularRule".into(),
                Byml::String(phase.regular_rule.clone().into()),
            );
            dict.insert(
                "GachiRule".into(),
                Byml::String(phase.gachi_rule.clone().into()),
            );

            Byml::Map(dict)
        })
        .collect();

    root.insert("Phases".into(), Byml::Array(phases_array));

    Byml::Map(root)
}

// Export the BYAML file
pub fn export(
    schedule: &Schedule,
    path: &str,
    hash_kick: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let byml = build_byml(schedule, hash_kick);
    // Version 1 is being used on console
    let bytes = byml.to_binary_with_version(Endian::Big, 1);
    std::fs::write(path, bytes)?;
    Ok(())
}

// Export the human-readable YAML file
pub fn export_text(
    schedule: &Schedule,
    path: &str,
    hash_kick: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = fix_yaml(&build_byml(schedule, hash_kick)).to_text();
    let mut out = String::from(HEADER);
    for line in body.lines() {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
    std::fs::write(path, out)?;
    Ok(())
}
