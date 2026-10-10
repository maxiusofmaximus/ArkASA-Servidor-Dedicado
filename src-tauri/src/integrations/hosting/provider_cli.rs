//! One-step bash runners for each provider.
//!
//! The operator copies one line into their workstation shell and the cloud
//! CLI does the auth (OAuth/device-code) before applying our cloud-init.

use super::cloud_init::shell_escape;
use super::{provision_script, HostProvider, HostTarget};

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
