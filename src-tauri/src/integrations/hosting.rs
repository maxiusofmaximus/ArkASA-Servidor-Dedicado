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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

// ─────────────────────────────────────────────────────────────────────────────

fn env_export_block(env: &HashMap<String, String>) -> String {
    let mut out = String::new();
    for (k, v) in env {
        out.push_str(&format!("export {}='{}'\n", k, shell_escape(v)));
    }
    out
}

fn shell_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

const SYSTEMD_UNIT: &str = r#"[Unit]
Description=ARK ASA Dedicated Server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=arkasa
WorkingDirectory=/home/arkasa/server
EnvironmentFile=/home/arkasa/server.env
ExecStart=/home/arkasa/server/ShooterGame/Binaries/Linux/ShooterGameServer TheIsland_WP
Restart=on-failure
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
"#;

const STEAMCMD_INSTALL: &str = r#"
mkdir -p /home/arkasa/steamcmd && cd /home/arkasa/steamcmd
if [ ! -f ./steamcmd.sh ]; then
    curl -sqL "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz" | tar xzf -
fi
sudo -u arkasa ./steamcmd.sh +login anonymous \
    +force_install_dir /home/arkasa/server \
    +app_update 2430930 validate +quit
"#;

fn common_setup(target: &HostTarget, server_zip_url: &str) -> String {
    let env_block = env_export_block(&target.env);
    let disk_block = if target.disk_gb >= 50 {
        format!("echo 'INFO: extra disk space requested ({} GB) — ensure the cloud volume is mounted under /home/arkasa'", target.disk_gb)
    } else {
        "# <50 GB requested — fine for one map".to_string()
    };

    format!(
        r#"#!/bin/bash
set -Eeuo pipefail
# -------------------------------------------------------------------------
# ARK ASA provisioning generated by ARK ASA Configuration Manager
# Provider : {provider}
# Region   : {region}
# Disk     : {disk} GB
# -------------------------------------------------------------------------
{env_block}{disk_block}

# 1. System prep
apt-get update -y
DEBIAN_FRONTEND=noninteractive apt-get install -y lib32gcc-s1 lib32stdc++6 \
    libc6-i386 libcurl4-gnutls-dev:i386 libsdl2-2.0-0:i386 wget curl ca-certificates \
    libsdl2-2.0-0 screen unzip

# 2. ARK user
id arkasa >/dev/null 2>&1 || useradd -m -d /home/arkasa -s /bin/bash arkasa

# 3. ARK server binary
{steamcmd_install}

# 4. Operator-supplied backup bundle (TOML config + saved world zip)
[ -f /tmp/arkasa-bundle.zip ] || curl -sL '{zip}' -o /tmp/arkasa-bundle.zip
sudo -u arkasa mkdir -p /home/arkasa/server
sudo -u arkasa unzip -o /tmp/arkasa-bundle.zip -d /home/arkasa/server/

# 5. systemd unit
cat >/etc/systemd/system/arkasa.service <<'EOF'
{systemd_unit}EOF

systemctl daemon-reload
systemctl enable --now arkasa.service

echo "✓ ARK ASA is now running under systemd. Log via 'journalctl -u arkasa -f'." 
"#,
        provider = target.provider.label(),
        region = target.region,
        disk = target.disk_gb,
        env_block = env_block,
        disk_block = disk_block,
        zip = server_zip_url,
        steamcmd_install = STEAMCMD_INSTALL,
        systemd_unit = SYSTEMD_UNIT.replace("\"\\", "").replace("\\\\n", "\\n"),
    )
}

fn oracle_cloud_init(target: &HostTarget, url: &str) -> String {
    format!(
        "#cloud-config\nruncmd:\n - bash -c 'set -e; {}'\n",
        common_setup(target, url).replace('\n', "\\n")
    )
}

fn hetzner_user_data(target: &HostTarget, url: &str) -> String {
    format!("#!/bin/bash\n{}\n", common_setup(target, url))
}

