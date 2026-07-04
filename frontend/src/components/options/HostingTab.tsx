import { useState, useEffect } from 'react'
import { useI18n } from '../../i18n/useI18n'
import { Section, Field, Select, TextArea } from '../ui/OptionsUI'
import { invoke } from '../../services/tauri'

interface ProviderView {
  key: string
  label: string
}

interface HostingTarget {
  provider: string
  region: string
  ssh_user: string
  ssh_host: string
  ssh_port: number
  ssh_key_path: string
  disk_gb: number
}

const DEFAULT_TARGET: HostingTarget = {
  provider: 'selfhosted',
  region: 'on-prem',
  ssh_user: 'arkasa',
  ssh_host: '',
  ssh_port: 22,
  ssh_key_path: '~/.ssh/id_ed25519',
  disk_gb: 50,
}

// Picker for the "Run on your own hardware" sub-section. We don't
// enumerate platform-specific steps in code; each value maps 1:1 to
// the `LocalTargetClass` enum in the Rust Tauri command.
const LOCAL_CLASS_OPTIONS: { value: string; label: string }[] = [
  { value: 'debian-pi5',   label: 'Raspberry Pi 5 — Debian Bookworm arm64' },
  { value: 'debian-x86',   label: 'Debian 12 / 13 minimal — Intel NUC, x86 server' },
  { value: 'ubuntu-x86',   label: 'Ubuntu Server 24.04 — Intel NUC, x86 server' },
  { value: 'wsl2-debian',  label: 'Windows 10/11 + WSL2 Debian' },
  { value: 'wsl2-ubuntu',  label: 'Windows 10/11 + WSL2 Ubuntu' },
  { value: 'macos-arm',    label: 'Apple Silicon Mac — macOS 14+' },
  { value: 'macos-intel',  label: 'Intel-based Mac — macOS 14+' },
]

interface ProvisionStageView {
  stage: string
  command_hint: string
  expecting: string
}

interface LocalPlanView {
  class_label: string
  bundled_script: string
  inline_command: string
  supports_systemd: boolean
  uses_apt: boolean
  stages: ProvisionStageView[]
  notes: string[]
}

