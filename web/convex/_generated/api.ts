// Stub. Replaced with the real generated bindings by `npx convex codegen`.
// Until then the dashboard skeleton renders without a working API client.
// After running `npx convex dev` once this file becomes the real generated
// output and the dashboard subscriptions start streaming live data.

export const api: any = new Proxy({}, {
  get: (_t: any, prop: string) => (..._args: any[]) => {
    if (prop.endsWith('_handler')) return undefined
    return (..._a: any[]) => { throw new Error(`convex not initialised yet (api.${prop})`) }
  },
})
