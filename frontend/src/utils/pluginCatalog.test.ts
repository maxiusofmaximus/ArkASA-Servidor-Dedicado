import { describe, expect, it } from 'vitest'
import { loadPluginCatalog } from './pluginCatalog'
import type { ServerConfig } from '../types'

describe('loadPluginCatalog', () => {
  it('passes the server config to the chat catalog and merges all catalogs', async () => {
    const calls: Array<{ command: string; args: unknown }> = []
    const invoke = async <T>(command: string, args?: unknown): Promise<T> => {
      calls.push({ command, args })
      const values: Record<string, unknown> = {
        list_plugin_catalog: [{ id: 'convex', label: 'Convex' }],
        list_connection_plugins: [{ id: 'tailscale', label: 'Tailscale' }],
        list_model_plugins: [{ id: 'ollama', label: 'Ollama' }],
      }
      return values[command] as T
    }

    const config = { installed_plugins: [] } as unknown as ServerConfig
    const result = await loadPluginCatalog(invoke, config)

    expect(result.cards.map((card) => card.data.id)).toEqual(['convex', 'tailscale', 'ollama'])
    expect(calls[0]).toEqual({ command: 'list_plugin_catalog', args: { config } })
    expect(result.errors).toEqual([])
  })

  it('keeps successful catalogs and reports rejected catalogs', async () => {
    const invoke = async <T>(command: string): Promise<T> => {
      if (command === 'list_plugin_catalog') throw new Error('missing config')
      return (command === 'list_connection_plugins'
        ? [{ id: 'tailscale', label: 'Tailscale' }]
        : [{ id: 'ollama', label: 'Ollama' }]) as T
    }

    const result = await loadPluginCatalog(invoke, { installed_plugins: [] } as unknown as ServerConfig)

    expect(result.cards).toHaveLength(2)
    expect(result.errors).toEqual([{ command: 'list_plugin_catalog', message: 'missing config' }])
  })
})
