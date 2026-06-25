import { useState, useRef, useCallback, useEffect, type ReactNode } from 'react'
import { createPortal } from 'react-dom'

interface TooltipProps {
  content: string
  children: ReactNode
  delay?: number
  maxWidth?: number
}

interface Pos { top: number; left: number }

function calcPos(trigger: DOMRect, tipW: number, tipH: number): Pos {
  const above = trigger.top
  const top = above >= tipH + 10 ? trigger.top - tipH - 8 : trigger.bottom + 8
  let left = trigger.left + trigger.width / 2 - tipW / 2
  left = Math.max(8, Math.min(left, window.innerWidth - tipW - 8))
  return { top, left }
}

export default function Tooltip({ content, children, delay = 400, maxWidth = 280 }: TooltipProps) {
  const [visible, setVisible] = useState(false)
  const [pos, setPos] = useState<Pos | null>(null)
  const triggerRef = useRef<HTMLDivElement>(null)
  const tipRef = useRef<HTMLDivElement>(null)
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined)

  const show = useCallback(() => {
    clearTimeout(timer.current)
    timer.current = setTimeout(() => {
      setVisible(true)
      requestAnimationFrame(() => {
        if (!triggerRef.current || !tipRef.current) return
        const tr = triggerRef.current.getBoundingClientRect()
        setPos(calcPos(tr, tipRef.current.offsetWidth, tipRef.current.offsetHeight))
      })
    }, delay)
  }, [delay])

  const hide = useCallback(() => {
    clearTimeout(timer.current)
    setVisible(false)
    setPos(null)
  }, [])

  useEffect(() => () => clearTimeout(timer.current), [])

  return (
    <>
      <div
        ref={triggerRef}
        style={{ display: 'inline-block' }}
        onMouseEnter={show}
        onMouseLeave={hide}
        onFocus={show}
        onBlur={hide}
      >
        {children}
      </div>
      {visible && createPortal(
        <div
          ref={tipRef}
          role="tooltip"
          style={{
            position: 'fixed',
            top:  pos?.top  ?? -9999,
            left: pos?.left ?? -9999,
            maxWidth,
            zIndex: 9999,
            pointerEvents: 'none',
            opacity: pos ? 1 : 0,
            transition: 'opacity 0.12s ease',
            background: 'rgba(2,12,24,0.97)',
            border: '1px solid rgba(0,200,255,0.28)',
            color: 'rgba(200,240,255,0.88)',
            boxShadow: '0 4px 24px rgba(0,0,0,0.7)',
            borderRadius: '4px',
            padding: '7px 11px',
            fontSize: '11px',
            lineHeight: '1.55',
            overflowWrap: 'break-word',
          }}
        >
          {content}
        </div>,
        document.body
      )}
    </>
  )
}
