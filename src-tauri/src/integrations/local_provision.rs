//! Local-provision generator — runs the same `provision_script` we use
//! for cloud-init, but wrapped for direct execution on the operator's
//! own hardware (Raspberry Pi 5, Intel NUC, old Win10 PC via WSL2, etc.).
//!
//! Why dedicated module? Because the cloud-init env we ship (apt-get,
//! useradd, steamcmd tarball, systemctl unit) is **Linux-only**. WSL2
//! and macOS need different bone-setup. We don't try to be clever —
//! the operator picks the platform, we give them the verified
//! one-liner, we show the expected output per stage.
//!
//! Backwards-rule: zero new first-party OS automation. If we ever grow
//! into macOS/Linux installers, they'd land here, but for v2.1 we
//! only bundle idempotent shells.

use crate::integrations::hosting::{provision_script, HostTarget, HostProvider};
use serde::{Deserialize, Serialize};

/// Concrete hardware/OS class the operator is running on. Each variant
/// yields a different `LocalProvisionPlan` because the install bone
/// setup differs (apt vs brew; systemd unit vs launchctl; etc.).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LocalTargetClass {
    /// Debian / Raspberry Pi OS Bookworm 64-bit on a Pi 5 (4 GB RAM min).
    DebianPi5,
    /// Debian 12 / 13 minimal install on Intel NUC or x86 server.
    DebianX86,
    /// Ubuntu Server 24.04 on Intel NUC or x86 server.
    UbuntuX86,
    /// Windows 10/11 with WSL2 running Debian.
    Wsl2Debian,
    /// Windows 10/11 with WSL2 running Ubuntu.
    Wsl2Ubuntu,
    /// Apple Silicon, macOS 14+.
    MacosArm,
    /// Intel-based Mac, macOS 14+.
    MacosIntel,
}

impl LocalTargetClass {
    pub fn all() -> &'static [LocalTargetClass] {
        &[
            Self::DebianPi5,
            Self::DebianX86,
            Self::UbuntuX86,
            Self::Wsl2Debian,
            Self::Wsl2Ubuntu,
            Self::MacosArm,
            Self::MacosIntel,
        ]
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::DebianPi5   => "Raspberry Pi 5 (Debian Bookworm arm64)",
            Self::DebianX86   => "Debian 12/13 minimal (Intel NUC / x86)",
            Self::UbuntuX86   => "Ubuntu Server 24.04 (Intel NUC / x86)",
            Self::Wsl2Debian  => "Windows 10/11 + WSL2 Debian",
            Self::Wsl2Ubuntu  => "Windows 10/11 + WSL2 Ubuntu",
            Self::MacosArm    => "Apple Silicon macOS 14+ (M1/M2/M3/M4)",
            Self::MacosIntel  => "Intel Mac macOS 14+",
        }
    }
    /// Display ordering used by the HostingTab UI.
    pub fn ord(self) -> u32 {
        match self {
            Self::DebianPi5 => 1,
            Self::DebianX86 => 2,
            Self::UbuntuX86 => 3,
            Self::Wsl2Debian => 4,
            Self::Wsl2Ubuntu => 5,
            Self::MacosArm   => 6,
            Self::MacosIntel => 7,
        }
    }
    /// Returns true if this class supports `systemctl` (== Linux with
    /// systemd). WSL2 does. macOS does not.
    pub fn supports_systemd(self) -> bool {
        matches!(
            self,
            Self::DebianPi5
                | Self::DebianX86
                | Self::UbuntuX86
                | Self::Wsl2Debian
                | Self::Wsl2Ubuntu
        )
    }
    /// Returns true if apt is the package manager (== Debian-family).
    pub fn uses_apt(self) -> bool {
        matches!(
            self,
            Self::DebianPi5 | Self::DebianX86 | Self::UbuntuX86 | Self::Wsl2Debian | Self::Wsl2Ubuntu
        )
    }
    /// Inline comment used in the generated plan to mark the platform.
    pub fn slug(self) -> &'static str {
        match self {
            Self::DebianPi5   => "pi5-arm64",
            Self::DebianX86   => "debian-x86_64",
            Self::UbuntuX86   => "ubuntu-x86_64",
            Self::Wsl2Debian  => "wsl2-debian",
            Self::Wsl2Ubuntu  => "wsl2-ubuntu",
            Self::MacosArm    => "macos-arm64",
            Self::MacosIntel  => "macos-x86_64",
        }
    }
}

