export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = {};

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/settings/shipping/providers`, { headers });
    if (res.ok) {
      const providers = await res.json();
      return { providers };
    }
  } catch (e) {
    console.error('Failed to load shipping providers:', e);
  }

  return { providers: [] };
}
