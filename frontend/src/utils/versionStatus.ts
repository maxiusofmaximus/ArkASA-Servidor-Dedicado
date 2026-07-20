/**
 * versionStatus
 *
 * Pure helper that classifies the (local,latest) buildid pair produced
 * by `check_server_version` into a status the UI can render:
 *
 *   • current  → local === latest > 0  (or undefined pair but known ok)
 *   • outdated → local < latest            (steam has a newer build)
 *   • unknown  → at least one side is null (no appmanifest acf, network
 *                failure on `app_info_print`, etc.)
 */
import type { ServerVersionInfo } from '../types'

export type VersionStatus = 'current' | 'outdated' | 'unknown'

export function versionStatus(info: ServerVersionInfo | null): VersionStatus {
  if (!info) return 'unknown'
  const { local_buildid, latest_buildid } = info
  if (local_buildid == null || latest_buildid == null) return 'unknown'
  if (local_buildid >= latest_buildid) return 'current'
  return 'outdated'
}
