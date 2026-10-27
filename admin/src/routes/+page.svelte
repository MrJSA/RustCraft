<script>
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { 
    DollarSign, ShoppingCart, TrendingUp, AlertTriangle, 
    Layers, ArrowUpRight, FileText, Printer, Eye,
    Users, Calendar, Filter, Award, Package, ChevronRight, BarChart3
  } from 'lucide-svelte';

  export let data;
  $: stats = data.stats || {};
  $: recentOrders = data.recentOrders || [];
  $: purchaseAnalysis = data.purchaseAnalysis || {
    summary: {},
    daily_points: [],
    top_categories: [],
    top_products: []
  };

  $: summary = purchaseAnalysis.summary || {};
  $: dailyPoints = purchaseAnalysis.daily_points || [];
  $: topCategories = purchaseAnalysis.top_categories || [];
  $: topProducts = purchaseAnalysis.top_products || [];

  // Tab State: 'overview' or 'analytics'
  let activeTab = 'overview';
  $: {
    const qTab = $page.url.searchParams.get('tab');
    if (qTab === 'analytics') {
      activeTab = 'analytics';
    } else {
      activeTab = 'overview';
    }
  }

  function switchTab(tab) {
    activeTab = tab;
    if (tab === 'analytics') {
      goto('/?tab=analytics', { keepFocus: true, noScroll: true });
    } else {
      goto('/', { keepFocus: true, noScroll: true });
    }
  }

  function formatPrice(cents) {
    return ((cents || 0) / 100).toFixed(2) + ' €';
  }

  // Analytics Filter state
  let activeFilter = data.filter || 'ytd';
  let filterYear = data.selectedYear || String(new Date().getFullYear());
  let filterMonth = data.selectedMonth || String(new Date().getMonth() + 1).padStart(2, '0');
  let customStart = data.startDate || '';
  let customEnd = data.endDate || '';

  // Chart Metric Toggle: 'net_sales' or 'orders'
  let chartMetric = 'net_sales';
  let hoveredPoint = null;
  let tooltipX = 0;
  let tooltipY = 0;

  function applyPreset(preset) {
    activeFilter = preset;
    if (preset === 'ytd') {
      goto(`/?tab=analytics&filter=ytd`);
    } else if (preset === 'month') {
      goto(`/?tab=analytics&filter=month&year=${filterYear}&month=${filterMonth}`);
    } else if (preset === 'year') {
      goto(`/?tab=analytics&filter=year&year=${filterYear}`);
    }
  }

  function applyCustom() {
    if (!customStart || !customEnd) return;
    goto(`/?tab=analytics&filter=custom&start_date=${customStart}&end_date=${customEnd}`);
  }

  // SVG Chart computation
  const svgWidth = 800;
  const svgHeight = 260;
  const padding = { top: 30, right: 30, bottom: 40, left: 60 };

  $: chartData = dailyPoints.map((p) => ({
    date: p.date,
    value: chartMetric === 'net_sales' ? (p.net_sales_cents || 0) / 100 : (p.orders_count || 0),
    raw: p
  }));

  $: maxValue = Math.max(...chartData.map((d) => d.value), chartMetric === 'net_sales' ? 100 : 5);
  $: pointsCoordinates = chartData.map((d, index) => {
    const totalPoints = Math.max(chartData.length - 1, 1);
    const x = padding.left + (index / totalPoints) * (svgWidth - padding.left - padding.right);
    const y = svgHeight - padding.bottom - (d.value / maxValue) * (svgHeight - padding.top - padding.bottom);
    return { x, y, data: d };
  });

  $: polylinePoints = pointsCoordinates.map((p) => `${p.x},${p.y}`).join(' ');
  $: areaPoints = pointsCoordinates.length > 0
    ? `${pointsCoordinates[0].x},${svgHeight - padding.bottom} ${polylinePoints} ${pointsCoordinates[pointsCoordinates.length - 1].x},${svgHeight - padding.bottom}`
    : '';

  const months = [
    { value: '01', label: 'January' },
    { value: '02', label: 'February' },
    { value: '03', label: 'March' },
    { value: '04', label: 'April' },
    { value: '05', label: 'May' },
    { value: '06', label: 'June' },
    { value: '07', label: 'July' },
    { value: '08', label: 'August' },
    { value: '09', label: 'September' },
    { value: '10', label: 'October' },
    { value: '11', label: 'November' },
    { value: '12', label: 'December' }
  ];

  const currentYearInt = new Date().getFullYear();
  const availableYears = [currentYearInt - 1, currentYearInt, currentYearInt + 1];
