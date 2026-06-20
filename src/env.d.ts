/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>
  export default component
}

interface TauriInvokeFn {
  (cmd: string, args?: Record<string, unknown>): Promise<unknown>
}

interface TauriEventListenFn {
  (event: string, handler: (event: { payload: unknown }) => void): Promise<() => void>
}

interface Window {
  __TAURI__?: {
    core: {
      invoke: TauriInvokeFn
    }
    event: {
      listen: TauriEventListenFn
    }
  }
}
