/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{ts,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        ark: {
          dark: '#0a0e27',
          secondary: '#1a2541',
          cyan: '#00d4ff',
          purple: '#9d4edd',
          accent: '#ff006e',
          glow: 'rgba(0, 212, 255, 0.8)',
        },
      },
    },
  },
  plugins: [],
}