</script>

<svelte:head>
  <title>Overview & Analytics | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-7xl mx-auto pb-12">
  <!-- Top Consolidated Tab Navigation Bar -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800 pb-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <BarChart3 size={24} class="text-orange-500" />
        Overview & Analytics
      </h1>
      <p class="text-xs text-slate-400 mt-0.5">Unified dashboard for executive business metrics and deep-dive purchase analytics.</p>
    </div>

    <!-- On-Page Top Tabs -->
    <div class="inline-flex rounded-2xl bg-slate-900 p-1.5 border border-slate-800 shadow-md">
      <button
        type="button"
        on:click={() => switchTab('overview')}
        class="px-5 py-2 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'overview' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/30' : 'text-slate-400 hover:text-white'}"
      >
        <Layers size={14} />
        <span>Executive Overview</span>
      </button>

      <button
        type="button"
        on:click={() => switchTab('analytics')}
        class="px-5 py-2 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'analytics' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/30' : 'text-slate-400 hover:text-white'}"
      >
        <TrendingUp size={14} />
        <span>Purchase Analysis</span>
      </button>
    </div>
  </div>

  <!-- TAB 1: EXECUTIVE OVERVIEW -->
  {#if activeTab === 'overview'}
    <div class="space-y-8 animate-fadeIn">
      <!-- KPI Metric Cards -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
        <!-- 1. Gross Revenue -->
        <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="flex items-center justify-between text-slate-400 mb-3">
            <span class="text-xs font-semibold uppercase tracking-wider">Gross Sales</span>
            <div class="w-8 h-8 rounded-lg bg-emerald-500/10 text-emerald-400 flex items-center justify-center">
              <DollarSign size={16} />
            </div>
          </div>
          <div class="text-2xl font-black text-white font-mono">
            {formatPrice(stats.gross_sales_cents)}
          </div>
          <div class="mt-2 text-[11px] text-emerald-400 flex items-center gap-1">
            <ArrowUpRight size={13} />
            <span>ACID Settled Total</span>
          </div>
        </div>

        <!-- 2. Orders Count -->
        <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="flex items-center justify-between text-slate-400 mb-3">
            <span class="text-xs font-semibold uppercase tracking-wider">Orders</span>
            <div class="w-8 h-8 rounded-lg bg-sky-500/10 text-sky-400 flex items-center justify-center">
              <ShoppingCart size={16} />
            </div>
          </div>
          <div class="text-2xl font-black text-white font-mono">
            {stats.total_orders || 0}
          </div>
          <div class="mt-2 text-[11px] text-slate-400">
            {stats.processing_orders || 0} processing &bull; {stats.shipped_orders || 0} shipped
          </div>
        </div>

        <!-- 3. AOV -->
        <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="flex items-center justify-between text-slate-400 mb-3">
            <span class="text-xs font-semibold uppercase tracking-wider">Avg Order Value</span>
            <div class="w-8 h-8 rounded-lg bg-purple-500/10 text-purple-400 flex items-center justify-center">
              <TrendingUp size={16} />
            </div>
          </div>
          <div class="text-2xl font-black text-white font-mono">
            {formatPrice(stats.average_order_value_cents)}
          </div>
          <div class="mt-2 text-[11px] text-slate-400">
            Per fulfilled basket
          </div>
        </div>

        <!-- 4. Total SKUs -->
        <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="flex items-center justify-between text-slate-400 mb-3">
            <span class="text-xs font-semibold uppercase tracking-wider">Active SKUs</span>
            <div class="w-8 h-8 rounded-lg bg-orange-500/10 text-orange-400 flex items-center justify-center">
              <Layers size={16} />
            </div>
          </div>
          <div class="text-2xl font-black text-white font-mono">
            {stats.total_skus || 0}
          </div>
          <div class="mt-2 text-[11px] text-slate-400">
            Tracked in warehouse
          </div>
        </div>

        <!-- 5. Low Stock Alert -->
        <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="flex items-center justify-between text-slate-400 mb-3">
            <span class="text-xs font-semibold uppercase tracking-wider">Low Stock</span>
            <div class="w-8 h-8 rounded-lg {(stats.low_stock_skus || 0) > 0 ? 'bg-amber-500/10 text-amber-400' : 'bg-slate-800 text-slate-500'} flex items-center justify-center">
              <AlertTriangle size={16} />
            </div>
          </div>
          <div class="text-2xl font-black text-white font-mono {(stats.low_stock_skus || 0) > 0 ? 'text-amber-400' : 'text-slate-400'}">
            {stats.low_stock_skus || 0}
          </div>
          <div class="mt-2 text-[11px] {(stats.low_stock_skus || 0) > 0 ? 'text-amber-400 font-bold' : 'text-slate-500'}">
            {(stats.low_stock_skus || 0) > 0 ? 'Reorder needed!' : 'Inventory optimal'}
          </div>
        </div>
      </div>

      <!-- Quick Jump Banner to Purchase Analytics -->
      <div class="p-5 rounded-2xl bg-gradient-to-r from-orange-950/40 via-slate-900 to-slate-900 border border-orange-500/20 flex flex-col sm:flex-row items-center justify-between gap-4">
        <div class="flex items-center gap-3">
          <div class="p-2.5 rounded-xl bg-orange-600/20 text-orange-400 border border-orange-500/30">
            <TrendingUp size={20} />
          </div>
          <div>
            <h3 class="text-sm font-bold text-white">Looking for detailed Sales Trajectories & Item Breakdowns?</h3>
            <p class="text-xs text-slate-400 mt-0.5">Switch to the Purchase Analysis tab for Year-to-Date trends, customer acquisition, and category performance.</p>
          </div>
        </div>
        <button
          type="button"
          on:click={() => switchTab('analytics')}
          class="px-4 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs flex items-center gap-1.5 transition-all shadow-md flex-shrink-0"
        >
          <span>Open Purchase Analysis</span>
          <ChevronRight size={14} />
        </button>
      </div>

      <!-- Recent Orders Section -->
      <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
        <div class="flex items-center justify-between pb-3 border-b border-slate-800">
          <div>
            <h2 class="text-base font-bold text-white flex items-center gap-2">
              <ShoppingCart size={18} class="text-orange-500" />
              Recent Orders
            </h2>
            <p class="text-xs text-slate-400 mt-0.5">Recent customer checkout activity and fulfillment statuses.</p>
          </div>
          <a
            href="/orders"
            class="px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-xs font-bold text-slate-300 transition-colors"
          >
            View All Orders &rarr;
          </a>
        </div>

        {#if recentOrders.length === 0}
          <div class="py-12 text-center text-slate-500 text-xs">
            No recent orders recorded yet.
          </div>
        {:else}
          <div class="overflow-x-auto">
            <table class="w-full text-left text-xs">
              <thead class="text-slate-400 font-bold border-b border-slate-800 text-[11px] uppercase tracking-wider">
                <tr>
                  <th class="pb-3 px-3">Order #</th>
                  <th class="pb-3 px-3">Customer</th>
                  <th class="pb-3 px-3">Date</th>
                  <th class="pb-3 px-3">Total</th>
                  <th class="pb-3 px-3">Status</th>
                  <th class="pb-3 px-3 text-right">Documents</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-800/60 font-medium">
                {#each recentOrders as ord}
                  <tr class="hover:bg-slate-800/40 transition-colors">
                    <td class="py-3 px-3 font-mono font-bold text-white">
                      {ord.order_number}
                      {#if ord.is_test}<span class="ml-1 px-1.5 py-0.5 rounded text-[9px] font-black uppercase bg-amber-500/15 text-amber-300 border border-amber-500/30" title="Paid in test mode — not counted in analytics">Test</span>{/if}
                    </td>
                    <td class="py-3 px-3 text-slate-300">{ord.customer_name || ord.customer_email}</td>
                    <td class="py-3 px-3 text-slate-400">{new Date(ord.created_at).toLocaleDateString()}</td>
                    <td class="py-3 px-3 font-mono font-bold text-orange-400">{formatPrice(ord.total_cents)}</td>
                    <td class="py-3 px-3">
                      <span class="px-2 py-0.5 rounded-full text-[10px] font-bold capitalize {ord.order_status === 'completed' || ord.order_status === 'delivered' ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : ord.order_status === 'shipped' ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20' : 'bg-sky-500/10 text-sky-400 border border-sky-500/20'}">
                        {ord.order_status}
                      </span>
                    </td>
                    <td class="py-3 px-3 text-right space-x-2">
                      <a
                        href="/api/v1/admin/orders/{ord.id}/invoice"
                        target="_blank"
                        class="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white inline-flex items-center gap-1 text-[11px] font-semibold"
                        title="View PDF Invoice"
                      >
                        <FileText size={12} class="text-orange-400" />
                        <span>Invoice</span>
                      </a>
                      <a
                        href="/api/v1/admin/orders/{ord.id}/packing-slip"
                        target="_blank"
                        class="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white inline-flex items-center gap-1 text-[11px] font-semibold"
                        title="View PDF Packing Slip"
                      >
                        <Printer size={12} class="text-orange-400" />
                        <span>Slip</span>
                      </a>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    </div>

  <!-- TAB 2: PURCHASE ANALYSIS -->
  {:else if activeTab === 'analytics'}
    <div class="space-y-8 animate-fadeIn">
      <!-- Filter Bar -->
      <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-4 bg-slate-900/60 p-6 rounded-3xl border border-slate-800 shadow-xl">
        <div>
          <div class="flex items-center gap-2.5">
            <div class="w-10 h-10 rounded-2xl bg-orange-500/10 text-orange-400 border border-orange-500/20 flex items-center justify-center">
              <TrendingUp size={22} />
            </div>
            <div>
              <h2 class="text-xl font-black text-white tracking-tight">Purchase & Sales Analytics</h2>
              <p class="text-xs text-slate-400 mt-0.5">Real-time financial KPIs, daily sales trajectories, and item leaderboards.</p>
            </div>
          </div>
        </div>

        <!-- Date Filters -->
        <div class="flex flex-wrap items-center gap-2">
          <!-- Quick Presets -->
          <div class="inline-flex rounded-xl bg-slate-950 p-1 border border-slate-800">
            <button
              type="button"
              on:click={() => applyPreset('ytd')}
              class="px-3 py-1.5 rounded-lg text-xs font-bold transition-all {activeFilter === 'ytd' ? 'bg-orange-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
            >
              Year to Date (YTD)
            </button>
            <button
              type="button"
              on:click={() => applyPreset('month')}
              class="px-3 py-1.5 rounded-lg text-xs font-bold transition-all {activeFilter === 'month' ? 'bg-orange-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
            >
              Month
            </button>
            <button
              type="button"
              on:click={() => applyPreset('year')}
              class="px-3 py-1.5 rounded-lg text-xs font-bold transition-all {activeFilter === 'year' ? 'bg-orange-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
            >
              Complete Year
            </button>
          </div>

          <!-- Specific Month & Year Selectors -->
          {#if activeFilter === 'month'}
            <div class="flex items-center gap-1.5">
              <select
                bind:value={filterMonth}
                on:change={() => applyPreset('month')}
                class="px-3 py-1.5 rounded-xl bg-slate-900 border border-slate-700 text-white text-xs font-semibold focus:outline-none focus:border-orange-500"
              >
                {#each months as m}
                  <option value={m.value}>{m.label}</option>
                {/each}
              </select>
              <select
                bind:value={filterYear}
                on:change={() => applyPreset('month')}
                class="px-3 py-1.5 rounded-xl bg-slate-900 border border-slate-700 text-white text-xs font-semibold focus:outline-none focus:border-orange-500"
              >
                {#each availableYears as yr}
                  <option value={String(yr)}>{yr}</option>
                {/each}
              </select>
            </div>
          {:else if activeFilter === 'year'}
            <select
              bind:value={filterYear}
              on:change={() => applyPreset('year')}
              class="px-3 py-1.5 rounded-xl bg-slate-900 border border-slate-700 text-white text-xs font-semibold focus:outline-none focus:border-orange-500"
            >
              {#each availableYears as yr}
                <option value={String(yr)}>{yr}</option>
              {/each}
            </select>
          {/if}

          <!-- Custom Date Range -->
          <div class="flex items-center gap-1.5 bg-slate-950 px-3 py-1.5 rounded-xl border border-slate-800 text-xs">
            <span class="text-slate-500 text-[11px] font-semibold">Custom:</span>
            <input
              type="date"
              bind:value={customStart}
              class="bg-transparent text-white font-mono text-[11px] focus:outline-none"
            />
            <span class="text-slate-600">&rarr;</span>
            <input
              type="date"
              bind:value={customEnd}
              class="bg-transparent text-white font-mono text-[11px] focus:outline-none"
            />
            <button
              type="button"
              on:click={applyCustom}
              class="ml-1 px-2.5 py-0.5 rounded-lg bg-slate-800 hover:bg-orange-600 text-slate-300 hover:text-white font-bold transition-all text-[11px]"
            >
              Apply
            </button>
          </div>
        </div>
      </div>

      <!-- Financial KPI Metrics Grid -->
      <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-7 gap-4">
        <!-- 1. Total Gross Sales -->
        <div class="p-4 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="text-[11px] text-slate-400 font-semibold uppercase tracking-wider mb-1 flex items-center justify-between">
            <span>Gross Sales</span>
            <DollarSign size={13} class="text-emerald-400" />
          </div>
          <div class="text-lg sm:text-xl font-mono font-black text-white">
            {formatPrice(summary.total_sales_cents)}
          </div>
        </div>

        <!-- 2. Net Sales -->
        <div class="p-4 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="text-[11px] text-slate-400 font-semibold uppercase tracking-wider mb-1 flex items-center justify-between">
            <span>Net Sales</span>
            <TrendingUp size={13} class="text-orange-400" />
          </div>
          <div class="text-lg sm:text-xl font-mono font-black text-orange-400">
            {formatPrice(summary.net_sales_cents)}
          </div>
        </div>

        <!-- 3. Shipping -->
        <div class="p-4 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="text-[11px] text-slate-400 font-semibold uppercase tracking-wider mb-1 flex items-center justify-between">
            <span>Shipping</span>
            <span class="text-slate-500 font-mono text-[10px]">DHL</span>
          </div>
          <div class="text-lg sm:text-xl font-mono font-black text-white">
            {formatPrice(summary.shipping_cents)}
          </div>
        </div>

        <!-- 4. Orders Placed -->
        <div class="p-4 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="text-[11px] text-slate-400 font-semibold uppercase tracking-wider mb-1 flex items-center justify-between">
            <span>Orders</span>
            <ShoppingCart size={13} class="text-sky-400" />
          </div>
          <div class="text-lg sm:text-xl font-mono font-black text-white">
            {summary.orders_count || 0}
          </div>
        </div>

        <!-- 5. Products Sold -->
        <div class="p-4 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="text-[11px] text-slate-400 font-semibold uppercase tracking-wider mb-1 flex items-center justify-between">
            <span>Products Sold</span>
            <Package size={13} class="text-purple-400" />
          </div>
          <div class="text-lg sm:text-xl font-mono font-black text-white">
            {summary.products_sold || 0}
          </div>
        </div>

        <!-- 6. Variations Sold -->
        <div class="p-4 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
          <div class="text-[11px] text-slate-400 font-semibold uppercase tracking-wider mb-1 flex items-center justify-between">
            <span>Variations</span>
            <Layers size={13} class="text-pink-400" />
          </div>
          <div class="text-lg sm:text-xl font-mono font-black text-white">
            {summary.variations_sold || 0}
          </div>
        </div>

        <!-- 7. Visitors & Traffic -->
        <div class="p-4 rounded-2xl bg-slate-900 border border-slate-800 shadow-md col-span-2 sm:col-span-1">
          <div class="text-[11px] text-slate-400 font-semibold uppercase tracking-wider mb-1 flex items-center justify-between">
            <span>Traffic</span>
            <Users size={13} class="text-amber-400" />
          </div>
          <div class="text-lg sm:text-xl font-mono font-black text-white">
            {summary.visitors || 0}
          </div>
          <div class="text-[10px] text-slate-500 font-mono mt-0.5">
            {summary.views || 0} Views
          </div>
        </div>
      </div>

      <!-- Interactive SVG Chart -->
      <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div>
            <h3 class="text-base font-bold text-white flex items-center gap-2">
              <TrendingUp size={18} class="text-orange-500" />
              Sales & Orders Trajectory
            </h3>
            <p class="text-xs text-slate-400 mt-0.5">
              Daily aggregates over the selected period. Hover data points to inspect numbers.
            </p>
          </div>

          <!-- Chart Metric Toggle -->
          <div class="inline-flex rounded-xl bg-slate-950 p-1 border border-slate-800 self-start sm:self-auto">
            <button
              type="button"
              on:click={() => (chartMetric = 'net_sales')}
              class="px-3 py-1.5 rounded-lg text-xs font-bold transition-all {chartMetric === 'net_sales' ? 'bg-orange-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
            >
              Net Sales (€)
            </button>
            <button
              type="button"
              on:click={() => (chartMetric = 'orders')}
              class="px-3 py-1.5 rounded-lg text-xs font-bold transition-all {chartMetric === 'orders' ? 'bg-orange-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
            >
              Orders Count
            </button>
          </div>
        </div>

        {#if dailyPoints.length === 0}
          <div class="h-64 flex items-center justify-center text-slate-500 text-xs">
            No sales data recorded in this timeframe.
          </div>
        {:else}
          <div class="relative w-full overflow-x-auto">
            <svg
              viewBox={`0 0 ${svgWidth} ${svgHeight}`}
              class="w-full h-64 overflow-visible"
              preserveAspectRatio="none"
            >
              <defs>
                <linearGradient id="areaGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="#ea580c" stop-opacity="0.35" />
                  <stop offset="100%" stop-color="#ea580c" stop-opacity="0.0" />
                </linearGradient>
              </defs>

              <!-- Gridlines -->
              {#each [0, 0.25, 0.5, 0.75, 1] as tick}
                {@const y = svgHeight - padding.bottom - tick * (svgHeight - padding.top - padding.bottom)}
                <line
                  x1={padding.left}
                  y1={y}
                  x2={svgWidth - padding.right}
                  y2={y}
                  stroke="#334155"
                  stroke-dasharray="3 3"
                  stroke-opacity="0.5"
                />
                <text
                  x={padding.left - 8}
                  y={y + 4}
                  text-anchor="end"
                  class="fill-slate-500 text-[10px] font-mono"
                >
                  {chartMetric === 'net_sales' ? `${Math.round(tick * maxValue)} €` : Math.round(tick * maxValue)}
                </text>
              {/each}

              <!-- Shaded Area -->
              {#if areaPoints}
                <polygon points={areaPoints} fill="url(#areaGradient)" />
              {/if}

              <!-- Polyline -->
              {#if polylinePoints}
                <polyline
                  points={polylinePoints}
                  fill="none"
                  stroke="#f97316"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              {/if}

              <!-- Data Points with hover -->
              {#each pointsCoordinates as pt}
                <circle
                  cx={pt.x}
                  cy={pt.y}
                  r="4.5"
                  class="fill-orange-500 stroke-slate-900 stroke-2 hover:r-6 hover:fill-white cursor-pointer transition-all"
                  on:mouseenter={(e) => {
                    hoveredPoint = pt.data;
                    const rect = e.target.getBoundingClientRect();
                    tooltipX = pt.x;
                    tooltipY = pt.y;
                  }}
                  on:mouseleave={() => (hoveredPoint = null)}
                />
              {/each}
            </svg>

            <!-- Tooltip -->
            {#if hoveredPoint}
              <div
                class="absolute z-20 pointer-events-none -translate-x-1/2 -translate-y-full mb-3 px-3 py-2 rounded-xl bg-slate-950 border border-slate-700 shadow-2xl text-xs"
                style="left: {(tooltipX / svgWidth) * 100}%; top: {(tooltipY / svgHeight) * 100}%;"
              >
                <div class="font-bold text-white text-[11px] mb-0.5">{hoveredPoint.date}</div>
                <div class="text-orange-400 font-mono font-bold">
                  {chartMetric === 'net_sales' ? `${hoveredPoint.value.toFixed(2)} €` : `${hoveredPoint.value} Orders`}
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Bottom Grids: Top Categories & Top Products -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <!-- Top Categories -->
        <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <div class="flex items-center justify-between pb-3 border-b border-slate-800">
            <h3 class="text-base font-bold text-white flex items-center gap-2">
              <Award size={18} class="text-orange-500" />
              Top Categories by Revenue
            </h3>
          </div>

          {#if topCategories.length === 0}
            <div class="py-8 text-center text-slate-500 text-xs">No category data recorded yet.</div>
          {:else}
            <div class="space-y-3">
              {#each topCategories as cat}
                {@const percent = summary.net_sales_cents > 0 ? (cat.sales_cents / summary.net_sales_cents) * 100 : 0}
                <div class="space-y-1">
                  <div class="flex justify-between text-xs">
                    <span class="font-bold text-white">{cat.category}</span>
                    <span class="font-mono text-orange-400 font-bold">{formatPrice(cat.sales_cents)}</span>
                  </div>
                  <div class="h-2 rounded-full bg-slate-950 overflow-hidden">
                    <div class="h-full bg-orange-600 rounded-full" style="width: {percent}%"></div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Top Products Leaderboard -->
        <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <div class="flex items-center justify-between pb-3 border-b border-slate-800">
            <h3 class="text-base font-bold text-white flex items-center gap-2">
              <Package size={18} class="text-orange-500" />
              Best-Selling Products
            </h3>
          </div>

          {#if topProducts.length === 0}
            <div class="py-8 text-center text-slate-500 text-xs">No products sold in this period.</div>
          {:else}
            <div class="overflow-x-auto">
              <table class="w-full text-left text-xs">
                <thead class="text-slate-400 font-bold border-b border-slate-800 text-[11px] uppercase tracking-wider">
                  <tr>
                    <th class="pb-2">Product Title</th>
                    <th class="pb-2 text-center">Units Sold</th>
                    <th class="pb-2 text-right">Revenue</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-800/60 font-medium">
                  {#each topProducts as prod}
                    <tr class="hover:bg-slate-800/40 transition-colors">
                      <td class="py-2.5 text-white font-bold">{prod.title}</td>
                      <td class="py-2.5 text-center font-mono text-slate-300">{prod.items_sold}</td>
                      <td class="py-2.5 text-right font-mono font-bold text-orange-400">{formatPrice(prod.gross_cents)}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .animate-fadeIn {
    animation: fadeIn 0.15s ease-out forwards;
  }
</style>
