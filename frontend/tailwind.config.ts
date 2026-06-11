import type { Config } from 'tailwindcss'

export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'ark-dark': '#0a0e27',
        'ark-secondary': '#1a2541',
        'ark-cyan': '#00d4ff',
        'ark-purple': '#9d4edd',
        'ark-accent': '#ff006e',
      },
      fontFamily: {
        'sans': ['Inter', 'sans-serif'],
        'mono': ['Fira Code', 'monospace'],
      },
      boxShadow: {
        'ark': '0 0 20px rgba(0, 212, 255, 0.3)',
      },
    },
  },
  plugins: [],
} satisfies Config