fn digitalocean_user_data(target: &HostTarget, url: &str) -> String {
    format!("#!/bin/bash\n{}\n", common_setup(target, url))
}

fn selfhosted_bootstrap(target: &HostTarget, url: &str) -> String {
    common_setup(target, url)
}

fn aws_user_data(target: &HostTarget, url: &str) -> String {
    format!(
        "Content-Type: multipart/mixed; boundary=\"//\"\nMIME-Version: 1.0\n\n--//\nContent-Type: text/cloud-config; charset=\"us-ascii\"\nMIME-Version: 1.0\nContent-Transfer-Encoding: 7bit\nContent-Disposition: attachment; filename=\"cloud-config.txt\"\n\n#cloud-config\nruncmd:\n - bash -c '{}'\n--//\n",
        common_setup(target, url).replace('\n', "\\n").replace("'", "\\'")
    )
}

fn azure_user_data(target: &HostTarget, url: &str) -> String {
    format!("#!/bin/bash\n{}\n", common_setup(target, url))
}

fn gcp_startup(target: &HostTarget, url: &str) -> String {
    format!("#!/bin/bash\n{}\n", common_setup(target, url))
}

// ═══════════════════════════════════════════════════════════════════════════
// One-step bash runners for each provider — the operator copies one
// line into their workstation shell and the cloud CLI does the auth
// (OAuth/device-code) before applying our cloud-init.
// ═══════════════════════════════════════════════════════════════════════════

/// Render a **single-file bash wrapper** that the operator can run on
/// their workstation to launch a new VPS for `target`.
/// The wrapper:
///
/// 1. Writes the cloud-init (returned by `provision_script`) to a tmp
///    file so the CLI can consume it via `--user-data <path>` etc.
/// 2. Calls the official CLI for the chosen provider. Auth is the
///    provider's responsibility (HCloud token, AWS SSO, `az login`, …)
///
/// Returns a multi-line heredoc-style script **without leading `$`**.
pub fn render_provider_run_script(target: &HostTarget, server_zip_url: &str) -> Result<String, String> {
    if target.ssh_key_path.trim().is_empty() {
        return Err("ssh_key_path is required to render the runner".into());
    }
    if server_zip_url.trim().is_empty() {
        return Err("server_zip_url is required (no empty bundle allowed)".into());
    }
    let cloud_init = provision_script(target, server_zip_url);
    let tmp_init   = format!("/tmp/arkasa-init-{}.sh", unix_nano_now());

    // Pick the bash flavour per provider. We deliberately use *the same
    // path for AWS, Azure and GCP* that the operator will already have
    // configured (e.g. `aws configure sso`, `az login`, `gcloud auth
    // login`). We do NOT handle tokens in this app.
    let body = match target.provider {
        HostProvider::Hetzner       => render_hetzner(target, &tmp_init),
        HostProvider::Digitalocean  => render_doctl(target, &tmp_init),
        HostProvider::AwsEc2        => render_aws(target, &tmp_init),
        HostProvider::GcpCompute    => render_gcloud(target, &tmp_init),
        HostProvider::AzureVm       => render_az(target, &tmp_init),
        HostProvider::Oracle        => render_oci(target, &tmp_init),
        HostProvider::SelfHosted    => render_selfhosted(target, &tmp_init),
    };
    Ok(format!(
        "#!/bin/bash\nset -euo pipefail\n\n# 1. Cloud-init -> tmp\nmkdir -p \"$(dirname '{tmp_init}')\"\ncat >'{tmp_init}' <<'__ARKASA_INIT__'\n{cloud_init}\n__ARKASA_INIT__\n\n# 2. Provider CLI — already authenticated in your shell.\n{body}\n\n# 3. Echo back final SSH endpoint so you can write it into the app.\necho\necho '✓ Done. USB the SSH output back into the app:' \\\n     \"ssh {ssh_user}@$(echo \"$0\")\"\n",
        tmp_init = tmp_init,
        cloud_init = cloud_init,
        body = body,
        ssh_user = target.ssh_user,
    ))
}

