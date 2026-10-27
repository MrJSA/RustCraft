export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = {};
  let settings = {};
  let products = [];
  let categories = [];
  let menuItems = [];
  let pages = [];

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/settings/system`, { headers });
    if (res.ok) {
      settings = await res.json();
    }
  } catch (e) {
    console.error('Failed to load system settings:', e);
  }

  try {
    const prodRes = await fetch(`${backendUrl}/api/v1/admin/products`, { headers });
    if (prodRes.ok) {
      products = await prodRes.json();
    }
  } catch (e) {
    console.error('Failed to load products in system settings:', e);
  }

  try {
    const catRes = await fetch(`${backendUrl}/api/v1/admin/categories`, { headers });
    if (catRes.ok) {
      categories = await catRes.json();
    }
  } catch (e) {
    console.error('Failed to load categories in system settings:', e);
  }

  try {
    const mRes = await fetch(`${backendUrl}/api/v1/admin/menu`, { headers });
    if (mRes.ok) {
      menuItems = await mRes.json();
    }
  } catch (e) {
    console.error('Failed to load menu in system settings:', e);
  }

  try {
    const pRes = await fetch(`${backendUrl}/api/v1/admin/pages`, { headers });
    if (pRes.ok) {
      pages = await pRes.json();
    }
  } catch (e) {
    console.error('Failed to load pages in system settings:', e);
  }

  return {
    settings: Object.keys(settings).length > 0 ? settings : {
      store_name: 'RustCraft Gear & Software',
      deployment_mode: 'development',
      debug_mode: true,
      currency: 'EUR',
      currency_symbol: '€',
      tax_rate_percent: 19.0,
      support_email: 'support@rustwebshop.local',
      phone: '+49 (0) 30 123456-78',
      company_address: 'Rustacean Way 42, 10115 Berlin, Germany',
      vat_id: 'DE314159265',
      logo_url: '',
      hero_config: { layout: 'split', carousel_items: [], featured_buttons: [] }
    },
    products,
    categories,
    menuItems,
    pages
  };
}
