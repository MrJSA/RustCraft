// In-memory cache of rendered policy pages (cleared by the backend when pages or settings change)
export const pageCache = new Map();

/** Entries older than this are reloaded, so a missed invalidation never lasts long. */
export const PAGE_CACHE_MS = 5 * 60 * 1000;