#[cfg(test)]
fn _assert_unix_nano_not_callable_in_lib() {}

fn unix_nano_now() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos()).unwrap_or(0)
}

fn render_hetzner(target: &HostTarget, tmp_init: &str) -> String {
    format!(
        r#"
# Hetzner Cloud (hcloud CLI must be installed: brew install hcloud-cli)
hcloud ssh-key describe --with-fingerprint --format json \
  --name "{ssh_key_path_basename}" || \
  hcloud ssh-key create --name "{ssh_key_path_basename}" \
    --public-key-from-file "{ssh_key_path}.pub"

hcloud server create \
  --name arkasa-{region}-{disk_gb} \
  --image ubuntu-24.04 \
  --type cax11 \
  --location {region} \
  --ssh-key "{ssh_key_path_basename}" \
  --user-data-from-file {tmp_init} \
  --start-after-create
"#,
        ssh_key_path = shell_escape(&target.ssh_key_path),
        ssh_key_path_basename = shell_escape(std::path::Path::new(&target.ssh_key_path)
            .file_name().and_then(|s| s.to_str()).unwrap_or("arkasa")),
        region = shell_escape(&target.region),
        disk_gb = target.disk_gb,
        tmp_init = tmp_init,
    )
}

fn render_doctl(target: &HostTarget, tmp_init: &str) -> String {
    format!(
        r#"
# DigitalOcean (doctl CLI must be installed: brew install doctl, doctl auth init)
SIZE=s-2vcpu-4gb
doctl compute ssh-key import {ssh_key_path_basename} \
  --public-key-file "{ssh_key_path}.pub"

doctl compute droplet create arkasa-{region}-{disk_gb} \
  --image ubuntu-24-04-x64 \
  --size "$SIZE" \
  --region {region} \
  --ssh-keys "{ssh_key_path_basename}" \
  --user-data-file {tmp_init} \
  --wait
"#,
        ssh_key_path = shell_escape(&target.ssh_key_path),
        ssh_key_path_basename = shell_escape(std::path::Path::new(&target.ssh_key_path)
            .file_name().and_then(|s| s.to_str()).unwrap_or("arkasa")),
        region = shell_escape(&target.region),
        disk_gb = target.disk_gb,
        tmp_init = tmp_init,
    )
}

fn render_aws(target: &HostTarget, tmp_init: &str) -> String {
    // AWS user-data has a 16 KB ceiling; our cloud-init is fine, but we
    // avoid quoting the bash inline. We'll just pass the file. The
    // multipart wrapper is constructed inside provision_script().
    format!(
        r#"
# AWS EC2 (aws CLI v2; AWS_PROFILE must be authentic)
aws ec2 run-instances \
  --instance-type t4g.medium \
  --image-id resolve:ssm:/aws/service/canonical/ubuntu/server/24.04/stable/current/arm64/hvm/ebs-gp3/ami-id \
  --region {region} \
  --block-device-mappings "[{{ \"DeviceName\": \"/dev/sda1\", \"Ebs\": {{ \"VolumeSize\": {disk_gb}, \"VolumeType\": \"gp3\", \"DeleteOnTermination\": true }}}}]" \
  --tag-specifications "ResourceType=instance,Tags=[{{Key=Name,Value=arkasa-{region}-{disk_gb}}},{{Key=Project,Value=arkasa}}]" \
  --key-name "{ssh_key_path_basename}" \
  --user-data file://{tmp_init} \
  --count 1

aws ec2 describe-instances --filters "Name=tag:Name,Values=arkasa-{region}-{disk_gb}" \
    --query "Reservations[0].Instances[0].PublicIpAddress" --output text
"#,
        region = shell_escape(&target.region),
        disk_gb = target.disk_gb,
        tmp_init = tmp_init,
        ssh_key_path_basename = shell_escape(std::path::Path::new(&target.ssh_key_path)
            .file_name().and_then(|s| s.to_str()).unwrap_or("arkasa")),
    )
}

