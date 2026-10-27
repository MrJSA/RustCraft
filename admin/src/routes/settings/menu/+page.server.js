export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  let menuItems = [];
  let categories = [];
  let pages = [];

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/menu`, {
      headers: {}
    });
    if (res.ok) menuItems = await res.json();
  } catch (e) {
    console.error('Failed to load menu items:', e);
  }

  try {
    const cRes = await fetch(`${backendUrl}/api/v1/admin/categories`, {
      headers: {}
    });
    if (cRes.ok) categories = await cRes.json();
  } catch (e) {
    console.error('Failed to load categories:', e);
  }

  try {
    const pRes = await fetch(`${backendUrl}/api/v1/admin/pages`, {
      headers: {}
    });
    if (pRes.ok) pages = await pRes.json();
  } catch (e) {
    console.error('Failed to load pages:', e);
  }

  return { menuItems, categories, pages };
}
