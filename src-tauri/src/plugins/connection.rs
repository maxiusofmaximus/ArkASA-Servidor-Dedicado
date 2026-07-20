//! Connection plugins — VPS provider wrappers plus the Tailscale
//! network connector built on top of the
//! `hosting` module's existing per-provider render functions.
//!
//! Why this wrapper layer? Because the P0/P1 plugin-hub registry
//! wants homogeneous metadata (`id`, `label`, `description`,
//! `docs_url`) for every connection adapter, and the existing
//! `HostProvider` enum is a closed-form TODO-list rather than a
//! registered plugin record. This module is the soft
//! pluggability seam — we don't ship the provider CLIs
//! dynamically (the operator still runs the `hcloud`/`doctl`/`aws`
//! binaries), we just expose a discoverable catalog + a Tauri
//! command set the UI can render.
//!
//! Backwards rule: NOTHING in this layer spawns a process. Persisted
//! state spelling, runner shape, and credentials management stay
//! 100% under the existing `hosting.rs` umbrella. The only new
//! surface is the listing / metadata API.

use crate::integrations::hosting::{HostProvider, HostTarget};

/// Stable id of every connection plugin we recognise. Mapping is
/// 1:1 with `HostProvider` today; future plugin add/remove can be
/// driven through this enum without touching `HostProvider` (which
/// would change the React-side contracts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionPluginId {
    Oracle,
    Hetzner,
    Digitalocean,
    SelfHosted,
    AwsEc2,
    AzureVm,
    GcpCompute,
    Tailscale,
}

impl ConnectionPluginId {
    pub fn from_host_provider(p: HostProvider) -> Self {
        match p {
            HostProvider::Oracle       => Self::Oracle,
            HostProvider::Hetzner      => Self::Hetzner,
            HostProvider::Digitalocean => Self::Digitalocean,
            HostProvider::SelfHosted   => Self::SelfHosted,
            HostProvider::AwsEc2       => Self::AwsEc2,
            HostProvider::AzureVm      => Self::AzureVm,
            HostProvider::GcpCompute   => Self::GcpCompute,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Oracle       => "oracle",
            Self::Hetzner      => "hetzner",
            Self::Digitalocean => "digitalocean",
            Self::SelfHosted   => "selfhosted",
            Self::AwsEc2       => "aws_ec2",
            Self::AzureVm      => "azure_vm",
            Self::GcpCompute   => "gcp_compute",
            Self::Tailscale    => "tailscale",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "oracle"       => Self::Oracle,
            "hetzner"      => Self::Hetzner,
            "digitalocean" => Self::Digitalocean,
            "selfhosted"   => Self::SelfHosted,
            "aws_ec2"      => Self::AwsEc2,
            "azure_vm"     => Self::AzureVm,
            "gcp_compute"  => Self::GcpCompute,
            "tailscale"    => Self::Tailscale,
            _              => return None,
        })
    }
}

/// Stable metadata record the React Plugin Hub renders.
#[derive(Debug, Clone)]
pub struct ConnectionPluginView {
    pub id:           &'static str,
    pub label:        &'static str,
    pub description:  &'static str,
    pub free_tier:    bool,
    pub requires_cli: &'static [&'static str],
    pub requires_credentials: bool,
    pub docs_url:     &'static str,
}

