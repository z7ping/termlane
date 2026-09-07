export type HostKeyErrorCode = 'HOST_KEY_UNKNOWN' | 'HOST_KEY_MISMATCH'

export interface HostKeyErrorDetail {
  code: HostKeyErrorCode
  host: string
  port: number
  algorithm: string
  fingerprint: string
}

function errorText(error: unknown): string {
  if (error instanceof Error) return error.message
  return String(error ?? '')
}

/** Parse structured host-key errors returned by the Rust SSH boundary. */
export function parseHostKeyError(error: unknown): HostKeyErrorDetail | null {
  const text = errorText(error)
  const codes: HostKeyErrorCode[] = ['HOST_KEY_UNKNOWN', 'HOST_KEY_MISMATCH']

  for (const code of codes) {
    const marker = `${code}:`
    const markerIndex = text.indexOf(marker)
    if (markerIndex < 0) continue

    try {
      const raw = JSON.parse(text.slice(markerIndex + marker.length)) as Partial<HostKeyErrorDetail>
      if (
        typeof raw.host !== 'string'
        || typeof raw.port !== 'number'
        || typeof raw.algorithm !== 'string'
        || typeof raw.fingerprint !== 'string'
      ) {
        return null
      }
      return {
        code,
        host: raw.host,
        port: raw.port,
        algorithm: raw.algorithm,
        fingerprint: raw.fingerprint,
      }
    } catch {
      return null
    }
  }

  return null
}

export function hostKeyTarget(detail: Pick<HostKeyErrorDetail, 'host' | 'port'>): string {
  return detail.port === 22 ? detail.host : `${detail.host}:${detail.port}`
}