fn render_gcloud(target: &HostTarget, tmp_init: &str) -> String {
    format!(
        r#"
# GCP Compute (gcloud CLI; gcloud auth login must be active)
gcloud compute instances create arkasa-{region}-{disk_gb} \
  --image-family ubuntu-2404-lts-arm64 \
  --image-project ubuntu-os-cloud \
  --machine-type e2-medium \
  --zone {region}-a \
  --boot-disk-size {disk_gb}GB \
  --metadata-from-file user-data={tmp_init} \
  --tags arkasa
"#,
        region = shell_escape(&target.region),
        disk_gb = target.disk_gb,
        tmp_init = tmp_init,
    )
}

fn render_az(target: &HostTarget, tmp_init: &str) -> String {
    format!(
        r#"
# Azure REST (az CLI + service-principal auth via AZURE_* env vars)
VMNAME="arkasa-{region}-{disk_gb}"
VMRG="{resource_group_default}"
az vm create \
  --resource-group "$VMRG" \
  --name "$VMNAME" \
  --image Canonical:ubuntu-24_04-lts:server-arm64:latest \
  --size Standard_B2ps_v2 \
  --location {region} \
  --admin-username {ssh_user} \
  --ssh-key-values "{ssh_key_path}.pub" \
  --os-disk-size-gb {disk_gb} \
  --custom-data "@{tmp_init}"
"#,
        region = shell_escape(&target.region),
        disk_gb = target.disk_gb,
        tmp_init = tmp_init,
        resource_group_default = if target.env.contains_key("AZURE_RG") {
            shell_escape(target.env.get("AZURE_RG").unwrap())
        } else {
            "arkasa-rg".into()
        },
        ssh_user = shell_escape(&target.ssh_user),
        ssh_key_path = shell_escape(&target.ssh_key_path),
    )
}

fn render_oci(target: &HostTarget, tmp_init: &str) -> String {
    let region   = shell_escape(&target.region);
    let disk_gb  = target.disk_gb;
    let tmp_init = shell_escape(tmp_init);
    // We build the JSON payload separately so the format! body stays
    // readable.
    let doc = format!(
        "Oracle launches require oci session authenticate first.\n\
         --compartment-id \"$COMPARTMENT_OCID\"\n\
         --availability-domain \"{region}-AD-1\"\n\
         --shape VM.Standard.A1.Flex\n\
         --shape-config '{{\"ocpus\":2,\"memoryInGBs\":12}}'\n\
         --image-id \"$(oci compute image list ...)\"\n\
         --metadata \"$USER_DATA_B64\"\n\
         --display-name \"arkasa-{region}-{disk_gb}\"\n\
         --assign-public-ip true\n\
         USER_DATA_B64=$(base64 -w0 {tmp_init})\n"
    );
    format!(
        "# Oracle Cloud Always-Free (oci CLI; oci session authenticate active)\n\
         COMPARTMENT_OCID=\"${{COMPARTMENT_OCID:-}}\"\n\
         if [ -z \"$COMPARTMENT_OCID\" ]; then echo \"Set COMPARTMENT_OCID env var to your Oracle compartment OCID.\" >&2; exit 1; fi\n\
         oci compute instance launch {body}",
        body = doc,
    )
}

