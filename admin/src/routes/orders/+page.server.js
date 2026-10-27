export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = {};

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/orders`, { headers });
    if (res.ok) {
      const orders = await res.json();
      return { orders };
    }
  } catch (e) {
    console.error('Failed to load orders:', e);
  }

  return { orders: [] };
}
