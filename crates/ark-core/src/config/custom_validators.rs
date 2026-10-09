//! Operator-defined configuration validators ("validator plugins").
//!
//! The six built-in validators in [`crate::config::validator`] are compiled in.
//! This module lets an operator add **declarative** rules at runtime — no
//! recompilation, no code execution, no new dependency — stored in
//! `<config_dir>/custom_validators.toml`:
//!
//! ```toml
//! [[rules]]
//! id = "min-players"
//! field = "gameplay.max_players"   # dot path into the config
//! kind = "range"
//! min = 5
//! max = 200
//! message = "El servidor debe admitir entre 5 y 200 jugadores"
//!
//! [[rules]]
//! id = "no-space-name"
//! field = "identification.session_name"
//! kind = "not_empty"
//! ```
//!
//! Rules are evaluated against a JSON projection of [`ServerConfig`], so a
//! `field` is simply the TOML path of the setting (`network.port`,
//! `multipliers.xp_multiplier`, …). This is the safe, portable middle ground
//! between "no extensibility" and "load arbitrary code".

use crate::config::schema::ServerConfig;
use crate::config::validator::ValidationError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// One declarative rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomRule {
    /// Unique id (also surfaced in the validation error list).
    pub id: String,
    /// Dot path into the config, e.g. `network.port`.
    pub field: String,
    /// One of the supported checks (see [`evaluate_rule`]).
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    // Check parameters (only the ones the `kind` needs are read).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_len: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_len: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl CustomRule {
    pub fn label(&self) -> String {
        format!("custom:{}", self.id)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomRuleSet {
    #[serde(default)]
    pub rules: Vec<CustomRule>,
}

/// Read/write access to `<config_dir>/custom_validators.toml`.
pub struct CustomValidatorStore {
    config_dir: PathBuf,
}

impl CustomValidatorStore {
    pub fn new(config_dir: impl Into<PathBuf>) -> Self {
        Self { config_dir: config_dir.into() }
    }

    pub fn path(&self) -> PathBuf {
        self.config_dir.join("custom_validators.toml")
    }

    /// Load the rule set. Missing/corrupt files yield an empty set (never a
    /// hard failure — a bad validator file must not brick the app).
    pub fn load(&self) -> CustomRuleSet {
        match std::fs::read_to_string(self.path()) {
            Ok(raw) => toml::from_str::<CustomRuleSet>(&raw).unwrap_or_default(),
            Err(_) => CustomRuleSet::default(),
        }
    }

    pub fn save(&self, set: &CustomRuleSet) -> Result<(), String> {
        std::fs::create_dir_all(&self.config_dir)
            .map_err(|e| format!("create {}: {e}", self.config_dir.display()))?;
        let raw = toml::to_string_pretty(set).map_err(|e| e.to_string())?;
        let tmp = self.path().with_extension("toml.tmp");
        std::fs::write(&tmp, raw).map_err(|e| format!("write {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, self.path())
            .map_err(|e| format!("rename -> {}: {e}", self.path().display()))?;
        Ok(())
    }

    pub fn list(&self) -> Vec<CustomRule> {
        self.load().rules
    }

    /// Insert or replace a rule (matched by id).
    pub fn upsert(&self, rule: CustomRule) -> Result<(), String> {
        if rule.id.trim().is_empty() {
            return Err("rule id cannot be empty".into());
        }
        if rule.field.trim().is_empty() {
            return Err("rule field cannot be empty".into());
        }
        let mut set = self.load();
        match set.rules.iter_mut().find(|r| r.id == rule.id) {
            Some(existing) => *existing = rule,
            None => set.rules.push(rule),
        }
        self.save(&set)
    }

    pub fn remove(&self, id: &str) -> Result<(), String> {
        let mut set = self.load();
        let before = set.rules.len();
        set.rules.retain(|r| r.id != id);
        if set.rules.len() == before {
            return Err(format!("unknown custom validator `{id}`"));
        }
        self.save(&set)
    }
}

/// Walk a dot path into a JSON value (`network.port`, `paths.server_dir`,
/// `mods.active_mods.0`).
fn resolve<'a>(mut v: &'a serde_json::Value, field: &str) -> Option<&'a serde_json::Value> {
    for seg in field.split('.') {
        v = match v {
            serde_json::Value::Object(m) => m.get(seg)?,
            serde_json::Value::Array(a) => a.get(seg.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(v)
}

fn as_len(v: &serde_json::Value) -> Option<usize> {
    match v {
        serde_json::Value::String(s) => Some(s.chars().count()),
        serde_json::Value::Array(a) => Some(a.len()),
        _ => None,
    }
}

/// Evaluate one rule against the config's JSON projection.
/// `Ok(())` = passed, `Err(message)` = violation.
pub fn evaluate_rule(rule: &CustomRule, cfg: &serde_json::Value) -> Result<(), String> {
    let default_msg = |why: &str| {
        rule.message
            .clone()
            .unwrap_or_else(|| format!("`{}` {}", rule.field, why))
    };

    let val = resolve(cfg, &rule.field)
        .ok_or_else(|| rule.message.clone().unwrap_or_else(|| {
            format!("campo desconocido `{}`", rule.field)
        }))?;

    match rule.kind.as_str() {
        "not_empty" => {
            let empty = match val {
                serde_json::Value::String(s) => s.trim().is_empty(),
                serde_json::Value::Array(a) => a.is_empty(),
                serde_json::Value::Null => true,
                _ => false,
            };
            if empty { Err(default_msg("no puede estar vacío")) } else { Ok(()) }
        }
        "min_length" => {
            let min = rule.min_len.ok_or("min_length requiere `min_len`")?;
            match as_len(val) {
                Some(n) if n >= min => Ok(()),
                Some(_) => Err(default_msg(&format!("debe tener al menos {min} caracteres"))),
                None => Err(default_msg("no admite longitud")),
            }
        }
        "max_length" => {
            let max = rule.max_len.ok_or("max_length requiere `max_len`")?;
            match as_len(val) {
                Some(n) if n <= max => Ok(()),
                Some(_) => Err(default_msg(&format!("no puede superar {max} caracteres"))),
                None => Err(default_msg("no admite longitud")),
            }
        }
        "range" => {
            let n = val.as_f64().ok_or_else(|| default_msg("no es numérico"))?;
            if let Some(min) = rule.min { if n < min { return Err(default_msg(&format!("debe ser ≥ {min}"))); } }
            if let Some(max) = rule.max { if n > max { return Err(default_msg(&format!("debe ser ≤ {max}"))); } }
            Ok(())
        }
        "one_of" => {
            let allowed = rule.allowed.as_ref().ok_or("one_of requiere `allowed`")?;
            let s = val.as_str().ok_or_else(|| default_msg("no es una cadena"))?;
            if allowed.iter().any(|a| a == s) {
                Ok(())
            } else {
                Err(default_msg(&format!("debe ser uno de {:?}", allowed)))
            }
        }
        "contains" => {
            let needle = rule.value.as_ref().ok_or("contains requiere `value`")?;
            match val.as_str() {
                Some(s) if s.contains(needle.as_str()) => Ok(()),
                Some(_) => Err(default_msg(&format!("debe contener `{needle}`"))),
                None => Err(default_msg("no es una cadena")),
            }
        }
        "starts_with" => {
            let p = rule.value.as_ref().ok_or("starts_with requiere `value`")?;
            match val.as_str() {
                Some(s) if s.starts_with(p.as_str()) => Ok(()),
                Some(_) => Err(default_msg(&format!("debe empezar por `{p}`"))),
                None => Err(default_msg("no es una cadena")),
            }
        }
        "ends_with" => {
            let p = rule.value.as_ref().ok_or("ends_with requiere `value`")?;
            match val.as_str() {
                Some(s) if s.ends_with(p.as_str()) => Ok(()),
                Some(_) => Err(default_msg(&format!("debe terminar en `{p}`"))),
                None => Err(default_msg("no es una cadena")),
            }
        }
        "numeric" => match val.as_str() {
            Some(s) if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) => Ok(()),
            Some(_) => Err(default_msg("debe ser numérico")),
            None => Err(default_msg("no es una cadena")),
        },
        "must_be_true" => match val.as_bool() {
            Some(true) => Ok(()),
            _ => Err(default_msg("debe estar activado")),
        },
        "must_be_false" => match val.as_bool() {
            Some(false) => Ok(()),
            _ => Err(default_msg("debe estar desactivado")),
        },
        other => Err(format!(
            "custom validator `{}`: tipo desconocido `{other}`",
            rule.id
        )),
    }
}

/// Run every rule and collect the violations as canonical [`ValidationError`]s.
pub fn evaluate_rules(config: &ServerConfig, rules: &[CustomRule]) -> Vec<ValidationError> {
    let Ok(cfg) = serde_json::to_value(config) else { return vec![] };
    rules
        .iter()
        .filter_map(|r| {
            evaluate_rule(r, &cfg)
                .err()
                .map(|message| ValidationError { validator: r.label(), message })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_value() -> serde_json::Value {
        serde_json::to_value(ServerConfig::default()).unwrap()
    }

    #[test]
    fn not_empty_and_range() {
        let cfg = cfg_value();
        let mut config = ServerConfig::default();
        config.gameplay.max_players = 3;

        let empty_name = CustomRule {
            id: "name".into(), field: "identification.session_name".into(),
            kind: "not_empty".into(), message: None, min: None, max: None,
            min_len: None, max_len: None, allowed: None, value: None,
        };
        assert!(evaluate_rule(&empty_name, &serde_json::to_value(&config).unwrap()).is_err());

        let min_players = CustomRule {
            id: "minp".into(), field: "gameplay.max_players".into(),
            kind: "range".into(), min: Some(5.0), max: Some(200.0),
            message: None, min_len: None, max_len: None, allowed: None, value: None,
        };
        assert!(evaluate_rule(&min_players, &serde_json::to_value(&config).unwrap()).is_err());
        let _ = cfg;

        config.gameplay.max_players = 70;
        assert!(evaluate_rule(&min_players, &serde_json::to_value(&config).unwrap()).is_ok());
    }

    #[test]
    fn unknown_field_fails_loudly() {
        let rule = CustomRule {
            id: "x".into(), field: "network.does_not_exist".into(),
            kind: "not_empty".into(), message: None, min: None, max: None,
            min_len: None, max_len: None, allowed: None, value: None,
        };
        let err = evaluate_rule(&rule, &cfg_value()).unwrap_err();
        assert!(err.contains("desconocido"));
    }

    #[test]
    fn store_roundtrip_upsert_and_remove() {
        let dir = std::env::temp_dir().join(format!("ark-cv-test-{}", uuid::Uuid::new_v4().simple()));
        let _ = std::fs::create_dir_all(&dir);
        let store = CustomValidatorStore::new(&dir);
        let rule = CustomRule {
            id: "r1".into(), field: "network.port".into(), kind: "range".into(),
            min: Some(1024.0), max: Some(65535.0), message: Some("puerto válido".into()),
            min_len: None, max_len: None, allowed: None, value: None,
        };
        store.upsert(rule.clone()).unwrap();
        assert_eq!(store.list().len(), 1);
        // upsert replaces, not appends
        store.upsert(CustomRule { message: Some("otro".into()), ..rule }).unwrap();
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].message.as_deref(), Some("otro"));
        store.remove("r1").unwrap();
        assert!(store.list().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