fn render_selfhosted(target: &HostTarget, tmp_init: &str) -> String {
    format!(
        r#"
# Self-hosted: rsync the cloud-init over SSH and run it.
TARGET='{ssh_user}@{ssh_host}'
ssh -p {ssh_port} "$TARGET" 'mkdir -p /tmp/arkasa-init'
rsync -avz -e "ssh -p {ssh_port}" {tmp_init} "$TARGET:/tmp/arkasa-init/run.sh"
ssh -p {ssh_port} "$TARGET" 'chmod +x /tmp/arkasa-init/run.sh && sudo /tmp/arkasa-init/run.sh'
"#,
        ssh_user = shell_escape(&target.ssh_user),
        ssh_host = shell_escape(&target.ssh_host),
        ssh_port = target.ssh_port,
        tmp_init = tmp_init,
    )
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

    #[test]
    fn escape_handles_quotes() {
        // single quote in POSIX shell is escaped as '\''.
        assert_eq!(shell_escape("a'b"), "a'\\''b");
    }

    #[test]
    fn oracle_uses_cloud_init() {
        let mut target = HostTarget::new_self();
        target.provider = HostProvider::Oracle;
        assert!(provision_script(&target, "x").starts_with("#cloud-config"));
    }

    // ─── render_provider_run_script ──────────────────────────────────────────────

    #[test]
    fn render_run_script_for_hetzner_includes_hcloud() {
        let target = hetzner_target();
        let script = render_provider_run_script(&target, "https://example.com/ark.zip").unwrap();
        assert!(script.contains("hcloud server create"), "shell missing:\n{script}");
        assert!(script.contains("fsn1"));
        assert!(script.contains("cax11"));
        assert!(script.contains("--ssh-key"), "ssh key flag expected");
        assert!(script.contains("https://example.com/ark.zip"));
    }

    #[test]
    fn render_run_script_for_digitalocean_includes_docl() {
        let mut target = hetzner_target();
        target.provider = HostProvider::Digitalocean;
        let script = render_provider_run_script(&target, "https://x").unwrap();
        assert!(script.contains("doctl compute droplet create"));
        assert!(script.contains("--region fsn1"), "got:\n{script}");
        assert!(script.contains("s-2vcpu-4gb"), "size fallback constant expected");
        assert!(script.contains("--size \"$SIZE\""), "size interpolated via SIZE var, got:\n{script}");
    }

    #[test]
    fn render_run_script_for_aws_includes_marketplace() {
        let mut target = hetzner_target();
        target.provider = HostProvider::AwsEc2;
        let script = render_provider_run_script(&target, "https://x").unwrap();
        assert!(script.contains("aws ec2 run-instances"));
        assert!(script.contains("--instance-type t4g.medium"));
        assert!(script.contains("--region fsn1"));
        // We DO NOT pass user-data inline for AWS — too long. We write to /tmp first.
        assert!(script.contains("/tmp/arkasa-init-"));
    }

    #[test]
    fn render_run_script_for_gcp_oci_azure_oracle_use_official_cli() {
        let target = hetzner_target();
        let cases = [
            (HostProvider::GcpCompute, vec!["gcloud"]),
            (HostProvider::Oracle,     vec!["oci", "COMPARTMENT_OCID"]),
            (HostProvider::AzureVm,    vec!["az"]),
            (HostProvider::SelfHosted, vec!["rsync", "ssh"]),
        ];
        for (provider, must_contain) in cases {
            let mut t = target.clone();
            t.provider = provider;
            let script = render_provider_run_script(&t, "https://x").unwrap();
            assert!(!script.trim().is_empty(), "provider {:?} produced empty", provider);
            for needle in &must_contain {
                assert!(script.contains(needle),
                    "provider {:?} missing `{}`:\n{}", provider, needle, script);
            }
        }
    }

    #[test]
    fn render_run_script_writes_cloud_init_to_tmpfile() {
        let target = hetzner_target();
        let script = render_provider_run_script(&target, "https://x").unwrap();
        // Every shell wrapper should dump the cloud-init to /tmp first so the
        // CLI can hand it as --userdata-from-file (Hetzner) or
        // --custom-data file@xxx (Azure).
        assert!(script.contains("/tmp/arkasa-init-"), "tmp file marker expected in:\n{script}");
    }

    fn hetzner_target() -> HostTarget {
        let mut t = HostTarget::new_self();
        t.provider = HostProvider::Hetzner;
        t.region = "fsn1".into();
        t.ssh_user = "root".into();
        t.ssh_host = "1.2.3.4".into();
        t.ssh_port = 22;
        t.ssh_key_path = "~/.ssh/id_ed25519".into();
        t.disk_gb = 80;
        t
    }
}
