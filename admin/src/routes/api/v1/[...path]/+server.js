import { backendUrl, setSessionCookie, clearSessionCookie } from '$lib/server/session.js';

// Hop-by-hop headers plus anything the browser must not be able to inject towards the backend.
const STRIPPED_REQUEST_HEADERS = new Set([
  'host',
  'connection',
  'content-length',
  'transfer-encoding',
  'content-encoding',
  'expect',
  'upgrade',
  'keep-alive',
  'proxy-connection',
  'proxy-authenticate',
  'proxy-authorization',
  'te',
  'trailer',
  'cookie',
  'authorization',
  'x-dev-mode'
]);

const FORWARDED_RESPONSE_HEADERS = ['content-type', 'content-disposition'];

function json(body, status = 200) {
  return new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } });
}

function backendHeaders(request, locals) {
  const headers = {};
  for (const [key, value] of request.headers.entries()) {
    if (!STRIPPED_REQUEST_HEADERS.has(key.toLowerCase())) headers[key] = value;
  }
  // The only credential the backend ever sees is the session from the httpOnly cookie
  if (locals.adminToken) headers['Authorization'] = `Bearer ${locals.adminToken}`;
  return headers;
}

async function readBody(request) {
  if (request.method === 'GET' || request.method === 'HEAD' || request.method === 'DELETE') return undefined;
  const contentType = request.headers.get('content-type') || '';
  if (contentType.includes('multipart/form-data') || contentType.includes('image/') || contentType.includes('octet-stream') || contentType.includes('application/zip')) {
    return request.arrayBuffer();
  }
  return request.text();
}

/** Backend errors are plain text; the admin UI expects `{ error }` JSON. */
async function toClientResponse(res) {
  const contentType = res.headers.get('content-type') || '';
  if (res.status >= 400 && !contentType.includes('application/json')) {
    const text = await res.text();
    return json({ error: text || res.statusText }, res.status);
  }
  const headers = {};
  for (const name of FORWARDED_RESPONSE_HEADERS) {
    const value = res.headers.get(name);
    if (value) headers[name] = value;
  }
  if (!headers['content-type']) headers['content-type'] = 'application/json';
  return new Response(await res.arrayBuffer(), { status: res.status, headers });
}

async function proxy({ params, url, request, cookies, locals }) {
  const path = params.path || '';

  // Logout never needs the backend
  if (path === 'admin/auth/logout') {
    clearSessionCookie(cookies);
    return json({ success: true });
  }

  const search = new URLSearchParams(url.search);
  search.delete('dev');
  search.delete('token');
  const query = search.toString();
  const target = `${backendUrl()}/api/v1/${path}${query ? `?${query}` : ''}`;

  let res;
  try {
    res = await fetch(target, {
      method: request.method,
      headers: backendHeaders(request, locals),
      body: await readBody(request)
    });
  } catch (err) {
    console.error(`Admin ${request.method} proxy error:`, err);
    return json({ error: 'The shop backend is not reachable.' }, 502);
  }

  // Login and credential changes return a token: keep it in the httpOnly cookie, never hand it to the page
  if (res.ok && (path === 'admin/auth/login' || path === 'admin/auth/change-credentials')) {
    const data = await res.json();
    if (data.token) {
      setSessionCookie(cookies, data.token, url);
      delete data.token;
    }
    return json(data);
  }

  return toClientResponse(res);
}

export const GET = proxy;
export const POST = proxy;
export const PUT = proxy;
export const PATCH = proxy;
export const DELETE = proxy;
