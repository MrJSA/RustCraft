export async function load({ fetch, url }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const category = url.searchParams.get('category') || '';
  const subcategory = url.searchParams.get('subcategory') || '';
  const search = url.searchParams.get('search') || '';

  let queryUrl = `${backendUrl}/api/v1/products?`;
  if (category) queryUrl += `category=${encodeURIComponent(category)}&`;
  if (subcategory) queryUrl += `subcategory=${encodeURIComponent(subcategory)}&`;
  if (search) queryUrl += `search=${encodeURIComponent(search)}&`;

  let products = [];
  try {
    const res = await fetch(queryUrl);
    if (res.ok) {
      products = await res.json();
    }
  } catch (e) {
    console.error('Failed to fetch products from backend:', e);
  }

  let carousels = [];
  try {
    const cRes = await fetch(`${backendUrl}/api/v1/products/carousels`);
    if (cRes.ok) {
      carousels = await cRes.json();
    }
  } catch (e) {
    console.error('Failed to fetch carousels from backend:', e);
  }

  let heroConfig = null;
  try {
    const sRes = await fetch(`${backendUrl}/api/v1/store/info`);
    if (sRes.ok) {
      const sData = await sRes.json();
      heroConfig = sData.store?.hero_config || null;
    }
  } catch (e) {
    console.error('Failed to load store hero config:', e);
  }

  let categories = [];
  try {
    const catRes = await fetch(`${backendUrl}/api/v1/categories`);
    if (catRes.ok) {
      categories = await catRes.json();
    }
  } catch (e) {
    console.error('Failed to load categories:', e);
  }

  return {
    products,
    carousels,
    categories,
    currentCategory: category,
    currentSubcategory: subcategory,
    currentSearch: search,
    heroConfig
  };
}
