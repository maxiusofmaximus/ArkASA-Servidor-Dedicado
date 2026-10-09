//! Config drift detection.
//!
//! The app owns `server-config.toml` and regenerates `Game.ini` /
//! `GameUserSettings.ini` from it. But ARK itself rewrites those INIs on boot
//! (this is the single most-reported admin pain — it is exactly how the
//! `ServerAdminPassword` corruption bug bit us), and plugins/other tools edit
//! them too. Drift detection answers "what is on disk differs from what I
//! intended?", which used to be invisible.
//!
//! Implementation: render the INI we *would* write, parse it and the on-disk
//! file into `section → key → value`, and report every managed key whose
//! on-disk value is missing or different.

use crate::config::persister::ConfigPersister;
use crate::config::schema::ServerConfig;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DriftEntry {
    /// Which generated file the entry belongs to (`GameUserSettings.ini` / `Game.ini`).
    pub file: String,
    pub section: String,
    pub key: String,
    /// What the app would write.
    pub expected: String,
    /// What is actually on disk (`None` = key absent).
    pub actual: Option<String>,
}

type IniMap = BTreeMap<String, BTreeMap<String, String>>;

/// Minimal INI reader: `[Section]` headers + `Key=Value`, `;`/`#` comments.
fn parse_ini(text: &str) -> IniMap {
    let mut map: IniMap = BTreeMap::new();
    let mut section = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            if let Some(name) = rest.strip_suffix(']') {
                section = name.trim().to_string();
            }
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.entry(section.clone())
                .or_default()
                .insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}

fn diff_file(file: &str, expected_ini: &str, actual_path: &str, out: &mut Vec<DriftEntry>) {
    let expected = parse_ini(expected_ini);
    let actual_text = std::fs::read_to_string(actual_path).ok();
    let actual = actual_text.as_deref().map(parse_ini).unwrap_or_default();

    for (section, keys) in &expected {
        // A section-less preamble (e.g. the `[Internationalization]` block) is
        // still compared; sections are just labels.
        for (key, exp_val) in keys {
            let act_val = actual.get(section).and_then(|m| m.get(key)).cloned();
            if act_val.as_deref() != Some(exp_val.as_str()) {
                out.push(DriftEntry {
                    file: file.to_string(),
                    section: section.clone(),
                    key: key.clone(),
                    expected: exp_val.clone(),
                    actual: act_val,
                });
            }
        }
    }
}

/// Compare the config against the on-disk INIs. Empty result = no drift.
pub fn detect(config: &ServerConfig) -> Vec<DriftEntry> {
    let mut out = Vec::new();

    if !config.paths.gamesettings_ini_path.trim().is_empty() {
        diff_file(
            "GameUserSettings.ini",
            &ConfigPersister::generate_gamesettings_ini_content(config),
            &config.paths.gamesettings_ini_path,
            &mut out,
        );
    }
    if !config.paths.game_ini_path.trim().is_empty() {
        diff_file(
            "Game.ini",
            &ConfigPersister::generate_game_ini_content(config),
            &config.paths.game_ini_path,
            &mut out,
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sections_and_keys() {
        let m = parse_ini("[A]\nx=1\n; c\n[B]\ny = 2 \n");
        assert_eq!(m["A"]["x"], "1");
        assert_eq!(m["B"]["y"], "2");
    }

    #[test]
    fn detects_drift_and_missing_keys() {
        let dir = std::env::temp_dir().join(format!("ark-drift-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        let gus = dir.join("GameUserSettings.ini");
        let gi = dir.join("Game.ini");

        let mut config = ServerConfig::default();
        config.paths.gamesettings_ini_path = gus.to_string_lossy().to_string();
        config.paths.game_ini_path = gi.to_string_lossy().to_string();

        // 1) Write exactly what the app would → zero drift.
        std::fs::write(&gus, ConfigPersister::generate_gamesettings_ini_content(&config)).unwrap();
        std::fs::write(&gi, ConfigPersister::generate_game_ini_content(&config)).unwrap();
        assert!(detect(&config).is_empty(), "identical files must report no drift");

        // 2) Corrupt one value + delete a key → both reported.
        let text = std::fs::read_to_string(&gus).unwrap();
        let mut drifted: Vec<String> = Vec::new();
        for l in text.lines() {
            if l.starts_with("SessionName=") {
                drifted.push("SessionName=HIJACKED".to_string());
            } else if l.starts_with("MaxPlayers=") {
                // drop it
            } else {
                drifted.push(l.to_string());
            }
        }
        std::fs::write(&gus, drifted.join("\n")).unwrap();

        let entries = detect(&config);
        assert!(entries.iter().any(|e| e.key == "SessionName" && e.actual.as_deref() == Some("HIJACKED")));
        assert!(entries.iter().any(|e| e.key == "MaxPlayers" && e.actual.is_none()));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
