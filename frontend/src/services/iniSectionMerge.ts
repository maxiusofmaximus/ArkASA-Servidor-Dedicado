import { invoke } from '../services/tauri'

export interface IniSectionBlock {
  name: string
  /** Lines that belong to this section, including the [Section] header and comments. */
  lines: string[]
}

/**
 * Parse an INI text into ordered section blocks. Lines before the first
 * section header go into an implicit block with `name === ''`.
 * Comments (`;` or `#`) attach to whatever section is currently being parsed.
 */
export function parseIniBlocks(text: string): IniSectionBlock[] {
  const blocks: IniSectionBlock[] = []
  let current: IniSectionBlock | null = null

  for (const raw of text.split('\n')) {
    const line = raw.trim()
    const sectionMatch = line.match(/^\[(.+)\]$/)
    if (sectionMatch) {
      current = { name: sectionMatch[1], lines: [raw] }
      blocks.push(current)
      continue
    }
    if (!current) {
      current = { name: '', lines: [] }
      blocks.push(current)
    }
    current.lines.push(raw)
  }
  return blocks
}

/**
 * Serialize section blocks back to INI text.
 */
export function serializeIniBlocks(blocks: IniSectionBlock[]): string {
  return blocks.map((b) => b.lines.join('\n')).join('\n')
}

/**
 * Merge sections from `modIniBlocks` into `serverIniBlocks`: for each
 * named section present in `modIniBlocks`, drop any existing section of
 * the same name from `serverIniBlocks` (keeps the first occurrence of
 * unnamed pre-amble), then append the mod's section at the end.
 *
 * Returns the new combined blocks (does NOT mutate inputs).
 */
export function mergeSections(serverIniBlocks: IniSectionBlock[], modIniBlocks: IniSectionBlock[]): IniSectionBlock[] {
  const modSectionNames = new Set(modIniBlocks.map((b) => b.name).filter((n) => n !== ''))
  if (modSectionNames.size === 0) return serverIniBlocks

  const kept = serverIniBlocks.filter((b) => b.name === '' || !modSectionNames.has(b.name))
  const namedModBlocks = modIniBlocks.filter((b) => b.name !== '')
  return [...kept, ...namedModBlocks]
}

/**
 * High-level helper used by ModsTab's "save" action:
 *
 *   1. Parse the per-mod INI text the operator authored.
 *   2. Read the server's GameUserSettings.ini (or Game.ini) from disk.
 *   3. For every section present in the mod INI, remove that section from
 *      the server INI if it already exists (so we don't end up with two
 *      [UpgradeStation] blocks), and append the mod's section at the end.
 *   4. Write the merged result back to the server INI.
 *
 * Returns the list of section names that were synced (for UI feedback).
 */
export async function syncModIniToServerFile(
  modIniText: string,
  serverIniPath: string,
): Promise<string[]> {
  const modBlocks = parseIniBlocks(modIniText)
  const sectionNames = modBlocks.map((b) => b.name).filter((n) => n !== '')
  if (sectionNames.length === 0) return []

  let serverText = ''
  try {
    serverText = await invoke<string>('read_text_file', { path: serverIniPath })
  } catch {
    serverText = ''
  }

  const serverBlocks = parseIniBlocks(serverText)
  const merged = mergeSections(serverBlocks, modBlocks)
  const newServerText = serializeIniBlocks(merged)

  await invoke('write_text_file', { path: serverIniPath, content: newServerText })
  return sectionNames
}
