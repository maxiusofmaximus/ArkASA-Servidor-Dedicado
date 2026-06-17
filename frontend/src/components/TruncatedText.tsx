import React, { useRef, useState, useEffect, useCallback } from 'react'

interface TruncatedTextProps {
  text: string
  className?: string
  style?: React.CSSProperties
}

/**
 * Renders text truncated with ellipsis when it overflows its container.
 * On hover, scrolls the text to reveal the full content (CSS marquee).
 * If the text fits without truncation, no animation is applied.
 */
export default function TruncatedText({ text, className = '', style }: TruncatedTextProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const textRef      = useRef<HTMLSpanElement>(null)
  const [overflows, setOverflows]     = useState(false)
  const [scrollDist, setScrollDist]   = useState(0)

  const measure = useCallback(() => {
    const container = containerRef.current
    const span = textRef.current
    if (!container || !span) return
    const cw = container.clientWidth
    const tw = span.scrollWidth
    setOverflows(tw > cw)
    setScrollDist(Math.max(0, tw - cw + 8))  // 8px breathing room
  }, [])

  useEffect(() => {
    measure()
    const observer = new ResizeObserver(measure)
    if (containerRef.current) observer.observe(containerRef.current)
    return () => observer.disconnect()
  }, [text, measure])

  return (
    <div
      ref={containerRef}
      className={`overflow-hidden ${overflows ? 'truncated-text-container' : ''}`}
      style={{ width: '100%', ...style }}
      title={overflows ? text : undefined}
    >
      <span
        ref={textRef}
        className={`inline-block whitespace-nowrap ${overflows ? 'truncated-text-scroll' : ''} ${className}`}
        style={overflows ? ({
          '--scroll-dist': `-${scrollDist}px`,
        } as React.CSSProperties) : undefined}
      >
        {text}
      </span>
    </div>
  )
}
