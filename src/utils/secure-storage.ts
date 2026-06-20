// DEPRECATED: This file previously used base64 encoding (NOT encryption) for password storage.
// It has been replaced by secure-store-browser.ts which uses AES-256-GCM encryption.
// This file now re-exports from the secure implementation for backwards compatibility.
// All new code should import from 'secure-store-browser' directly.
export { secureStore, storePassword, getPassword, removePassword } from './secure-store-browser'
