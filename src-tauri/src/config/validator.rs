use crate::config::schema::ServerConfig;
use crate::error::{Error, Result};
use async_trait::async_trait;

#[async_trait]
pub trait ConfigValidator: Send + Sync {
    async fn validate(&self, config: &ServerConfig) -> Result<()>;
    fn name(&self) -> &str;
}

pub struct CompositeValidator {
    validators: Vec<Box<dyn ConfigValidator>>,
}

impl CompositeValidator {
    pub fn new() -> Self {
        Self {
            validators: vec![
                Box::new(PortValidator),
                Box::new(PasswordValidator),
                Box::new(PlayerCountValidator),
                Box::new(ModValidator),
                Box::new(PathValidator),
                Box::new(MultiplierValidator),
            ],
        }
    }

    pub fn with_validator(mut self, validator: Box<dyn ConfigValidator>) -> Self {
        self.validators.push(validator);
        self
    }

    pub async fn validate(&self, config: &ServerConfig) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        for validator in &self.validators {
            if let Err(e) = validator.validate(config).await {
                errors.push(ValidationError {
                    validator: validator.name().to_string(),
                    message: e.to_string(),
                });
            }
        }

        if errors.is_empty() {
            Ok(ValidationResult { valid: true, errors: vec![] })
        } else {
            Ok(ValidationResult { valid: false, errors })
        }
    }
}

impl Default for CompositeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationError {
    pub validator: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

// ============= VALIDATORS =============

struct PortValidator;

#[async_trait]
impl ConfigValidator for PortValidator {
    async fn validate(&self, config: &ServerConfig) -> Result<()> {
        let port = config.network.port;
        if port < 1024 {
            return Err(Error::field_error(
                "network.port",
                &format!("Port {} must be between 1024 and 65535", port),
                "INVALID_PORT",
            ));
        }

        let query_port = config.network.query_port;
        if query_port < 1024 {
            return Err(Error::field_error(
                "network.query_port",
                &format!("Query port {} must be between 1024 and 65535", query_port),
                "INVALID_QUERY_PORT",
            ));
        }

        if port == query_port || port == config.network.rcon_port {
            return Err(Error::field_error(
                "network.port",
                "Port numbers must be unique",
                "DUPLICATE_PORTS",
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "PortValidator"
    }
}

struct PasswordValidator;

#[async_trait]
impl ConfigValidator for PasswordValidator {
    async fn validate(&self, config: &ServerConfig) -> Result<()> {
        if config.identification.admin_password.is_empty() {
            return Err(Error::field_error(
                "identification.admin_password",
                "Admin password cannot be empty",
                "EMPTY_PASSWORD",
            ));
        }

        if config.identification.admin_password == "changeme"
            || config.identification.admin_password == "password"
        {
            return Err(Error::field_error(
                "identification.admin_password",
                "Please change the default admin password",
                "DEFAULT_PASSWORD",
            ));
        }

        if config.identification.admin_password.len() < 4 {
            return Err(Error::field_error(
                "identification.admin_password",
                "Admin password must be at least 4 characters",
                "SHORT_PASSWORD",
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "PasswordValidator"
    }
}

struct PlayerCountValidator;

#[async_trait]
impl ConfigValidator for PlayerCountValidator {
    async fn validate(&self, config: &ServerConfig) -> Result<()> {
        let max_players = config.gameplay.max_players;
        if max_players < 1 || max_players > 1000 {
            return Err(Error::field_error(
                "gameplay.max_players",
                &format!("Max players {} must be between 1 and 1000", max_players),
                "INVALID_PLAYER_COUNT",
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "PlayerCountValidator"
    }
}

struct ModValidator;

#[async_trait]
impl ConfigValidator for ModValidator {
    async fn validate(&self, config: &ServerConfig) -> Result<()> {
        for mod_id in &config.mods.active_mods {
            if mod_id.trim().is_empty() {
                return Err(Error::field_error(
                    "mods.active_mods",
                    "Mod ID cannot be empty",
                    "EMPTY_MOD_ID",
                ));
            }

            if mod_id == "0" {
                return Err(Error::field_error(
                    "mods.active_mods",
                    "Mod ID 0 is invalid",
                    "INVALID_MOD_ID_ZERO",
                ));
            }

            if !mod_id.chars().all(|c| c.is_numeric()) {
                return Err(Error::field_error(
                    "mods.active_mods",
                    &format!("Mod ID '{}' must be numeric", mod_id),
                    "NON_NUMERIC_MOD_ID",
                ));
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "ModValidator"
    }
}

struct PathValidator;

#[async_trait]
impl ConfigValidator for PathValidator {
    async fn validate(&self, config: &ServerConfig) -> Result<()> {
        let paths = &config.paths;

        if paths.server_dir.is_empty() {
            return Err(Error::field_error(
                "paths.server_dir",
                "Server directory cannot be empty",
                "EMPTY_SERVER_DIR",
            ));
        }

        if !paths.server_dir.starts_with("C:\\") && !paths.server_dir.starts_with("D:\\") {
            return Err(Error::field_error(
                "paths.server_dir",
                "Server directory should be on a local drive",
                "INVALID_SERVER_PATH",
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "PathValidator"
    }
}

struct MultiplierValidator;

#[async_trait]
impl ConfigValidator for MultiplierValidator {
    async fn validate(&self, config: &ServerConfig) -> Result<()> {
        let multipliers = &config.multipliers;

        if multipliers.xp_multiplier <= 0.0 || multipliers.xp_multiplier > 100.0 {
            return Err(Error::field_error(
                "multipliers.xp_multiplier",
                "XP multiplier must be between 0.1 and 100",
                "INVALID_MULTIPLIER",
            ));
        }

        if multipliers.taming_speed_multiplier <= 0.0 {
            return Err(Error::field_error(
                "multipliers.taming_speed_multiplier",
                "Taming speed multiplier must be positive",
                "INVALID_MULTIPLIER",
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "MultiplierValidator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_port_validation() {
        let validator = PortValidator;
        let mut config = ServerConfig::default();

        config.network.port = 999;
        assert!(validator.validate(&config).await.is_err());

        config.network.port = 7777;
        assert!(validator.validate(&config).await.is_ok());
    }

    #[tokio::test]
    async fn test_password_validation() {
        let validator = PasswordValidator;
        let mut config = ServerConfig::default();

        config.identification.admin_password = "changeme".to_string();
        assert!(validator.validate(&config).await.is_err());

        config.identification.admin_password = "SecurePass123".to_string();
        assert!(validator.validate(&config).await.is_ok());
    }

    #[tokio::test]
    async fn test_composite_validation() {
        let validator = CompositeValidator::new();
        let mut config = ServerConfig::default();

        config.network.port = 999;
        config.identification.admin_password = "changeme".to_string();

        let result = validator.validate(&config).await.unwrap();
        assert!(!result.valid);
        assert!(result.errors.len() > 0);
    }
}
