function buildMenuTree(flatItems) {
  if (!Array.isArray(flatItems) || flatItems.length === 0) return [];
  const map = new Map();
  const roots = [];

  flatItems.forEach(item => {
    map.set(item.id, { ...item, children: [] });
  });

  flatItems.forEach(item => {
    if (item.parent_id && map.has(item.parent_id)) {
      map.get(item.parent_id).children.push(map.get(item.id));
    } else {
      roots.push(map.get(item.id));
    }
  });

  map.forEach(item => {
    item.children.sort((a, b) => (a.sort_order || 0) - (b.sort_order || 0));
  });

  return roots.sort((a, b) => (a.sort_order || 0) - (b.sort_order || 0));
}

export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  let store = {};
  let paymentProviders = [];
  let headerMenu = [];
  let footerMenu = [];

  try {
    const res = await fetch(`${backendUrl}/api/v1/store/info`);
    if (res.ok) {
      const data = await res.json();
      store = data.store || {};
      paymentProviders = data.payment_providers || [];
    }
  } catch (e) {
    console.error('Failed to load store info from backend:', e);
  }

  try {
    const hRes = await fetch(`${backendUrl}/api/v1/menu?location=header`);
    if (hRes.ok) {
      const rawHeader = await hRes.json();
      headerMenu = buildMenuTree(rawHeader);
    }
  } catch (e) {
    console.error('Failed to load header menu:', e);
  }

  try {
    const fRes = await fetch(`${backendUrl}/api/v1/menu?location=footer`);
    if (fRes.ok) {
      const rawFooter = await fRes.json();
      footerMenu = buildMenuTree(rawFooter);
    }
  } catch (e) {
    console.error('Failed to load footer menu:', e);
  }

  const defaultHeader = [
    { id: '1', label: 'Catalog', url: '/' },
    { id: '2', label: 'Hardware', url: '/?category=Hardware' },
    { id: '3', label: 'Apparel', url: '/?category=Apparel' },
    { id: '4', label: 'Digital & Books', url: '/?category=Software+%26+Books' },
    { id: '5', label: 'Shipping Policy', url: '/policies/shipment-policy' },
    { id: '6', label: 'Track Order', url: '/track' }
  ];

  return {
    store: Object.keys(store).length > 0 ? store : {
      store_name: 'RustCraft Gear & Software',
      currency: 'EUR',
      currency_symbol: '€',
      tax_rate_percent: 19.0,
      debug_mode: true
    },
    paymentProviders,
    headerMenu: headerMenu.length > 0 ? headerMenu : defaultHeader,
    footerMenu: footerMenu.length > 0 ? footerMenu : [
      { id: 'f1', label: 'Shipping Policy', url: '/policies/shipment-policy' },
      { id: 'f2', label: 'Terms & Conditions', url: '/policies/terms-and-conditions' },
      { id: 'f3', label: 'Privacy Policy', url: '/policies/privacy-policy' },
      { id: 'f4', label: 'Legal Notice', url: '/policies/legal-notice' },
      { id: 'f5', label: 'Contact Us', url: '/policies/contact-information' }
    ],
    menuItems: headerMenu.length > 0 ? headerMenu : defaultHeader
  };
}

