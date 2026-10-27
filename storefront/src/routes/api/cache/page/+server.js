import { json } from '@sveltejs/kit';
import { pageCache } from '$lib/server/pageCache.js';

// Called by the backend after a policy page or the shop settings changed. It only forgets cached
// pages (they are reloaded from the backend with all placeholders filled in), so a call from
// anywhere else can do no harm.
export async function POST({ request }) {
  let slug = '*';
  try {
    slug = (await request.json())?.slug || '*';
  } catch (_) {
    // no body: forget everything
  }
  if (slug === '*') {
    pageCache.clear();
  } else {
    pageCache.delete(String(slug));
  }
  return json({ success: true });
}
