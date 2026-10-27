export async function load({ fetch, url }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = {};

  const tab = url.searchParams.get('tab') || 'overview';
  const filter = url.searchParams.get('filter') || 'ytd';
  const startDate = url.searchParams.get('start_date') || '';
  const endDate = url.searchParams.get('end_date') || '';
  const selectedYear = url.searchParams.get('year') || String(new Date().getFullYear());
  const selectedMonth = url.searchParams.get('month') || String(new Date().getMonth() + 1).padStart(2, '0');

  let queryUrl = `${backendUrl}/api/v1/admin/analytics/purchase-analysis?`;
  if (filter === 'month' && selectedYear && selectedMonth) {
    const y = parseInt(selectedYear);
    const m = parseInt(selectedMonth);
    const start = `${selectedYear}-${selectedMonth.padStart(2, '0')}-01`;
    const lastDay = new Date(y, m, 0).getDate();
    const end = `${selectedYear}-${selectedMonth.padStart(2, '0')}-${String(lastDay).padStart(2, '0')}`;
    queryUrl += `start_date=${start}&end_date=${end}`;
  } else if (filter === 'year' && selectedYear) {
    queryUrl += `start_date=${selectedYear}-01-01&end_date=${selectedYear}-12-31`;
  } else if (filter === 'custom' && startDate && endDate) {
    queryUrl += `start_date=${startDate}&end_date=${endDate}`;
  } else {
    const now = new Date();
    const start = `${now.getFullYear()}-01-01`;
    const end = now.toISOString().split('T')[0];
    queryUrl += `start_date=${start}&end_date=${end}`;
  }

  let stats = {
    gross_sales_cents: 0,
    total_orders: 0,
    average_order_value_cents: 0,
    total_skus: 0,
    low_stock_skus: 0
  };
  let analytics = [];
  let recentOrders = [];
  let purchaseAnalysis = {
    summary: {},
    daily_points: [],
    top_categories: [],
    top_products: []
  };

  try {
    const [statsRes, analyticsRes, ordersRes, purchaseRes] = await Promise.all([
      fetch(`${backendUrl}/api/v1/admin/dashboard/stats`, { headers }),
      fetch(`${backendUrl}/api/v1/admin/dashboard/sales-analytics`, { headers }),
      fetch(`${backendUrl}/api/v1/admin/orders`, { headers }),
      fetch(queryUrl, { headers })
    ]);

    if (statsRes.ok) stats = await statsRes.json();
    if (analyticsRes.ok) analytics = await analyticsRes.json();
    if (ordersRes.ok) {
      const orders = await ordersRes.json();
      recentOrders = orders.slice(0, 8);
    }
    if (purchaseRes.ok) purchaseAnalysis = await purchaseRes.json();
  } catch (e) {
    console.error('Failed to load combined dashboard & analytics data:', e);
  }

  return {
    tab,
    stats,
    analytics,
    recentOrders,
    purchaseAnalysis,
    filter,
    startDate,
    endDate,
    selectedYear,
    selectedMonth
  };
}
