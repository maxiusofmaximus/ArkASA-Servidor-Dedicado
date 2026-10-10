//! Hosting adapter layer — first-class provisioning playbooks for any cloud
//! the operator owns. We do **not** provision servers from inside the
//! desktop app (that's a separate "deploy bot"); this module:
//!
//!   1. Generates cloud-init / startup scripts ARK ASA needs.
//!   2. Provides REST/SSE hooks for the operator to trigger deploys via
//!      their own existing Terraform / CLI tooling.
//!   3. Documents each provider in `docs/HOSTING.md`.
//!
//! All providers share the same `HostTarget` shape:
//!   - name (label)
//!   - ssh endpoint (host, port, user)
//!   - storage hint (S3, SBucket, …)
//!   - env vars shipped to the new VPS
//!
//! Adding a new provider only requires implementing `provision_script()`
//! which returns the shell/arminit/cloud-init file as a `String`.

mod cloud_init;
mod provider_cli;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use cloud_init::{
    aws_user_data, azure_user_data, digitalocean_user_data, gcp_startup, hetzner_user_data,
    oracle_cloud_init, selfhosted_bootstrap,
};
pub use provider_cli::render_provider_run_script;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum HostProvider {
    // Free + easy to subscribe
    Oracle,         // Always Free tier: 2 x ARM, 24 GB RAM
    Hetzner,        // Specialised game-server hosting
    Digitalocean,   // 1-click droplets
    #[default]
    SelfHosted,
    // Major clouds
    AwsEc2,
    AzureVm,
    GcpCompute,
}

impl HostProvider {
    pub fn label(self) -> &'static str {
        match self {
            HostProvider::Oracle => "Oracle Cloud Always-Free",
            HostProvider::Hetzner => "Hetzner Dedicated Server",
            HostProvider::Digitalocean => "DigitalOcean 1-Click",
            HostProvider::SelfHosted => "Self-hosted (Ansible/Local)",
            HostProvider::AwsEc2 => "AWS EC2 (t4g.medium)",
            HostProvider::AzureVm => "Azure VM (B2ps v2)",
            HostProvider::GcpCompute => "Google Compute e2-medium",
        }
    }
    pub fn all() -> &'static [HostProvider] {
        const ALL: &[HostProvider] = &[
            HostProvider::Oracle,
            HostProvider::Hetzner,
            HostProvider::Digitalocean,
            HostProvider::SelfHosted,
            HostProvider::AwsEc2,
            HostProvider::AzureVm,
            HostProvider::GcpCompute,
        ];
        ALL
    }
}

/// What the user wants their ARK ASA server stack to look like.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostTarget {
    pub provider: HostProvider,
    pub region:   String,    // "us-phoenix-1", "fsn1", "nyc3", "us-east-1", "taipeitw-1" ...
    pub ssh_user: String,    // "ubuntu", "opc", "ec2-user"
    pub ssh_host: String,    // public IPv4 or DNS
    pub ssh_port: u16,       // 22 default
    pub ssh_key_path: String, // path on the operator's workstation
    pub env:      HashMap<String, String>,
    pub disk_gb:  u32,
}

impl HostTarget {
    pub fn new_self() -> Self {
        Self {
            provider: HostProvider::SelfHosted,
            region: "on-prem".into(),
            ssh_user: "arkasa".into(),
            ssh_host: "127.0.0.1".into(),
            ssh_port: 22,
            ssh_key_path: "~/.ssh/id_ed25519".into(),
            env: Default::default(),
            disk_gb: 50,
        }
    }
}

/// Render a cloud-init that installs SteamCMD + downloads the dedicated
/// server, uploads the local TOML config and starts ARK with the right
/// command-line. The generated script is **idempotent** (re-running is safe).
pub fn provision_script(target: &HostTarget, server_zip_url: &str) -> String {
    match target.provider {
        HostProvider::Oracle => oracle_cloud_init(target, server_zip_url),
        HostProvider::Hetzner => hetzner_user_data(target, server_zip_url),
        HostProvider::Digitalocean => digitalocean_user_data(target, server_zip_url),
        HostProvider::SelfHosted => selfhosted_bootstrap(target, server_zip_url),
        HostProvider::AwsEc2 => aws_user_data(target, server_zip_url),
        HostProvider::AzureVm => azure_user_data(target, server_zip_url),
        HostProvider::GcpCompute => gcp_startup(target, server_zip_url),
    }
}

/// Tracks deployments made by the operator so the desktop UI can show a
/// status panel.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentLedger {
    pub entries: Vec<DeploymentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentEntry {
    pub provider:    HostProvider,
    pub region:      String,
    pub ssh_host:    String,
    pub deployed_at: i64,        // epoch seconds
    pub status:      String,     // "ok" | "pending" | "failed:<reason>"
}

impl DeploymentLedger {
    pub fn append(&mut self, target: &HostTarget) {
        self.entries.push(DeploymentEntry {
            provider: target.provider,
            region: target.region.clone(),
            ssh_host: target.ssh_host.clone(),
            deployed_at: chrono::Utc::now().timestamp(),
            status: "pending".into(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_providers_yield_nonempty_script() {
        let target = HostTarget::new_self();
        for provider in HostProvider::all() {
            let mut t = target.clone();
            t.provider = *provider;
            let script = provision_script(&t, "https://example.com/ark-bundle.zip");
            assert!(script.contains("ARK"));
            assert!(script.contains("systemd"));
        }
    }

    #[test]
    fn ledger_records_entry() {
        let mut ledger = DeploymentLedger::default();
        let target = HostTarget::new_self();
        ledger.append(&target);
        assert_eq!(ledger.entries.len(), 1);
        assert_eq!(ledger.entries[0].status, "pending");
    }
}
