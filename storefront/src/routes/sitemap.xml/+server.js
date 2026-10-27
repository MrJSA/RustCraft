// Dynamic XML sitemap: home, categories, active products and CMS/policy pages linked in the menus.

const escapeXml = (s) =>
  String(s).replace(/[<>&'"]/g, (c) => ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', "'": '&apos;', '"': '&quot;' })[c]);

async function getJson(fetch, url) {
  try {
    const res = await fetch(url);
    return res.ok ? await res.json() : null;
  } catch (e) {
    console.error(`Sitemap: failed to load ${url}:`, e);
    return null;
  }
}

export async function GET({ url, fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const origin = url.origin;

  const [products, categories, headerMenu, footerMenu] = await Promise.all([
    getJson(fetch, `${backendUrl}/api/v1/products`),
    getJson(fetch, `${backendUrl}/api/v1/categories`),
    getJson(fetch, `${backendUrl}/api/v1/menu?location=header`),
    getJson(fetch, `${backendUrl}/api/v1/menu?location=footer`)
  ]);

  /** @type {Map<string, {lastmod?: string, priority: string}>} */
  const entries = new Map();
  const add = (path, priority, lastmod) => {
    if (!entries.has(path)) entries.set(path, { priority, lastmod });
  };

  add('/', '1.0');

  for (const c of Array.isArray(categories) ? categories : []) {
    if (c?.name) add(`/?category=${encodeURIComponent(c.name)}`, '0.7');
  }

  for (const p of Array.isArray(products) ? products : []) {
    if (p?.slug && p.is_active !== false) {
      add(`/products/${encodeURIComponent(p.slug)}`, '0.8', p.updated_at ? String(p.updated_at).slice(0, 10) : undefined);
    }
  }

  // Internal content pages (policies, imprint, ...) linked from the navigation menus
  const walk = (items) => {
    for (const item of Array.isArray(items) ? items : []) {
      const href = item?.url || '';
      if (href.startsWith('/policies/') && !href.includes('?')) add(href, '0.3');
      if (item?.children) walk(item.children);
    }
  };
  walk(headerMenu);
  walk(footerMenu);

  const xml = [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
    ...[...entries].map(
      ([path, e]) =>
        `  <url><loc>${escapeXml(origin + path)}</loc>${e.lastmod ? `<lastmod>${e.lastmod}</lastmod>` : ''}<priority>${e.priority}</priority></url>`
    ),
    '</urlset>',
    ''
  ].join('\n');

  return new Response(xml, {
    headers: {
      'Content-Type': 'application/xml; charset=utf-8',
      'Cache-Control': 'public, max-age=3600'
    }
  });
}
