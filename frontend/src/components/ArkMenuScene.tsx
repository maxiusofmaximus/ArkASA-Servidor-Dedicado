import { useEffect, useRef, type CSSProperties } from 'react'

interface ArkMenuSceneProps {
  fallbackImage: string
}

/** A UI-free Blender render. The fallback remains available if video decoding fails. */
export default function ArkMenuScene({ fallbackImage }: ArkMenuSceneProps) {
  const videoRef = useRef<HTMLVideoElement>(null)

  useEffect(() => {
    const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
    const syncMotionPreference = () => {
      const video = videoRef.current
      if (!video) return

      if (motionQuery.matches) {
        video.pause()
      } else {
        void video.play().catch(() => {
          // The still poster/fallback remains readable if playback is unavailable.
        })
      }
    }

    syncMotionPreference()
    motionQuery.addEventListener('change', syncMotionPreference)
    return () => motionQuery.removeEventListener('change', syncMotionPreference)
  }, [])

  return (
    <div
      className="ark-scene"
      style={{ '--ark-scene-fallback': `url("${fallbackImage}")` } as CSSProperties}
      aria-hidden="true"
    >
      <video
        ref={videoRef}
        className="ark-scene-video"
        autoPlay
        loop
        muted
        playsInline
        preload="auto"
        poster="/assets/ark-menu-master.png"
      >
        <source src="/assets/ark-orbital-loop.mp4" type="video/mp4" />
      </video>
    </div>
  )
}
