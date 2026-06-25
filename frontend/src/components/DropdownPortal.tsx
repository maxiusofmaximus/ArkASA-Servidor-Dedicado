import { useState, useEffect, useCallback } from 'react'
import { createPortal } from 'react-dom'

interface DropdownPortalProps {
  anchorRef: React.RefObject<HTMLElement | null>
  children: React.ReactNode
  open: boolean
}

export default function DropdownPortal({ anchorRef, children, open }: DropdownPortalProps) {
  const [pos, setPos] = useState<React.CSSProperties>({})

  const update = useCallback(() => {
    const el = anchorRef.current
    if (!el) return
    const rect = el.getBoundingClientRect()
    setPos({
      position: 'fixed',
      bottom: window.innerHeight - rect.top + 4,
      left: rect.left,
      minWidth: Math.max(rect.width, 200),
    })
  }, [anchorRef])

  useEffect(() => {
    if (!open) return
    update()
    window.addEventListener('resize', update)
    window.addEventListener('scroll', update, true)
    return () => {
      window.removeEventListener('resize', update)
      window.removeEventListener('scroll', update, true)
    }
  }, [open, update])

  if (!open) return null
  return createPortal(
    <div style={pos} className="z-[9999] rounded-md py-1 ark-panel" onClick={(e) => e.stopPropagation()}>
      {children}
    </div>,
    document.body
  )
}
