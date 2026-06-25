export function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-3">
      <p className="text-ark-cyan/60 text-[10px] font-bold tracking-widest uppercase">{title}</p>
      {children}
    </div>
  )
}

export function Toggle({ value, onChange }: { value: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!value)}
      className="relative w-10 h-5 rounded-full transition-colors flex-shrink-0"
      style={{ background: value ? 'rgba(0,200,255,0.7)' : 'rgba(255,255,255,0.1)' }}
    >
      <span
        className="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform"
        style={{ left: value ? '1.3rem' : '0.125rem' }}
      />
    </button>
  )
}

export function Field({
  label, value, onChange, placeholder, type = 'text'
}: {
  label: string
  value: string
  onChange: (v: string) => void
  placeholder?: string
  type?: string
}) {
  return (
    <div className="flex items-center gap-3">
      <label className="text-ark-cyan/50 text-xs w-32 flex-shrink-0 text-right">{label}</label>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="flex-1 bg-transparent border border-ark-cyan/25 text-ark-cyan/90 text-xs px-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/60 placeholder-ark-cyan/20 font-mono"
      />
    </div>
  )
}