/// Static catalog of every connection plugin. Adding a new provider
/// touches this table only — it doesn't require the React side to
/// learn anything new.
pub const CATALOG: &[ConnectionPluginView] = &[
    ConnectionPluginView {
        id: "oracle", label: "Oracle Cloud Always-Free",
        description: "2x ARM Ampere A1, 24 GB RAM. Free forever tier — slow sign-up but worth it.",
        free_tier: true,
        requires_cli: &["oci"],
        requires_credentials: true,
        docs_url: "https://docs.oracle.com/en-us/iaas/Content/API/SDKDocs/cliinstall.htm",
    },
    ConnectionPluginView {
        id: "hetzner", label: "Hetzner Dedicated Server",
        description: "Specialised game-server hosting. Cheap ccx/cx/ax ranges in EU/US.",
        free_tier: false,
        requires_cli: &["hcloud"],
        requires_credentials: true,
        docs_url: "https://github.com/hetznercloud/cli",
    },
    ConnectionPluginView {
        id: "digitalocean", label: "DigitalOcean 1-Click",
        description: "1-click droplets, SFO/NYC/SGP. Good starter.",
        free_tier: false,
        requires_cli: &["doctl"],
        requires_credentials: true,
        docs_url: "https://docs.digitalocean.com/reference/doctl/",
    },
    ConnectionPluginView {
        id: "selfhosted", label: "Self-hosted (Ansible/Local)",
        description: "Run on bare-metal — Pi 5, Intel NUC, old PC, etc. See Local Provision.",
        free_tier: false,
        requires_cli: &[],
        requires_credentials: false,
        docs_url: "https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/blob/main/docs/HOSTING_SELFHOSTED.md",
    },
    ConnectionPluginView {
        id: "aws_ec2", label: "AWS EC2 (t4g.medium)",
        description: "ARM64 t4g instances. Best Pay-as-you-go ARK performance.",
        free_tier: false,
        requires_cli: &["aws"],
        requires_credentials: true,
        docs_url: "https://aws.amazon.com/cli/",
    },
    ConnectionPluginView {
        id: "azure_vm", label: "Azure VM (B2ps v2)",
        description: "ARM64 B-series, Burstable. Standard Azure.",
        free_tier: false,
        requires_cli: &["az"],
        requires_credentials: true,
        docs_url: "https://learn.microsoft.com/cli/azure/install-azure-cli",
    },
    ConnectionPluginView {
        id: "gcp_compute", label: "Google Compute e2-medium",
        description: "x86 e2-medium / e2-standard. GCE has free tier.",
        free_tier: true,
        requires_cli: &["gcloud"],
        requires_credentials: true,
        docs_url: "https://cloud.google.com/sdk/docs/install",
    },
    ConnectionPluginView {
        id: "tailscale", label: "Tailscale network",
        description: "Private 100.x.x.x connectivity for ARK servers behind CGNAT or without port forwarding.",
        free_tier: true,
        requires_cli: &["tailscale"],
        requires_credentials: true,
        docs_url: "https://tailscale.com/docs/reference/tailscale-cli/up",
    },
];

/// Lookup the view for a ConnectionPluginId.
pub fn view(id: ConnectionPluginId) -> ConnectionPluginView {
    CATALOG.iter().find(|v| v.id == id.as_str())
        .expect("missing ConnectionPluginView for known id")
        .clone()
}

/// All connection plugins, in canonical order.
pub fn all() -> Vec<ConnectionPluginView> {
    CATALOG.to_vec()
}

