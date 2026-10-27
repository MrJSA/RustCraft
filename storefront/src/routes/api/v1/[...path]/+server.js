const FORBIDDEN_HEADERS = new Set([
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
  // Never forward the backend's development auth bypass from public visitors
  'x-dev-mode',
  // Set by this proxy only (proof of the newsletter double opt-in)
  'x-shop-client-ip'
]);

// The public storefront must never expose the admin API; that is only reachable through the admin app.
function isBlockedPath(path) {
  let normalized = '';
  try {
    normalized = decodeURIComponent(path || '');
  } catch (_) {
    return true;
  }
  normalized = normalized.replace(/^\/+/, '').toLowerCase();
  return normalized === 'admin' || normalized.startsWith('admin/') || normalized.includes('..');
}

function notFound() {
  return new Response(JSON.stringify({ error: 'Not found' }), {
    status: 404,
    headers: { 'Content-Type': 'application/json' }
  });
}

function targetUrl(params, url) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const search = new URLSearchParams(url.search);
  search.delete('dev');
  const query = search.toString();
  return `${backendUrl}/api/v1/${params.path}${query ? `?${query}` : ''}`;
}

function cleanHeaders(request, clientIp) {
  const headers = {};
  if (clientIp) headers['x-shop-client-ip'] = clientIp;
  for (const [key, value] of request.headers.entries()) {
    if (!FORBIDDEN_HEADERS.has(key.toLowerCase())) {
      headers[key] = value;
    }
  }
  return headers;
}

async function proxy(method, { params, url, request, getClientAddress }) {
  if (isBlockedPath(params.path)) return notFound();

  let body;
  if (method === 'POST' || method === 'PUT') {
    const contentType = request.headers.get('content-type') || '';
    if (contentType.includes('multipart/form-data') || contentType.includes('image/') || contentType.includes('octet-stream')) {
      body = await request.arrayBuffer();
    } else {
      body = await request.text();
    }
  }

  try {
    let clientIp = '';
    try {
      clientIp = getClientAddress();
    } catch (_) {
      // not available (e.g. during prerendering)
    }
    const isDownload = params.path.startsWith('downloads/');
    const res = await globalThis.fetch(targetUrl(params, url), {
      method,
      headers: cleanHeaders(request, clientIp),
      body,
      // Externally hosted downloads: the browser follows the redirect, not this server
      redirect: isDownload ? 'manual' : 'follow'
    });
    if (isDownload && res.ok) {
      // Stream files instead of buffering them, keep their name
      const headers = { 'Cache-Control': 'private, no-store', 'X-Content-Type-Options': 'nosniff' };
      for (const name of ['content-type', 'content-length', 'content-disposition']) {
        const value = res.headers.get(name);
        if (value) headers[name] = value;
      }
      return new Response(res.body, { status: res.status, headers });
    }
    if (isDownload && res.status >= 300 && res.status < 400 && res.headers.get('location')) {
      return new Response(null, { status: 302, headers: { Location: res.headers.get('location'), 'Cache-Control': 'no-store' } });
    }
    const contentType = res.headers.get('content-type') || '';
    // Backend errors are plain text; clients consistently receive `{ error }` JSON
    if (res.status >= 400 && !contentType.includes('application/json')) {
      const text = await res.text();
      return new Response(JSON.stringify({ error: text || res.statusText }), {
        status: res.status,
        headers: { 'Content-Type': 'application/json', 'Cache-Control': 'no-store' }
      });
    }
    const data = await res.arrayBuffer();
    return new Response(data, {
      status: res.status,
      headers: { 'Content-Type': contentType || 'application/json', 'Cache-Control': 'no-store' }
    });
  } catch (err) {
    console.error(`Storefront ${method} Proxy error:`, err);
    return new Response(JSON.stringify({ error: err.message }), {
      status: 502,
      headers: { 'Content-Type': 'application/json' }
    });
  }
}

export const GET = (event) => proxy('GET', event);
export const POST = (event) => proxy('POST', event);
export const PUT = (event) => proxy('PUT', event);
export const DELETE = (event) => proxy('DELETE', event);
