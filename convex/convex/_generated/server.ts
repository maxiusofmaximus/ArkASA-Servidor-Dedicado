// Stub. Replaced with the real generated bindings by `npx convex dev`.
// These three exports keep TS happy in editor / CI while the local
// code is being authored; once the Convex Cloud dashboard is linked and
// `npx convex dev` runs, _generated/* is rewritten with full types.
export const action: any          = (..._args: unknown[]) => ({} as any)
export const query: any           = (..._args: unknown[]) => ({} as any)
export const mutation: any        = (..._args: unknown[]) => ({} as any)
export const internalAction: any  = (..._args: unknown[]) => ({} as any)
export const internalQuery: any   = (..._args: unknown[]) => ({} as any)
export const internalMutation: any = (..._args: unknown[]) => ({} as any)
export const internal: any = new Proxy({}, {
  get: () => new Proxy({}, {
    get: () => () => ({} as any),
  }),
})
