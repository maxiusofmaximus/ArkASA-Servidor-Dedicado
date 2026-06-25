import { useCallback, useState } from 'react'

const MAX_STEPS = 100

type Stack = { items: string[]; index: number }

/**
 * Undo/redo stack for a single text buffer (e.g. raw INI editor).
 */
export function useTextHistory(initial: string) {
  const [stack, setStack] = useState<Stack>({ items: [initial], index: 0 })

  const text = stack.items[stack.index] ?? initial

  const commit = useCallback((value: string) => {
    setStack((s) => {
      const truncated = s.items.slice(0, s.index + 1)
      if (truncated[truncated.length - 1] === value) return s
      let items = [...truncated, value]
      let index = items.length - 1
      if (items.length > MAX_STEPS) {
        items = items.slice(1)
        index = items.length - 1
      }
      return { items, index }
    })
  }, [])

  const reset = useCallback((value: string) => {
    setStack({ items: [value], index: 0 })
  }, [])

  const undo = useCallback(() => {
    setStack((s) => (s.index > 0 ? { ...s, index: s.index - 1 } : s))
  }, [])

  const redo = useCallback(() => {
    setStack((s) =>
      s.index < s.items.length - 1 ? { ...s, index: s.index + 1 } : s
    )
  }, [])

  return {
    text,
    commit,
    reset,
    undo,
    redo,
    canUndo: stack.index > 0,
    canRedo: stack.index < stack.items.length - 1,
  }
}
