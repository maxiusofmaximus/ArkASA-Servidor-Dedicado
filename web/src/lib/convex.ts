/**
 * Convex client setup for the Vercel-deployed admin web.
 *
 * `VITE_CONVEX_URL` must be set in `.env` (or in the Vercel project
 * dashboard) before deployment; `npm run dev` falls back to a fake URL
 * so local dev still renders the dashboard skeleton until you wire one in.
 */
import { ConvexReactClient } from 'convex/react'

const url = (import.meta.env.VITE_CONVEX_URL as string | undefined)
  ?? 'http://127.0.0.1:8787' // dummy default — UI only renders when you configure it

export const convex = new ConvexReactClient(url, {
  // Use unsigned POC mode in dev — production sets `VITE_CONVEX_TOKEN` separately.
  unsavedChangesWarning: false,
})
