import { useCallback } from 'react'
import { useConfigStore } from '../stores/configStore'
import type { ServerConfig } from '../types'

export function useConfigUpdate<K extends keyof ServerConfig>(section: K) {
  const { config, setConfig } = useConfigStore()

  return useCallback(
    <F extends keyof ServerConfig[K]>(field: F, value: ServerConfig[K][F]) => {
      if (!config) return
      setConfig({
        ...config,
        [section]: { ...config[section], [field]: value },
      })
    },
    [config, setConfig, section]
  )
}