export default function HostingTab() {
  const { tk } = useI18n()
  const [providers, setProviders] = useState<ProviderView[]>([])
  const [target, setTarget] = useState<HostingTarget>(DEFAULT_TARGET)
  const [bundleUrl, setBundleUrl] = useState('https://your-bucket.s3.amazonaws.com/arkasa-bundle.zip')
  const [script, setScript] = useState('')
  const [runCmd, setRunCmd] = useState('')
  const [generating, setGenerating] = useState(false)

  // Local-provision state (Pi / NUC / WSL2 / macOS)
  const [localClass, setLocalClass] = useState('debian-x86')
  const [localUser, setLocalUser] = useState('arkasa')
  const [localHost, setLocalHost] = useState('127.0.0.1')
  const [localDiskGb, setLocalDiskGb] = useState(64)
  const [localPlan, setLocalPlan] = useState<LocalPlanView | null>(null)
  const [localBusy, setLocalBusy] = useState(false)
  const [localErr, setLocalErr] = useState('')

  useEffect(() => {
    invoke<ProviderView[]>('list_hosting_providers')
      .then(setProviders)
      .catch(() => setProviders([]))
  }, [])

  const generate = async () => {
    setGenerating(true)
    setScript('')
    setRunCmd('')
    try {
      const sc = await invoke<string>('render_hosting_script', {
        target: {
          provider: target.provider,
          region: target.region,
          ssh_user: target.ssh_user,
          ssh_host: target.ssh_host,
          ssh_port: target.ssh_port,
          ssh_key_path: target.ssh_key_path,
          disk_gb: target.disk_gb,
          env: {},
        },
        bundleUrl,
      })
      setScript(sc)
      const rc = await invoke<string>('render_provider_run_script', {
        target: {
          provider: target.provider,
          region: target.region,
          ssh_user: target.ssh_user,
          ssh_host: target.ssh_host,
          ssh_port: target.ssh_port,
          ssh_key_path: target.ssh_key_path,
          disk_gb: target.disk_gb,
          env: {},
        },
        bundleUrl,
      })
      setRunCmd(rc)
    } catch (e: unknown) {
      setScript(`ERROR: ${String(e)}`)
    } finally {
      setGenerating(false)
    }
  }

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text)
    } catch { /* ignore */ }
  }

  const generateLocal = async () => {
    setLocalBusy(true)
    setLocalErr('')
    setLocalPlan(null)
    try {
      const plan = await invoke<LocalPlanView>('render_local_provision_plan', {
        class: localClass,
        sshUser: localUser,
        sshHost: localHost,
        bundleUrl,
        diskGb: localDiskGb,
      })
      setLocalPlan(plan)
    } catch (e: unknown) {
      setLocalErr(String(e))
    } finally {
      setLocalBusy(false)
    }
  }

  const providerOptions = providers.map((p) => ({ value: p.key, label: p.label }))

  return (
    <>
      <Section title={tk('section_hosting_provider', 'Cloud provider')}>
        <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
          {tk('hosting_intro',
            'Generate a cloud-init / startup script for the chosen VPS provider, then run it locally with `hcloud up`, `doctl`, `openTofu`, etc. The desktop app never asks for your cloud credentials.')}
        </p>
        <Select
          label={tk('provider_label', 'Provider')}
          value={target.provider}
          onChange={(v) => setTarget({ ...target, provider: v })}
          options={providerOptions}
          placeholder="—"
        />
        <Field
          label={tk('region_label', 'Region')}
          value={target.region}
          onChange={(v) => setTarget({ ...target, region: v })}
          placeholder="fsn1 / nyc3 / eu-central-1 / …"
        />
        <Field
          label={tk('ssh_user_label', 'SSH user')}
          value={target.ssh_user}
          onChange={(v) => setTarget({ ...target, ssh_user: v })}
        />
        <Field
          label={tk('ssh_host_label', 'SSH host (optional)')}
          value={target.ssh_host}
          onChange={(v) => setTarget({ ...target, ssh_host: v })}
          placeholder="empty for now; you fill in once the VPS is up"
        />
        <Field
          label={tk('ssh_port_label', 'SSH port')}
          value={String(target.ssh_port)}
          onChange={(v) => setTarget({ ...target, ssh_port: Number(v) || 22 })}
          type="number"
        />
        <Field
          label={tk('ssh_key_label', 'SSH key path')}
          value={target.ssh_key_path}
          onChange={(v) => setTarget({ ...target, ssh_key_path: v })}
        />
        <Field
          label={tk('disk_label', 'Disk GB')}
          value={String(target.disk_gb)}
          onChange={(v) => setTarget({ ...target, disk_gb: Number(v) || 50 })}
          type="number"
        />
        <Field
          label={tk('bundle_url_label', 'Backup bundle URL')}
          value={bundleUrl}
          onChange={setBundleUrl}
          placeholder="https://…/arkasa-bundle.zip  (or any HTTPS file)"
        />
        <button
          onClick={generate}
          disabled={generating || !bundleUrl}
          className="ark-action-btn px-4 py-2 text-xs tracking-widest disabled:opacity-40"
        >
          {generating
            ? tk('generating', 'Generating…')
            : tk('btn_generate_script', 'GENERATE SCRIPT')}
        </button>
      </Section>

      {script && (
        <Section title={tk('section_user_data', 'cloud-init / user-data')}>
          <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
            {tk('user_data_intro',
              'Paste this into your VPS provider\'s "user data" / "cloud-init" field. Install SteamCMD, download ARK ASA, run as systemd service.')}
          </p>
          <TextArea
            label={tk('script_label', 'Generated script')}
            value={script}
            onChange={() => { /* read-only */ }}
            rows={10}
          />
          <button
            onClick={() => copyToClipboard(script)}
            className="text-ark-cyan/70 hover:text-ark-cyan text-[10px] tracking-widest uppercase"
          >
            ⧉ {tk('btn_copy', 'Copy')}
          </button>
        </Section>
      )}

      {runCmd && (
        <Section title={tk('section_run_script', 'One-click deploy')}>
          <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
            {tk('run_script_intro',
              'Run this command in a Linux shell that has `hcloud`, `doctl`, `aws`, `az`, `gcloud` or `oci` CLI configured. The script creates the VPS, attaches cloud-init, and prints the public IP.')}
          </p>
          <TextArea
            label={tk('run_script_label', 'Run command (bash one-liner)')}
            value={runCmd}
            onChange={() => { /* read-only */ }}
            rows={4}
          />
          <button
            onClick={() => copyToClipboard(runCmd)}
            className="text-ark-cyan/70 hover:text-ark-cyan text-[10px] tracking-widest uppercase"
          >
            ⧉ {tk('btn_copy', 'Copy')}
          </button>
        </Section>
      )}

      {/* ── Run on your own hardware (Pi / NUC / WSL2 / macOS) ────────────── */}
      <Section title={tk('section_local_provision', 'Run on your own hardware')}>
        <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
          {tk('local_provision_intro',
            'Pick your hardware class, paste the same backup bundle URL you would for a cloud VPS, and click GENERATE PLAN. The app produces a platform-tailored bash, an inline one-liner, and a stage-by-stage checklist.')}
        </p>
        <Select
          label={tk('local_class_label', 'Hardware / OS')}
          value={localClass}
          onChange={setLocalClass}
          options={LOCAL_CLASS_OPTIONS}
          placeholder="—"
        />
        <Field
          label={tk('local_ssh_user_label', 'Local user (for plan tracking)')}
          value={localUser}
          onChange={setLocalUser}
          placeholder="arkasa / ubuntu / $USER"
        />
        <Field
          label={tk('local_ssh_host_label', 'Local host (informational)')}
          value={localHost}
          onChange={setLocalHost}
          placeholder="127.0.0.1 / pi5.lan / mac.local"
        />
        <Field
          label={tk('disk_label', 'Disk GB')}
          value={String(localDiskGb)}
          onChange={(v) => setLocalDiskGb(Number(v) || 64)}
          type="number"
        />
        <button
          onClick={generateLocal}
          disabled={localBusy || !bundleUrl}
          className="ark-action-btn px-4 py-2 text-xs tracking-widest disabled:opacity-40"
          style={{ borderColor: 'rgba(0, 200, 255, 0.4)' }}
        >
          {localBusy ? tk('generating', 'Generating…') : tk('btn_generate_local_plan', 'GENERATE LOCAL PLAN')}
        </button>
        {localErr && <p className="text-red-400 text-xs">⚠ {localErr}</p>}
      </Section>

      {localPlan && (
        <Section title={localPlan.class_label}>
          <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
            {tk('local_plan_intro',
              'Save this bash as run.sh on the target hardware, then run it as the user specified below. The checklist under the script shows what success looks like at each stage.')}
          </p>
          <ul className="list-disc pl-5 text-ark-cyan/55 text-[11px] space-y-1 mt-1 mb-3">
            {localPlan.notes.map((n, i) => <li key={i}>{n}</li>)}
          </ul>
          <TextArea
            label={tk('local_inline_label', 'Inline one-liner (operator-friendly)')}
            value={localPlan.inline_command}
            onChange={() => { /* read-only */ }}
            rows={3}
          />
          <button
            onClick={() => copyToClipboard(localPlan.inline_command)}
            className="text-ark-cyan/70 hover:text-ark-cyan text-[10px] tracking-widest uppercase mb-3"
          >
            ⧉ {tk('btn_copy', 'Copy')}
          </button>
          <TextArea
            label={tk('local_script_label', 'Bundled bash script (prefer this; the inline command is a shortcut)')}
            value={localPlan.bundled_script}
            onChange={() => { /* read-only */ }}
            rows={12}
          />
          <button
            onClick={() => copyToClipboard(localPlan.bundled_script)}
            className="text-ark-cyan/70 hover:text-ark-cyan text-[10px] tracking-widest uppercase mb-3"
          >
            ⧉ {tk('btn_copy', 'Copy')}
          </button>
          <details className="mt-2">
            <summary className="cursor-pointer text-ark-cyan/70 text-[11px] uppercase tracking-widest">
              📋 Stage-by-stage checklist ({localPlan.stages.length})
            </summary>
            <ol className="list-decimal pl-6 text-[11px] text-ark-cyan/60 mt-3 space-y-2">
              {localPlan.stages.map((s, i) => (
                <li key={i}>
                  <span className="font-mono text-ark-accent">{s.stage}</span>
                  <pre className="mt-1 text-[10px] bg-black/40 border border-ark-cyan/5 rounded p-2 font-mono whitespace-pre-wrap leading-tight text-ark-cyan/55">
{s.command_hint}
                  </pre>
                  <p className="text-ark-cyan/45 text-[10px] mt-1 italic">
                    Expecting: {s.expecting}
                  </p>
                </li>
              ))}
            </ol>
          </details>
        </Section>
      )}
    </>
  )
}
