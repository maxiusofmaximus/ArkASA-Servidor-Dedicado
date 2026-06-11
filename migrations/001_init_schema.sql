-- Migration: 001_init_schema.sql
-- Description: Initialize core database tables
-- Created: 2026-06-11

-- ============= CONFIG SNAPSHOTS =============
CREATE TABLE IF NOT EXISTS config_snapshots (
  id TEXT PRIMARY KEY,
  version INTEGER NOT NULL UNIQUE,
  config_json TEXT NOT NULL,
  checksum TEXT NOT NULL,
  created_at TEXT NOT NULL,
  created_by TEXT NOT NULL,
  description TEXT,
  size_bytes INTEGER,
  CONSTRAINT version_positive CHECK (version > 0)
);

CREATE INDEX IF NOT EXISTS idx_config_version ON config_snapshots(version DESC);
CREATE INDEX IF NOT EXISTS idx_config_created ON config_snapshots(created_at DESC);

-- ============= AUDIT LOGS =============
CREATE TABLE IF NOT EXISTS audit_logs (
  id TEXT PRIMARY KEY,
  action TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  changes TEXT,
  status TEXT NOT NULL,
  user TEXT NOT NULL,
  ip_address TEXT,
  created_at TEXT NOT NULL,
  duration_ms INTEGER
);

CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action);

-- ============= SERVER ACTIVITY LOGS =============
CREATE TABLE IF NOT EXISTS server_activity_logs (
  id TEXT PRIMARY KEY,
  event_type TEXT NOT NULL,
  message TEXT NOT NULL,
  severity TEXT NOT NULL,
  created_at TEXT NOT NULL,
  source TEXT,
  metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_activity_created ON server_activity_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_activity_severity ON server_activity_logs(severity);
CREATE INDEX IF NOT EXISTS idx_activity_type ON server_activity_logs(event_type);

-- ============= SERVER METRICS =============
CREATE TABLE IF NOT EXISTS server_metrics (
  id TEXT PRIMARY KEY,
  timestamp TEXT NOT NULL,
  cpu_percent REAL,
  memory_mb INTEGER,
  memory_percent REAL,
  network_in_mbps REAL,
  network_out_mbps REAL,
  player_count INTEGER,
  fps INTEGER,
  server_uptime_seconds INTEGER,
  disk_usage_percent REAL
);

CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON server_metrics(timestamp DESC);

-- ============= SETTINGS =============
CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  updated_by TEXT
);

-- ============= BACKUP HISTORY =============
CREATE TABLE IF NOT EXISTS backup_history (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  config_version INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  created_by TEXT NOT NULL,
  size_bytes INTEGER,
  description TEXT,
  restore_count INTEGER DEFAULT 0,
  last_restored_at TEXT,
  FOREIGN KEY(config_version) REFERENCES config_snapshots(version)
);

CREATE INDEX IF NOT EXISTS idx_backup_created ON backup_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_backup_version ON backup_history(config_version);

-- ============= VALIDATION ERRORS HISTORY =============
CREATE TABLE IF NOT EXISTS validation_errors (
  id TEXT PRIMARY KEY,
  timestamp TEXT NOT NULL,
  field TEXT NOT NULL,
  message TEXT NOT NULL,
  code TEXT NOT NULL,
  config_version INTEGER,
  user TEXT
);

CREATE INDEX IF NOT EXISTS idx_validation_timestamp ON validation_errors(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_validation_code ON validation_errors(code);

-- ============= SERVER STATE =============
CREATE TABLE IF NOT EXISTS server_state (
  id TEXT PRIMARY KEY DEFAULT 'singleton',
  is_running BOOLEAN DEFAULT 0,
  process_id INTEGER,
  started_at TEXT,
  stopped_at TEXT,
  last_status_check TEXT,
  last_config_applied TEXT,
  CONSTRAINT only_one_row CHECK (id = 'singleton')
);

-- Initialize server state
INSERT OR IGNORE INTO server_state (id) VALUES ('singleton');

-- ============= INITIALIZATION SETTINGS =============
INSERT OR IGNORE INTO settings (key, value, updated_at, updated_by)
VALUES
  ('db_version', '1', datetime('now'), 'system'),
  ('app_version', '2.0.0', datetime('now'), 'system'),
  ('initialized_at', datetime('now'), datetime('now'), 'system');
