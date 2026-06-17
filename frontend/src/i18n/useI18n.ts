import { useBackupStore } from '../stores/backupStore'
import { translateLabel, getTranslation, labelToKey } from './translations'
import tooltips from './tooltips'

export interface I18n {
  /** Translate a human-readable label (auto-normalizes to key). */
  t: (label: string) => string
  /** Look up a translation by explicit key. Falls back to `fallback`. */
  tk: (key: string, fallback?: string) => string
  /** Get the tooltip description for a label. Returns undefined if none. */
  tip: (label: string) => string | undefined
  lang: string
}

export function useI18n(): I18n {
  const lang = useBackupStore((s) => s.language)

  const t = (label: string): string => translateLabel(lang, label)
  const tk = (key: string, fallback = ''): string => getTranslation(lang, key, fallback)
  const tip = (label: string): string | undefined => {
    const key = labelToKey(label)
    return tooltips[key]
  }

  return { t, tk, tip, lang }
}
