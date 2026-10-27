export async function load({ fetch, url }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = {};

  const filter = url.searchParams.get('filter') || 'ytd';
  const startDate = url.searchParams.get('start_date') || '';
  const endDate = url.searchParams.get('end_date') || '';
  const selectedYear = url.searchParams.get('year') || String(new Date().getFullYear());
  const selectedMonth = url.searchParams.get('month') || String(new Date().getMonth() + 1).padStart(2, '0');

  let queryUrl = `${backendUrl}/api/v1/admin/analytics/purchase-analysis?preset=${filter}&`;

  if (filter === 'month' && selectedYear && selectedMonth) {
    const y = parseInt(selectedYear);
    const m = parseInt(selectedMonth);
    const start = `${selectedYear}-${selectedMonth.padStart(2, '0')}-01`;
    const lastDay = new Date(y, m, 0).getDate();
    const end = `${selectedYear}-${selectedMonth.padStart(2, '0')}-${String(lastDay).padStart(2, '0')}`;
    queryUrl += `year=${selectedYear}&month=${selectedMonth}&start_date=${start}&end_date=${end}`;
  } else if (filter === 'year' && selectedYear) {
    queryUrl += `year=${selectedYear}&start_date=${selectedYear}-01-01&end_date=${selectedYear}-12-31`;
  } else if (filter === 'custom' && startDate && endDate) {
    queryUrl += `start_date=${startDate}&end_date=${endDate}`;
  } else {
    const now = new Date();
    const start = `${now.getFullYear()}-01-01`;
    const end = now.toISOString().split('T')[0];
    queryUrl += `start_date=${start}&end_date=${end}`;
  }

  let analytics = {
    summary: {
      total_sales_cents: 0,
      net_sales_cents: 0,
      shipping_cents: 0,
      orders_count: 0,
      products_sold: 0,
      variations_sold: 0,
      visitors: 0,
      views: 0
    },
    daily_points: [],
    top_categories: [],
    top_products: []
  };

  try {
    const res = await fetch(queryUrl, { headers });
    if (res.ok) {
      analytics = await res.json();
    }
  } catch (e) {
    console.error('Failed to load purchase analysis data:', e);
  }

  return {
    analytics,
    filter,
    startDate,
    endDate,
    selectedYear,
    selectedMonth
  };
}
