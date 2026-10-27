export async function load({ fetch, locals }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const admin = locals.admin || null;
  // Public storefront URL for the "Open Storefront" / preview links (any domain, no trailing slash)
  const shopUrl = (process.env.SHOP_PUBLIC_URL || 'http://localhost:8080').replace(/\/+$/, '');

  // Logged-out visitors (login page) get no store data at all
  if (admin && !admin.is_default) {
    try {
      const res = await fetch(`${backendUrl}/api/v1/admin/settings/system`);
      if (res.ok) {
        return { admin, shopUrl, settings: await res.json() };
      }
    } catch (e) {
      console.error('Failed to load system settings in admin layout:', e);
    }
  }

  return {
    admin,
    shopUrl,
    settings: {
      store_name: 'Shop Admin',
      deployment_mode: 'development',
      debug_mode: false,
      currency_symbol: '€'
    }
  };
}
