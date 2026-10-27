import { redirect } from '@sveltejs/kit';
import { SESSION_COOKIE, backendUrl, fetchSessionAdmin } from '$lib/server/session.js';

// Reachable without a session: the login page and the login endpoint itself.
const PUBLIC_PATHS = new Set(['/login', '/api/v1/admin/auth/login']);

const SECURITY_HEADERS = {
  // SAMEORIGIN: the admin previews its own invoice PDFs and packing slips in a frame
  'X-Frame-Options': 'SAMEORIGIN',
  'X-Content-Type-Options': 'nosniff',
  'Referrer-Policy': 'same-origin',
  'Permissions-Policy': 'camera=(), microphone=(), geolocation=(), payment=()',
  'Cross-Origin-Opener-Policy': 'same-origin'
};

// Admin section of a page path (same sections as the backend permission system)
export function sectionForPath(path) {
  if (path === '/' || path.startsWith('/analytics')) return 'overview';
  if (path.startsWith('/products') || path.startsWith('/categories') || path.startsWith('/logistics')) return 'products';
  if (path.startsWith('/orders')) return 'orders';
  if (path.startsWith('/settings/system') || path.startsWith('/settings/menu') || path.startsWith('/settings/pages')) return 'storefront';
  if (path.startsWith('/settings')) return 'settings';
  return null;
}

const SECTION_HOME = {
  overview: '/',
  products: '/products',
  orders: '/orders',
  storefront: '/settings/system',
  settings: '/settings'
};

export async function handle({ event, resolve }) {
  const token = event.cookies.get(SESSION_COOKIE) || null;
  event.locals.adminToken = token;
  event.locals.admin = null;

  const path = event.url.pathname;
  if (!PUBLIC_PATHS.has(path)) {
    const admin = await fetchSessionAdmin(token);
    if (!admin) {
      if (path.startsWith('/api/') || path.startsWith('/uploads/')) {
        return new Response(JSON.stringify({ error: 'Please log in', code: 'unauthenticated' }), {
          status: 401,
          headers: { 'Content-Type': 'application/json' }
        });
      }
      redirect(303, `/login?next=${encodeURIComponent(path + event.url.search)}`);
    }
    event.locals.admin = admin;

    // Pages of sections this admin may not use redirect to the first allowed section
    const section = sectionForPath(path);
    const perms = admin.permissions || {};
    if (section && !admin.is_default && perms[section] === false) {
      const fallback = Object.keys(SECTION_HOME).find((s) => perms[s]);
      if (!fallback) {
        return new Response('Your account has no admin sections enabled. Please ask a superadmin.', {
          status: 403,
          headers: { 'Content-Type': 'text/plain; charset=utf-8' }
        });
      }
      redirect(303, SECTION_HOME[fallback]);
    }
  }

  const response = await resolve(event);
  for (const [name, value] of Object.entries(SECURITY_HEADERS)) {
    response.headers.set(name, value);
  }
  // Admin pages contain customer data: never let shared caches store them
  if (!path.startsWith('/_app/')) response.headers.set('Cache-Control', 'no-store');
  return response;
}

/**
 * Server-side `fetch` calls from load functions to the backend automatically carry the
 * logged-in admin's token; the legacy development bypass header is always removed.
 */
export async function handleFetch({ event, request, fetch }) {
  if (request.url.startsWith(backendUrl())) {
    const headers = new Headers(request.headers);
    headers.delete('x-dev-mode');
    headers.delete('cookie');
    if (event.locals.adminToken) headers.set('Authorization', `Bearer ${event.locals.adminToken}`);
    else headers.delete('authorization');
    request = new Request(request, { headers });
  }
  return fetch(request);
}