/// A single stage in the local provision — used to render the operator's
/// "what you should see at each step" UI panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionStage {
    pub stage:        String,    // "apt setup", "steamcmd install", "systemd", "ready"
    pub command_hint: String,    // the literal command line the operator typed
    pub expecting:    String,    // what stdout should contain when it worked
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalProvisionPlan {
    pub target:           HostTarget,        // includes provider = SelfHosted
    pub class:            LocalTargetClass,
    pub class_label:      String,
    pub bundled_script:   String,            // complete script, ready to run as `bash run.sh`
    pub inline_command:   String,            // one-liner the operator runs in their shell
    pub supports_systemd: bool,
    pub uses_apt:         bool,
    pub stages:           Vec<ProvisionStage>,
    pub notes:            Vec<String>,
}

/// Build a complete `LocalProvisionPlan` for a hardware class + cloud-init
/// bundle URL. The bundled_script is runnable as-is. The inline_command is
/// the `curl | sudo bash` style one-liner (operator's choice).
pub fn build_local_plan(
    class:  LocalTargetClass,
    ssh_user: &str,
    ssh_host: &str,
    server_zip_url: &str,
    disk_gb: u32,
) -> LocalProvisionPlan {
    // Pack everything ARK ASA needs into a generic self-hosted cloud-init.
    let target = HostTarget {
        provider: HostProvider::SelfHosted,
        region:   class.slug().to_string(),
        ssh_user: ssh_user.to_string(),
        ssh_host: ssh_host.to_string(),
        ssh_port: 22,
        ssh_key_path: String::new(),       // ssh-key not used (running locally)
        env:      Default::default(),
        disk_gb,
    };

    let mut bundled_script = provision_script(&target, server_zip_url);

    // ── Per-platform patches ────────────────────────────────────────────
    if !class.uses_apt() {
        // macOS path: brew lands where we expect (or fail loudly).
        bundled_script = macos_friendly_patch(&bundled_script);
    }
    if !class.supports_systemd() {
        // Strip systemctl calls; offer launchctl instead (macOS),
        // and a screen + logfile fallback for the operator.
        bundled_script = strip_systemd_blocks(&bundled_script);
    }
    if matches!(class, LocalTargetClass::Wsl2Debian | LocalTargetClass::Wsl2Ubuntu) {
        // WSL2 quirk: /proc/cpuinfo ident doesn't expose systemd flags
        // the same way. Append a guard.
        bundled_script.push_str(WSL2_TAIL);
    }

    // One-liner form: pipe into bash. Operators like this for "fresh box".
    let script_url = format!(
        "https://EXAMPLE/arkasa-bundle-{}.sh", class.slug()
    );
    let inline_command = if class.supports_systemd() {
        format!(
            "# On the operator's local box (Pi 5 / NUC / WSL2):\n\
             curl -fsSL {script_url} | sudo bash -s --",
        )
    } else {
        format!(
            "# On macOS — sudo is rare; brew requires userland perm:\n\
             curl -fsSL {script_url} | bash -s --",
        )
    };

    // Stages are rendered as operator-facing copy.
    let stages = render_stages(class);

    // Per-platform notes (printed in the UI panel).
    let mut notes = vec![format!(
        "Hardware: {}",
        class.label()
    )];
    if class.supports_systemd() {
        notes.push("ARK service runs under `systemd` as user `arkasa`.".into());
    } else if matches!(class, LocalTargetClass::MacosArm | LocalTargetClass::MacosIntel) {
        notes.push("ARK service runs in a `screen` session; logs at `/var/log/arkasa.log`.".into());
        notes.push("`brew install --cask screen` if `screen` is missing.".into());
    } else {
        notes.push("Manual launch required (no service manager on this class).".into());
    }
    if matches!(class, LocalTargetClass::DebianPi5) {
        notes.push("Pi 5 needs an active cooling fan — heat-throttles under load.".into());
        notes.push("Use Raspberry Pi OS Bookworm 64-bit (Lite is fine).".into());
    }
    if matches!(class, LocalTargetClass::Wsl2Debian | LocalTargetClass::Wsl2Ubuntu) {
        notes.push("WSL2: enable systemd in `wsl.conf` first (see docs/HOSTING_SELFHOSTED.md).".into());
        notes.push("On Windows 10 (build 19041+), run as Administrator once.".into());
    }

    LocalProvisionPlan {
        target:           target.clone(),
        class,
        class_label:      class.label().to_string(),
        bundled_script,
        inline_command,
        supports_systemd: class.supports_systemd(),
        uses_apt:         class.uses_apt(),
        stages,
        notes,
    }
}

