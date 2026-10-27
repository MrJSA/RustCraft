export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/pages`, {
      headers: {}
    });
    if (res.ok) {
      const pages = await res.json();
      return { pages };
    }
  } catch (e) {
    console.error('Failed to load admin pages:', e);
  }

  return { pages: [] };
}