/// Bridge back to the original `HostProvider` enum (used when the
/// plugin is selected and we need to compose a runner). This is the
/// soft boundary: the catalog UI is keyed by `ConnectionPluginId`,
/// but actual deployment still flows through `HostTarget` →
/// `provision_script` / `render_provider_run_script`. That's
/// intentional — keeps the existing tests and frontend contract.
pub fn host_target_for(plugin: ConnectionPluginId,
                       region: String,
                       target_template: &HostTarget) -> HostTarget {
    let mut t = target_template.clone();
    t.provider = match plugin {
        ConnectionPluginId::Oracle       => HostProvider::Oracle,
        ConnectionPluginId::Hetzner      => HostProvider::Hetzner,
        ConnectionPluginId::Digitalocean => HostProvider::Digitalocean,
        ConnectionPluginId::SelfHosted   => HostProvider::SelfHosted,
        ConnectionPluginId::AwsEc2       => HostProvider::AwsEc2,
        ConnectionPluginId::AzureVm      => HostProvider::AzureVm,
        ConnectionPluginId::GcpCompute   => HostProvider::GcpCompute,
        // Tailscale is a network connector, not a VPS provider. Preserve
        // the selected hosting target while the connection manager handles
        // its 100.x.x.x endpoint separately.
        ConnectionPluginId::Tailscale    => return t,
    };
    if !region.is_empty() { t.region = region; }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_includes_tailscale_connection_plugin() {
        assert_eq!(CATALOG.len(), 8, "expected 8 connection plugins");
        assert!(CATALOG.iter().any(|entry| entry.id == "tailscale"));
    }

    #[test]
    fn all_ids_unique() {
        let mut seen = std::collections::HashSet::new();
        for v in CATALOG {
            assert!(seen.insert(v.id), "duplicate id {}", v.id);
        }
    }

    #[test]
    fn round_trip_id_to_str() {
        for v in CATALOG {
            let parsed = ConnectionPluginId::from_str(v.id)
                .unwrap_or_else(|| panic!("unknown id {}", v.id));
            assert_eq!(parsed.as_str(), v.id);
        }
    }

    #[test]
    fn free_tier_includes_tailscale_and_hosting_options() {
        let frees: Vec<_> = CATALOG.iter().filter(|v| v.free_tier).map(|v| v.id).collect();
        assert!(frees.contains(&"oracle"));
        assert!(frees.contains(&"gcp_compute"));
        assert!(frees.contains(&"tailscale"));
        assert_eq!(frees.len(), 3, "Oracle, GCP, and Tailscale are free-tier options");
    }

    #[test]
    fn selfhosted_requires_no_cli_or_credentials() {
        let v = view(ConnectionPluginId::SelfHosted);
        assert!(v.requires_cli.is_empty(), "selfhosted must not require a CLI");
        assert!(!v.requires_credentials, "selfhosted must not require credentials");
    }

    #[test]
    fn host_target_for_fills_provider_and_region() {
        let t = HostTarget::new_self();
        let filled = host_target_for(
            ConnectionPluginId::Hetzner, "fsn1".into(), &t
        );
        assert_eq!(filled.provider, HostProvider::Hetzner);
        assert_eq!(filled.region, "fsn1");
    }
}

// ─── Tauri commands (Session 6 / P2) ───────────────────────────────────────

/// Plain struct crossing the React bridge; serde flattens the static
/// `&'static str` fields so React receives ordinary strings.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionPluginJson {
    pub id:                    String,
    pub label:                 String,
    pub description:           String,
    pub free_tier:             bool,
    pub requires_cli:          Vec<String>,
    pub requires_credentials:  bool,
    pub docs_url:              String,
}

/// Lists every connection plugin (one per VPS provider) the desktop
/// app recognises. Returns the same shape the existing
/// `list_hosting_providers` returns, but with extra metadata
/// (free_tier, requires_cli, docs_url) layered on.
#[tauri::command]
pub fn list_connection_plugins() -> Vec<ConnectionPluginJson> {
    CATALOG.iter().map(|v| ConnectionPluginJson {
        id:                    v.id.to_string(),
        label:                 v.label.to_string(),
        description:           v.description.to_string(),
        free_tier:             v.free_tier,
        requires_cli:          v.requires_cli.iter().map(|s| (*s).to_string()).collect(),
        requires_credentials:  v.requires_credentials,
        docs_url:              v.docs_url.to_string(),
    }).collect()
}

/// Returns the metadata for a single plugin id.
#[tauri::command]
pub fn get_connection_plugin(id: String) -> Result<ConnectionPluginJson, String> {
    let parsed = ConnectionPluginId::from_str(&id)
        .ok_or_else(|| format!("unknown connection plugin id: {id}"))?;
    let v = view(parsed);
    Ok(ConnectionPluginJson {
        id:                    v.id.to_string(),
        label:                 v.label.to_string(),
        description:           v.description.to_string(),
        free_tier:             v.free_tier,
        requires_cli:          v.requires_cli.iter().map(|s| (*s).to_string()).collect(),
        requires_credentials:  v.requires_credentials,
        docs_url:              v.docs_url.to_string(),
    })
}
