import { error } from '@sveltejs/kit';
import { pageCache, PAGE_CACHE_MS } from '$lib/server/pageCache.js';

export async function load({ params, fetch, setHeaders }) {
  const { slug } = params;
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';

  // Legal texts must reflect changes quickly: short browser cache
  setHeaders({
    'Cache-Control': 'public, max-age=300, stale-while-revalidate=600'
  });

  // In-memory server cache (cleared by the backend when pages or settings change)
  const cached = pageCache.get(slug);
  if (cached && cached.page && Date.now() - (cached.at || 0) < PAGE_CACHE_MS) {
    return {
      page: cached.page,
      providers: cached.providers || []
    };
  }

  try {
    const res = await fetch(`${backendUrl}/api/v1/pages/${slug}`);
    if (res.ok) {
      const data = await res.json();
      const payload = {
        page: data.page,
        providers: data.providers || []
      };
      pageCache.set(slug, { ...payload, at: Date.now() });
      return payload;
    }
  } catch (e) {
    console.error(`Failed to fetch policy page ${slug}:`, e);
  }

  throw error(404, 'Policy page not found');
}