fn render_stages(class: LocalTargetClass) -> Vec<ProvisionStage> {
    if class.uses_apt() {
        vec![
            ProvisionStage {
                stage: "apt setup".into(),
                command_hint: "DEBIAN_FRONTEND=noninteractive apt-get install -y lib32gcc-s1 lib32stdc++6 libc6-i386 libcurl4-gnutls-dev:i386 libsdl2-2.0-0:i386 wget curl ca-certificates libsdl2-2.0-0 screen unzip".into(),
                expecting: "Reading package lists... Done. (...) Processing triggers for libc-bin (...)".into(),
            },
            ProvisionStage {
                stage: "user".into(),
                command_hint: "id arkasa || useradd -m -d /home/arkasa -s /bin/bash arkasa".into(),
                expecting: "(no output on success; silent)".into(),
            },
            ProvisionStage {
                stage: "steamcmd".into(),
                command_hint: "./steamcmd.sh +login anonymous +force_install_dir /home/arkasa/server +app_update 2430930 validate +quit".into(),
                expecting: "Update state (0x...) ... OK, Steam Console initialized. Success! App '2430930' fully installed.".into(),
            },
            ProvisionStage {
                stage: "bundle".into(),
                command_hint: "curl -L https://...bundle.zip -o /tmp/arkasa-bundle.zip && sudo -u arkasa unzip -o /tmp/arkasa-bundle.zip -d /home/arkasa/server".into(),
                expecting: "inflating: /home/arkasa/server/ShooterGame/... (concluded)".into(),
            },
            ProvisionStage {
                stage: "systemd".into(),
                command_hint: "systemctl daemon-reload && systemctl enable --now arkasa.service".into(),
                expecting: "Created symlink /etc/systemd/system/multi-user.target.wants/arkasa.service ... (service starts; then check) → systemctl is-active arkasa ⇒ active".into(),
            },
            ProvisionStage {
                stage: "ready".into(),
                command_hint: "journalctl -u arkasa -f".into(),
                expecting: "ARK server listening on UDP 7777 (visible from Steam server browser after <=2 min)".into(),
            },
        ]
    } else {
        // macOS stages
        vec![
            ProvisionStage {
                stage: "brew setup".into(),
                command_hint: "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"".into(),
                expecting: "Installation successful!".into(),
            },
            ProvisionStage {
                stage: "brew packages".into(),
                command_hint: "brew install wget curl screen unzip".into(),
                expecting: "(all formulas installed with `✓` checkmarks)".into(),
            },
            ProvisionStage {
                stage: "steamcmd".into(),
                command_hint: "mkdir -p ~/steamcmd && cd ~/steamcmd && wget https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz && tar xzf steamcmd_linux.tar.gz".into(),
                expecting: "steamcmd.sh present in ~/steamcmd".into(),
            },
            ProvisionStage {
                stage: "server install".into(),
                command_hint: "cd ~/steamcmd && ./steamcmd.sh +login anonymous +force_install_dir ~/server +app_update 2430930 validate +quit".into(),
                expecting: "Success! App '2430930' fully installed.".into(),
            },
            ProvisionStage {
                stage: "ready".into(),
                command_hint: "screen -dmS arkasa bash -c 'cd ~/server && ./ShooterGame/Binaries/Linux/ShooterGameServer TheIsland_WP > /var/log/arkasa.log 2>&1'".into(),
                expecting: "(no output; check `screen -ls` for the 'arkasa' session, and `tail -n 50 /var/log/arkasa.log`)".into(),
            },
        ]
    }
}

