export async function GET({ params, fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const filePath = params.file || '';

  // Only plain file names inside the uploads folder
  if (!filePath || filePath.includes('..') || filePath.includes('\\')) {
    return new Response('Not Found', { status: 404 });
  }

  try {
    const response = await fetch(`${backendUrl}/uploads/${encodeURI(filePath)}`);
    if (!response.ok) {
      return new Response('Not Found', { status: response.status });
    }

    const contentType = response.headers.get('content-type') || 'application/octet-stream';
    const cacheControl = response.headers.get('cache-control') || 'public, max-age=86400';

    return new Response(response.body, {
      status: 200,
      headers: {
        'content-type': contentType,
        'cache-control': cacheControl,
        // Uploaded files are data, never active content (e.g. scripts embedded in SVGs)
        'content-security-policy': "default-src 'none'; img-src 'self' data:; style-src 'unsafe-inline'; sandbox",
        'x-content-type-options': 'nosniff'
      }
    });
  } catch (err) {
    return new Response('Internal Server Error', { status: 500 });
  }
}
