#![allow(dead_code, unused_imports)]

// Server log reading and tailing
use crate::error::Result;
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub raw: String,
}

pub struct LogReader {
    log_path: std::path::PathBuf,
}

impl LogReader {
    pub fn new(log_path: impl AsRef<Path>) -> Self {
        Self {
            log_path: log_path.as_ref().to_path_buf(),
        }
    }

    /// Read last N lines from log file
    pub async fn read_last_n(&self, n: usize) -> Result<Vec<LogEntry>> {
        if !self.log_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&self.log_path)
            .await
            .map_err(|e| crate::error::Error::IoError(e))?;

        let lines: Vec<&str> = content.lines().collect();
        let start = if lines.len() > n { lines.len() - n } else { 0 };

        let entries: Vec<LogEntry> = lines[start..]
            .iter()
            .filter_map(|line| Self::parse_log_line(line))
            .collect();

        Ok(entries)
    }

    /// Parse ARK server log line
    /// Format: [YYYY.MM.DD-HH:MM:SS:UUU][CHANNEL][LEVEL] Message
    fn parse_log_line(line: &str) -> Option<LogEntry> {
        // Example: [2026.06.11-14.30.45.123][000]LogOnline: Warning: Something happened
        let parts: Vec<&str> = line.split(']').collect();
        if parts.len() < 3 {
            return None;
        }

        let timestamp = parts[0].trim_start_matches('[').to_string();
        let _channel = parts[1].trim_start_matches('[').to_string();

        let rest = parts[2..].join("]");
        
        let (level, message) = if let Some(first_colon) = rest.find(':') {
            let channel_part = rest[..first_colon].trim();
            let remainder = rest[first_colon + 1..].trim();
            
            if let Some(second_colon) = remainder.find(':') {
                let level_part = remainder[..second_colon].trim();
                let msg_part = remainder[second_colon + 1..].trim();
                
                // Check if the level_part matches any standard log level
                let known_levels = ["warning", "error", "info", "display", "fatal", "success", "log", "verbose"];
                if known_levels.contains(&level_part.to_lowercase().as_str()) {
                    (level_part.to_string(), msg_part.to_string())
                } else {
                    (channel_part.to_string(), remainder.to_string())
                }
            } else {
                (channel_part.to_string(), remainder.to_string())
            }
        } else {
            ("INFO".to_string(), rest.trim().to_string())
        };

        Some(LogEntry {
            timestamp,
            level,
            message,
            raw: line.to_string(),
        })
    }

    /// Search logs for pattern
    pub async fn search(&self, pattern: &str) -> Result<Vec<LogEntry>> {
        if !self.log_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&self.log_path)
            .await
            .map_err(|e| crate::error::Error::IoError(e))?;

        let entries: Vec<LogEntry> = content
            .lines()
            .filter(|line| line.to_lowercase().contains(&pattern.to_lowercase()))
            .filter_map(Self::parse_log_line)
            .collect();

        Ok(entries)
    }

    /// Get logs with specific level
    pub async fn get_by_level(&self, level: &str) -> Result<Vec<LogEntry>> {
        if !self.log_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&self.log_path)
            .await
            .map_err(|e| crate::error::Error::IoError(e))?;

        let entries: Vec<LogEntry> = content
            .lines()
            .filter_map(Self::parse_log_line)
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect();

        Ok(entries)
    }

    /// Count events by level
    pub async fn count_by_level(&self) -> Result<std::collections::HashMap<String, usize>> {
        if !self.log_path.exists() {
            return Ok(std::collections::HashMap::new());
        }

        let content = fs::read_to_string(&self.log_path)
            .await
            .map_err(|e| crate::error::Error::IoError(e))?;

        let mut counts = std::collections::HashMap::new();
        for line in content.lines() {
            if let Some(entry) = Self::parse_log_line(line) {
                *counts.entry(entry.level.to_uppercase()).or_insert(0) += 1;
            }
        }

        Ok(counts)
    }

    /// Check for critical errors
    pub async fn has_errors(&self) -> Result<bool> {
        let entries = self.get_by_level("error").await?;
        Ok(!entries.is_empty())
    }

    /// Get recent errors
    pub async fn get_recent_errors(&self, limit: usize) -> Result<Vec<LogEntry>> {
        let entries = self.get_by_level("error").await?;
        Ok(entries.into_iter().rev().take(limit).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_line() {
        let line = "[2026.06.11-14.30.45.123][000]LogOnline: Warning: Something happened";
        let entry = LogReader::parse_log_line(line);

        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.level, "Warning");
        assert!(entry.message.contains("Something happened"));
    }

    #[test]
    fn test_parse_log_with_colons_in_message() {
        let line = "[2026.06.11-14.30.45.123][000]LogNet: Error: Connection failed: timeout";
        let entry = LogReader::parse_log_line(line);

        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.level, "Error");
        assert!(entry.message.contains("Connection failed"));
    }
}
