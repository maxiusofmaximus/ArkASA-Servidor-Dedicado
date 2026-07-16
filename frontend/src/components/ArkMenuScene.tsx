import type { CSSProperties } from 'react'

interface ArkMenuSceneProps {
  fallbackImage: string
}

const constellationNodes = [
  { x: 222, y: 238, tone: 'cyan' },
  { x: 274, y: 194, tone: 'violet' },
  { x: 328, y: 244, tone: 'pink' },
  { x: 186, y: 338, tone: 'violet' },
  { x: 248, y: 378, tone: 'cyan' },
  { x: 324, y: 342, tone: 'pink' },
  { x: 214, y: 526, tone: 'pink' },
  { x: 280, y: 564, tone: 'violet' },
  { x: 348, y: 510, tone: 'cyan' },
  { x: 688, y: 206, tone: 'cyan' },
  { x: 754, y: 170, tone: 'pink' },
  { x: 824, y: 232, tone: 'violet' },
  { x: 700, y: 354, tone: 'pink' },
  { x: 784, y: 326, tone: 'cyan' },
  { x: 852, y: 404, tone: 'violet' },
  { x: 670, y: 522, tone: 'violet' },
  { x: 748, y: 572, tone: 'pink' },
  { x: 834, y: 540, tone: 'cyan' },
] as const

/**
 * An original CSS/SVG interpretation of the ARK-style orbital menu: the
 * artwork remains a clean master image, while the constellation uses explicit
 * tracks so energy travels through sectors instead of spinning as one texture.
 */
export default function ArkMenuScene({ fallbackImage }: ArkMenuSceneProps) {
  return (
    <div
      className="ark-scene"
      style={{ '--ark-scene-fallback': `url("${fallbackImage}")` } as CSSProperties}
      aria-hidden="true"
    >
      <div className="ark-scene-master" />
      <div className="ark-scene-planet-rotation" />
      <div className="ark-scene-matrix">
        <svg className="ark-orbital-svg" viewBox="0 0 1000 760" preserveAspectRatio="xMidYMid meet">
          <defs>
            <filter id="ark-orbit-glow" x="-200%" y="-200%" width="400%" height="400%">
              <feGaussianBlur stdDeviation="3" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
            <path id="ark-orbit-left" d="M 388 620 C 147 552 140 278 384 112" />
            <path id="ark-orbit-right" d="M 612 112 C 857 214 866 507 624 656" />
            <path id="ark-orbit-upper" d="M 270 182 C 456 42 704 55 824 240" />
            <path id="ark-orbit-lower" d="M 176 510 C 372 720 676 733 856 514" />
            <g id="ark-orbit-glyph">
              <circle r="3.3" className="ark-orbit-glyph-core" />
              <path d="M 0 -9 L 7.8 -4.5 L 7.8 4.5 L 0 9 L -7.8 4.5 L -7.8 -4.5 Z" className="ark-orbit-glyph-hex" />
            </g>
          </defs>

          <g className="ark-orbit-routes">
            <use href="#ark-orbit-left" className="ark-orbit-route ark-orbit-route--violet" />
            <use href="#ark-orbit-right" className="ark-orbit-route ark-orbit-route--cyan" />
            <use href="#ark-orbit-upper" className="ark-orbit-route ark-orbit-route--upper" />
            <use href="#ark-orbit-lower" className="ark-orbit-route ark-orbit-route--lower" />
          </g>

          <g className="ark-constellation ark-constellation--left">
            <path className="ark-constellation-link" d="M 186 338 L 222 238 L 274 194 L 328 244 L 324 342 L 248 378 L 186 338" />
            <path className="ark-constellation-link" d="M 214 526 L 248 378 L 324 342 L 348 510 L 280 564 L 214 526" />
            <path className="ark-constellation-link ark-constellation-link--dim" d="M 222 238 L 324 342 M 248 378 L 348 510" />
          </g>
          <g className="ark-constellation ark-constellation--right">
            <path className="ark-constellation-link" d="M 688 206 L 754 170 L 824 232 L 784 326 L 700 354 L 688 206" />
            <path className="ark-constellation-link" d="M 700 354 L 784 326 L 852 404 L 834 540 L 748 572 L 670 522 L 700 354" />
            <path className="ark-constellation-link ark-constellation-link--dim" d="M 824 232 L 700 354 M 784 326 L 748 572" />
          </g>

          <g className="ark-constellation-nodes">
            {constellationNodes.map((node, index) => (
              <g
                className={`ark-constellation-node ark-constellation-node--${node.tone}`}
                key={`${node.x}-${node.y}`}
              style={{ '--constellation-delay': `${-(index * 0.43)}s` } as CSSProperties}
              transform={`translate(${node.x} ${node.y})`}
            >
                <use className="ark-constellation-glyph" href="#ark-orbit-glyph" />
              </g>
            ))}
          </g>

          <g className="ark-energy-streams" filter="url(#ark-orbit-glow)">
            <path className="ark-energy-stream ark-energy-stream--left" d="M 386 618 C 145 552 141 279 383 114" />
            <path className="ark-energy-stream ark-energy-stream--right" d="M 614 114 C 856 215 864 507 625 654" />
            <path className="ark-energy-stream ark-energy-stream--lower" d="M 177 510 C 373 719 675 731 854 515" />
          </g>

          <g className="ark-orbit-travellers" filter="url(#ark-orbit-glow)">
            <use href="#ark-orbit-glyph" className="ark-orbit-traveller ark-orbit-traveller--pink">
              <animateMotion dur="18s" repeatCount="indefinite" rotate="auto">
                <mpath href="#ark-orbit-left" />
              </animateMotion>
            </use>
            <use href="#ark-orbit-glyph" className="ark-orbit-traveller ark-orbit-traveller--cyan">
              <animateMotion dur="23s" begin="-9s" repeatCount="indefinite" rotate="auto">
                <mpath href="#ark-orbit-right" />
              </animateMotion>
            </use>
            <use href="#ark-orbit-glyph" className="ark-orbit-traveller ark-orbit-traveller--violet">
              <animateMotion dur="27s" begin="-16s" repeatCount="indefinite" rotate="auto">
                <mpath href="#ark-orbit-lower" />
              </animateMotion>
            </use>
          </g>
        </svg>
      </div>
      <div className="ark-scene-nebula" />
      <div className="ark-scene-stars" />
      <div className="ark-scene-grain" />
    </div>
  )
}