fn macos_friendly_patch(script: &str) -> String {
    // Strip the apt-get block — it doesn't exist on macOS. We prepend a brew install.
    let mut out = String::new();
    out.push_str(&format!(
        "#!/bin/bash\n# macOS-friendly ARK ASA bootstrap\n\
         # Auto-derived from the linux provisioning script on {}\n\
         set -e\n\n",
        chrono::Utc::now().date_naive()
    ));
    out.push_str("# 1. Homebrew packages\n\
                  if ! command -v brew >/dev/null 2>&1; then\n\
                  /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"\n\
                  fi\n\
                  brew install --quiet wget curl screen unzip\n\n");
    out.push_str("# 2. ARK user (macOS): we use the operator user directly\n\
                  SERVER_HOME=\"$HOME\"\n");
    out.push_str("# 3. The remainder of the script body is unchanged:\n");
    for line in script.lines() {
        let tr = line.trim_start();
        // Drop apt-specific instructions entirely
        if tr.starts_with("apt-get") || tr.starts_with("DEBIAN_FRONTEND")
            || tr.contains("apt-get install")
            || tr.contains("lib32gcc") || tr.contains("i386")
            // Drop systemd unit block & systemctl commands
            || tr.starts_with("# 5.")
            || tr.starts_with("cat >/etc/systemd")
            || tr.starts_with("systemctl ")
            || tr.starts_with("echo \"✓ ARK ASA is now running under systemd")
        {
            continue;
        }
        // Drop the systemd unit heredoc body (the lines inside `<<'EOF'`...)
        // — they reference /home/arkasa paths and have NO use on macOS.
        if tr.starts_with("[Unit]") || tr.starts_with("[Service]") || tr.starts_with("[Install]")
            || tr.starts_with("Description=") || tr.starts_with("After=") || tr.starts_with("Wants=")
            || tr.starts_with("Type=") || tr.starts_with("WorkingDirectory=")
            || tr.starts_with("EnvironmentFile=") || tr.starts_with("ExecStart=")
            || tr.starts_with("Restart=") || tr.starts_with("RestartSec=") || tr.starts_with("LimitNOFILE=")
            || tr.starts_with("User=") || tr.starts_with("WantedBy=")
            || tr == "EOF"
        {
            continue;
        }
        // Replace /home/arkasa with $SERVER_HOME
        // Drop `sudo -u arkasa` because the user is already running.
        let mut patched = line.to_string();
        if patched.contains("sudo -u arkasa") {
            patched = patched.replace("sudo -u arkasa", "");
        }
        if patched.contains("/home/arkasa") {
            patched = patched.replace("/home/arkasa", "$SERVER_HOME");
        }
        out.push_str(&patched);
        out.push('\n');
    }
    out.push_str("\n# 4. Persist with screen instead of systemd\n\
                  screen -dmS arkasa bash -c \"cd \\\"$SERVER_HOME/server\\\" && \\\n\
                  ./ShooterGame/Binaries/Linux/ShooterGameServer TheIsland_WP \\\n\
                  > /var/log/arkasa.log 2>&1\"\n\
                  tail -n 30 /var/log/arkasa.log\n");
    out
}

fn strip_systemd_blocks(script: &str) -> String {
    const KEEP: &[&str] = &[
        "systemctl ",
        "/etc/systemd/",
        "systemd_unit",
        "^systemctl$",
    ];
    script
        .lines()
        .filter(|line| {
            for k in KEEP {
                if line.contains(k) {
                    return false;
                }
            }
            true
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

const WSL2_TAIL: &str = r#"
# ── WSL2 hardening ─────────────────────────────────────────────────────────
# systemd must be enabled for the arkasa.service unit to come alive:
#   1. exit and run:  wsl --shutdown
#   2. edit /etc/wsl.conf (in WSL2's root) and add:
#        [boot]
#        systemd=true
#   3. wsl --start; verify `systemctl is-active arkasa` → active
#
# If systemd is unbootable on the Windows build, fall back to:
#   sudo -u arkasa bash -c 'cd /home/arkasa/server && \
#       ./ShooterGame/Binaries/Linux/ShooterGameServer TheIsland_WP'
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_target(class: LocalTargetClass) -> HostTarget {
        let mut t = HostTarget::new_self();
        t.region = class.slug().into();
        t.disk_gb = 64;
        t
    }

    #[test]
    fn pi5_plan_uses_systemd_and_apt() {
        let p = build_local_plan(
            LocalTargetClass::DebianPi5,
            "arkasa",
            "127.0.0.1",
            "https://example.com/ark-bundle.zip",
            64,
        );
        assert!(p.uses_apt);
        assert!(p.supports_systemd);
        assert!(p.bundled_script.contains("apt-get"));
        assert!(p.bundled_script.contains("systemctl"));
        assert_eq!(p.stages.len(), 6, "Pi5 stages: apt, user, steamcmd, bundle, systemd, ready");
        assert!(p.notes.iter().any(|n| n.contains("cooling fan")));
        // Per-class slug for the script URL
        assert!(p.inline_command.contains("pi5-arm64"));
    }

    #[test]
    fn ubuntu_x86_plan_no_cooling_note() {
        let p = build_local_plan(
            LocalTargetClass::UbuntuX86,
            "arkasa",
            "127.0.0.1",
            "https://example.com/ark-bundle.zip",
            100,
        );
        assert!(p.uses_apt);
        assert!(p.supports_systemd);
        assert!(!p.notes.iter().any(|n| n.contains("cooling fan")));
        // ubuntu region slug for tracking
        assert!(p.inline_command.contains("ubuntu-x86_64"));
    }

    #[test]
    fn wsl2_plan_includes_tail_warning() {
        let p = build_local_plan(
            LocalTargetClass::Wsl2Debian,
            "arkasa",
            "127.0.0.1",
            "https://example.com/b.zip",
            64,
        );
        assert!(p.bundled_script.contains("WSL2 hardening"));
        assert!(p.notes.iter().any(|n| n.contains("wsl.conf")));
    }

    #[test]
    fn macos_drops_systemd_and_apt_switches_to_brew() {
        let p = build_local_plan(
            LocalTargetClass::MacosArm,
            "$USER",
            "127.0.0.1",
            "https://example.com/b.zip",
            64,
        );
        assert!(!p.uses_apt);
        assert!(!p.supports_systemd);
        // Brew block injected
        assert!(p.bundled_script.contains("brew install"));
        // /home/arkasa replaced with $SERVER_HOME
        assert!(!p.bundled_script.contains("/home/arkasa"));
        // systemD calls are gone
        assert!(!p.bundled_script.contains("systemctl"));
        // No i386 Linux libs in the macOS script
        assert!(!p.bundled_script.contains("i386"));
        // Final-stage fallback uses screen
        assert!(p.bundled_script.contains("screen -dmS"));
        // Stages should be a different set
        assert!(p.stages.iter().any(|s| s.stage == "brew packages"));
    }

    #[test]
    fn plan_render_doesnt_panic_for_any_class() {
        for &class in LocalTargetClass::all() {
            let p = build_local_plan(
                class,
                "arkasa",
                "127.0.0.1",
                "https://e.com/x.zip",
                64,
            );
            // Script must be non-empty and have a shebang
            assert!(!p.bundled_script.is_empty(), "{class:?} produced empty");
            assert!(p.bundled_script.contains("#!/bin/bash"), "{class:?} missing shebang");
            // Stages must have at least one entry
            assert!(!p.stages.is_empty(), "{class:?} empty stages");
            // Notes must mention the platform label
            assert!(
                p.notes.iter().any(|n| n.contains(class.label())),
                "{class:?} notes missing label"
            );
        }
    }
}
