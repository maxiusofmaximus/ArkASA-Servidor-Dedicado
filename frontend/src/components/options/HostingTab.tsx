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

export default function HostingTab() {
  const { tk } = useI18n()
  const [providers, setProviders] = useState<ProviderView[]>([])
  const [target, setTarget] = useState<HostingTarget>(DEFAULT_TARGET)
  const [bundleUrl, setBundleUrl] = useState('https://your-bucket.s3.amazonaws.com/arkasa-bundle.zip')
  const [script, setScript] = useState('')
  const [runCmd, setRunCmd] = useState('')
  const [generating, setGenerating] = useState(false)

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
    </>
  )
}
