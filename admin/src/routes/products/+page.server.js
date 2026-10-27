export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = {};
  let products = [];
  let categories = [];
  let featuredProductIds = [];

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/products`, { headers });
    if (res.ok) {
      products = await res.json();
    }
  } catch (e) {
    console.error('Failed to load products in admin:', e);
  }

  try {
    const catRes = await fetch(`${backendUrl}/api/v1/admin/categories`, { headers });
    if (catRes.ok) {
      categories = await catRes.json();
    }
  } catch (e) {
    console.error('Failed to load categories in admin products:', e);
  }

  let storeSettings = {};

  try {
    const setRes = await fetch(`${backendUrl}/api/v1/admin/settings`, { headers });
    if (setRes.ok) {
      const settings = await setRes.json();
      storeSettings = settings;
      const sections = settings?.carousels_config?.sections || [];
      const feat = sections.find(s => s.id === 'featured');
      if (feat && Array.isArray(feat.product_ids)) {
        featuredProductIds = feat.product_ids;
      }
    }
  } catch (e) {
    console.error('Failed to load settings in admin products:', e);
  }

  let inventory = [];

  try {
    const invRes = await fetch(`${backendUrl}/api/v1/admin/logistics/inventory`, { headers });
    if (invRes.ok) {
      inventory = await invRes.json();
    }
  } catch (e) {
    console.error('Failed to load inventory in admin products:', e);
  }

  let coupons = [];

  try {
    const coupRes = await fetch(`${backendUrl}/api/v1/admin/coupons`, { headers });
    if (coupRes.ok) {
      coupons = await coupRes.json();
    }
  } catch (e) {
    console.error('Failed to load coupons in admin products:', e);
  }

  let bomParts = [];

  try {
    const bomRes = await fetch(`${backendUrl}/api/v1/admin/bom-parts`, { headers });
    if (bomRes.ok) {
      bomParts = await bomRes.json();
    }
  } catch (e) {
    console.error('Failed to load BOM parts in admin products:', e);
  }

  return { products, categories, featuredProductIds, inventory, storeSettings, coupons, bomParts };
}


