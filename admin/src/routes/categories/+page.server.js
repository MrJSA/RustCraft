export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  let categories = [];
  let products = [];

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/categories`, {
      headers: {}
    });
    if (res.ok) {
      categories = await res.json();
    }
  } catch (e) {
    console.error('Failed to load categories:', e);
  }

  try {
    const pRes = await fetch(`${backendUrl}/api/v1/admin/products`, {
      headers: {}
    });
    if (pRes.ok) {
      products = await pRes.json();
    }
  } catch (e) {
    console.error('Failed to load products in categories:', e);
  }

  return { categories, products };
}
