// DEPRECATED: This file was an exact duplicate of secure-store-browser.ts.
// It has been consolidated into secure-store-browser.ts (AES-256-GCM encrypted storage).
// This file re-exports from the canonical implementation for backwards compatibility.
export {
  secureStore,
  storePassword,
  getPassword,
  removePassword,
  storeBookmarks,
  getBookmarks,
  storeFavorites,
  getFavorites,
  storeSchedulerTasks,
  getSchedulerTasks,
  checkBrowserSupport,
  getStorageInfo,
} from './secure-store-browser'
