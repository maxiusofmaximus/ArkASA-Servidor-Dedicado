import type { ServerConfig } from '../types'

export interface PluginCatalogView {
  id: string
  label: string
  channel: string
  capabilities: string[]
  requiredSecrets: string[]
  oauthUrl: string | null
  enabled: boolean
  installed: boolean
  hasRequiredSecrets: boolean
}

export interface ConnectionPluginView {
  id: string
  label: string
  description: string
  freeTier?: boolean
  requiresCli?: string[]
  requiresCredentials?: boolean
  docsUrl?: string
}

export interface ModelPluginView {
  id: string
  label: string
  description: string
  defaultBaseUrl?: string
  defaultModel?: string
  requiresApiKey?: boolean
  isLocal?: boolean
  installHint?: string
  docsUrl?: string
}

export type CatalogCard =
  | { kind: 'chat'; data: PluginCatalogView }
  | { kind: 'conn'; data: ConnectionPluginView }
  | { kind: 'model'; data: ModelPluginView }

export interface CatalogLoadError {
  command: string
  message: string
}

export interface PluginCatalogResult {
  cards: CatalogCard[]
  errors: CatalogLoadError[]
}

type Invoke = <T>(command: string, args?: unknown) => Promise<T>

const CATALOG_REQUESTS = [
  { command: 'list_plugin_catalog', kind: 'chat' as const },
  { command: 'list_connection_plugins', kind: 'conn' as const },
  { command: 'list_model_plugins', kind: 'model' as const },
]

function messageFromError(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

export async function loadPluginCatalog(
  invoke: Invoke,
  config: ServerConfig,
): Promise<PluginCatalogResult> {
  const results = await Promise.allSettled(
    CATALOG_REQUESTS.map((request) => {
      const args = request.kind === 'chat' ? { config } : undefined
      return invoke<unknown[]>(request.command, args)
    }),
  )

  const cards: CatalogCard[] = []
  const errors: CatalogLoadError[] = []

  results.forEach((result, index) => {
    const request = CATALOG_REQUESTS[index]
    if (result.status === 'rejected') {
      errors.push({ command: request.command, message: messageFromError(result.reason) })
      return
    }

    for (const data of result.value) {
      if (request.kind === 'chat') cards.push({ kind: 'chat', data: data as PluginCatalogView })
      if (request.kind === 'conn') cards.push({ kind: 'conn', data: data as ConnectionPluginView })
      if (request.kind === 'model') cards.push({ kind: 'model', data: data as ModelPluginView })
    }
  })

  return { cards, errors }
}
