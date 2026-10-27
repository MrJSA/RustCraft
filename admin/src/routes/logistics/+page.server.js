export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = {};

  let inventory = [];
  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/logistics/inventory`, { headers });
    if (res.ok) {
      inventory = await res.json();
    }
  } catch (e) {
    console.error('Failed to load logistics inventory:', e);
  }

  let bomParts = [];
  try {
    const bomRes = await fetch(`${backendUrl}/api/v1/admin/bom-parts`, { headers });
    if (bomRes.ok) {
      bomParts = await bomRes.json();
    }
  } catch (e) {
    console.error('Failed to load BOM parts in logistics:', e);
  }

  return { inventory, bomParts };
}
