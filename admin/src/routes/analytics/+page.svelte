<script>
  import { goto, invalidateAll } from '$app/navigation';
  import {
    DollarSign,
    ShoppingCart,
    Package,
    Layers,
    Users,
    Eye,
    Calendar,
    TrendingUp,
    Filter,
    ArrowUpRight,
    Award,
    RefreshCw
  } from 'lucide-svelte';

  let isRefreshing = false;
  async function handleRefresh() {
    isRefreshing = true;
    try {
      await invalidateAll();
    } finally {
      setTimeout(() => isRefreshing = false, 400);
    }
  }

  export let data;
  $: analytics = data.analytics || {
    summary: {},
    daily_points: [],
    top_categories: [],
    top_products: []
  };
  $: summary = analytics.summary || {};
  $: dailyPoints = analytics.daily_points || [];
  $: topCategories = analytics.top_categories || [];
  $: topProducts = analytics.top_products || [];

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
      goto('/analytics?filter=ytd');
    } else if (preset === 'month') {
      goto(`/analytics?filter=month&year=${filterYear}&month=${filterMonth}`);
    } else if (preset === 'year') {
      goto(`/analytics?filter=year&year=${filterYear}`);
    }
  }

  function applyCustom() {
    if (!customStart || !customEnd) return;
    goto(`/analytics?filter=custom&start_date=${customStart}&end_date=${customEnd}`);
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
  <title>Purchase Analysis & Sales Metrics | RustCraft Admin</title>
</svelte:head>

<div class="space-y-8 max-w-7xl mx-auto pb-12">
  <!-- Top Header & Filter Bar -->
  <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-4 bg-slate-900/60 p-6 rounded-3xl border border-slate-800 shadow-xl">
    <div>
      <div class="flex items-center gap-2.5">
        <div class="w-10 h-10 rounded-2xl bg-orange-500/10 text-orange-400 border border-orange-500/20 flex items-center justify-center">
          <TrendingUp size={22} />
        </div>
        <div>
          <h1 class="text-2xl font-black text-white tracking-tight">Purchase Analysis</h1>
          <p class="text-xs text-slate-400 mt-0.5">Comprehensive real-time financial KPIs, daily sales trajectories, and item leaderboards.</p>
        </div>
      </div>
    </div>

    <!-- Date Filters (YTD default, month, complete year, custom) -->
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

      <!-- Specific Month & Year Selectors (if Month or Year is selected) -->
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
      <div class="flex items-center gap-1.5">
        <input
          type="date"
          bind:value={customStart}
          class="px-2.5 py-1.5 rounded-xl bg-slate-900 border border-slate-800 text-white text-xs font-mono focus:outline-none focus:border-orange-500"
        />
        <span class="text-slate-500 text-xs">&rarr;</span>
        <input
          type="date"
          bind:value={customEnd}
          class="px-2.5 py-1.5 rounded-xl bg-slate-900 border border-slate-800 text-white text-xs font-mono focus:outline-none focus:border-orange-500"
        />
        <button
          type="button"
          on:click={applyCustom}
          class="px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-bold transition-colors"
        >
          Filter
        </button>
      </div>

      <!-- Live Refresh Button -->
      <button
        type="button"
        on:click={handleRefresh}
        class="px-3.5 py-1.5 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-700/80 text-slate-300 hover:text-white text-xs font-semibold flex items-center gap-1.5 transition-all shadow"
        title="Refresh Analytics with latest purchases"
      >
        <RefreshCw size={13} class={isRefreshing ? 'animate-spin text-orange-400' : 'text-slate-400'} />
        <span>{isRefreshing ? 'Refreshing...' : 'Refresh'}</span>
      </button>
    </div>
  </div>

  <!-- 6 Key Financial & Activity KPIs -->
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-4">
    <!-- Total Sales (Gross with Shipping) -->
    <div class="p-5 rounded-3xl bg-slate-900/60 border border-slate-800 shadow-xl relative overflow-hidden group hover:border-orange-500/40 transition-colors">
      <div class="flex items-center justify-between text-slate-400 mb-2">
        <span class="text-xs font-bold uppercase tracking-wider">Total Sales</span>
        <div class="p-2 rounded-xl bg-orange-500/10 text-orange-400 border border-orange-500/20">
          <DollarSign size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-white font-mono">
        {((summary.total_sales_cents || 0) / 100).toFixed(2)} €
      </div>
      <p class="text-[10px] text-slate-500 mt-1">Gross (incl. {(((summary.shipping_cost_cents || summary.shipping_cents || 0)) / 100).toFixed(2)} € shipping)</p>
    </div>

    <!-- Net Sales (without shipping) -->
    <div class="p-5 rounded-3xl bg-slate-900/60 border border-slate-800 shadow-xl relative overflow-hidden group hover:border-emerald-500/40 transition-colors">
      <div class="flex items-center justify-between text-slate-400 mb-2">
        <span class="text-xs font-bold uppercase tracking-wider text-emerald-400">Net Sales</span>
        <div class="p-2 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
          <DollarSign size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-emerald-400 font-mono">
        {((summary.net_sales_cents || 0) / 100).toFixed(2)} €
      </div>
      <p class="text-[10px] text-slate-500 mt-1">Pure merchandise sales (excl. shipping)</p>
    </div>

    <!-- Orders -->
    <div class="p-5 rounded-3xl bg-slate-900/60 border border-slate-800 shadow-xl relative overflow-hidden group hover:border-sky-500/40 transition-colors">
      <div class="flex items-center justify-between text-slate-400 mb-2">
        <span class="text-xs font-bold uppercase tracking-wider">Orders</span>
        <div class="p-2 rounded-xl bg-sky-500/10 text-sky-400 border border-sky-500/20">
          <ShoppingCart size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-white font-mono">
        {summary.orders_count || 0}
      </div>
      <p class="text-[10px] text-slate-500 mt-1">Confirmed customer checkouts</p>
    </div>

    <!-- Products Sold -->
    <div class="p-5 rounded-3xl bg-slate-900/60 border border-slate-800 shadow-xl relative overflow-hidden group hover:border-amber-500/40 transition-colors">
      <div class="flex items-center justify-between text-slate-400 mb-2">
        <span class="text-xs font-bold uppercase tracking-wider">Products Sold</span>
        <div class="p-2 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/20">
          <Package size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-white font-mono">
        {summary.products_sold || 0}
      </div>
      <p class="text-[10px] text-slate-500 mt-1">Physical & digital items dispatched</p>
    </div>

    <!-- Variations Sold -->
    <div class="p-5 rounded-3xl bg-slate-900/60 border border-slate-800 shadow-xl relative overflow-hidden group hover:border-purple-500/40 transition-colors">
      <div class="flex items-center justify-between text-slate-400 mb-2">
        <span class="text-xs font-bold uppercase tracking-wider">Variations Sold</span>
        <div class="p-2 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/20">
          <Layers size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-white font-mono">
        {summary.variations_sold || 0}
      </div>
      <p class="text-[10px] text-slate-500 mt-1">Unique SKUs / variant models</p>
    </div>

    <!-- Visitors & Views -->
    <div class="p-5 rounded-3xl bg-slate-900/60 border border-slate-800 shadow-xl relative overflow-hidden group hover:border-rose-500/40 transition-colors">
      <div class="flex items-center justify-between text-slate-400 mb-2">
        <span class="text-xs font-bold uppercase tracking-wider">Visitors & Views</span>
        <div class="p-2 rounded-xl bg-rose-500/10 text-rose-400 border border-rose-500/20">
          <Eye size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-white font-mono">
        {summary.visitors_count || summary.visitors || 0} <span class="text-xs text-slate-400 font-normal">/ {summary.views_count || summary.views || 0}</span>
      </div>
      <p class="text-[10px] text-slate-500 mt-1">Store traffic & page impressions</p>
    </div>
  </div>

  <!-- Interactive SVG Daily Trajectory Chart with Hover Tooltips -->
  <div class="p-6 sm:p-8 rounded-3xl bg-slate-900/60 border border-slate-800 shadow-xl">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
      <div>
        <h2 class="text-lg font-bold text-white tracking-tight flex items-center gap-2">
          <span>Daily Performance Trajectory</span>
          <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-orange-500/10 text-orange-400 border border-orange-500/20">
            Interactive Day Dots
          </span>
        </h2>
        <p class="text-xs text-slate-400 mt-0.5">Hover any day point to inspect daily net sales, gross revenue, and order volume.</p>
      </div>

      <!-- Metric Toggle -->
      <div class="inline-flex rounded-xl bg-slate-950 p-1 border border-slate-800 self-start sm:self-auto">
        <button
          type="button"
          on:click={() => chartMetric = 'net_sales'}
          class="px-3 py-1 rounded-lg text-xs font-bold transition-all {chartMetric === 'net_sales' ? 'bg-orange-600 text-white' : 'text-slate-400 hover:text-white'}"
        >
          Net Sales (€)
        </button>
        <button
          type="button"
          on:click={() => chartMetric = 'orders'}
          class="px-3 py-1 rounded-lg text-xs font-bold transition-all {chartMetric === 'orders' ? 'bg-orange-600 text-white' : 'text-slate-400 hover:text-white'}"
        >
          Orders Count
        </button>
      </div>
    </div>

    <!-- SVG Container -->
    <div class="relative w-full overflow-x-auto">
      <svg
        viewBox="0 0 {svgWidth} {svgHeight}"
        class="w-full h-64 sm:h-72 select-none overflow-visible"
      >
        <defs>
          <linearGradient id="chartGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#ea580c" stop-opacity="0.35" />
            <stop offset="100%" stop-color="#ea580c" stop-opacity="0.0" />
          </linearGradient>
        </defs>

        <!-- Horizontal Grid Lines & Y-axis labels -->
        {#each [0, 0.25, 0.5, 0.75, 1] as ratio}
          {@const y = svgHeight - padding.bottom - ratio * (svgHeight - padding.top - padding.bottom)}
          {@const val = (ratio * maxValue).toFixed(chartMetric === 'net_sales' ? 0 : 0)}
          <line
            x1={padding.left}
            y1={y}
            x2={svgWidth - padding.right}
            y2={y}
            stroke="#1e293b"
            stroke-dasharray="4,4"
            stroke-width="1"
          />
          <text
            x={padding.left - 10}
            y={y + 4}
            text-anchor="end"
            fill="#64748b"
            font-size="10"
            font-family="monospace"
          >
            {chartMetric === 'net_sales' ? `${val}€` : val}
          </text>
        {/each}

        <!-- Gradient Area Fill -->
        {#if areaPoints}
          <polygon points={areaPoints} fill="url(#chartGradient)" />
        {/if}

        <!-- Trajectory Polyline -->
        {#if polylinePoints}
          <polyline
            points={polylinePoints}
            fill="none"
            stroke="#ea580c"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        {/if}

        <!-- Interactive Connected Day Dots -->
        {#each pointsCoordinates as pt}
          <circle
            cx={pt.x}
            cy={pt.y}
            r="4.5"
            fill="#ea580c"
            stroke="#0f172a"
            stroke-width="2"
            class="cursor-pointer transition-transform hover:scale-150"
            on:mouseenter={(e) => {
              hoveredPoint = pt.data.raw;
              const rect = e.target.getBoundingClientRect();
              tooltipX = pt.x;
              tooltipY = pt.y - 12;
            }}
            on:mouseleave={() => {
              hoveredPoint = null;
            }}
          />
        {/each}

        <!-- X-axis Date Labels (Sampled) -->
        {#each pointsCoordinates.filter((_, idx) => idx % Math.max(1, Math.floor(pointsCoordinates.length / 7)) === 0) as pt}
          <text
            x={pt.x}
            y={svgHeight - 12}
            text-anchor="middle"
            fill="#64748b"
            font-size="10"
            font-family="monospace"
          >
            {pt.data.date.substring(5)}
          </text>
        {/each}
      </svg>

      <!-- Hover Tooltip Box -->
      {#if hoveredPoint}
        <div
          class="absolute z-30 pointer-events-none -translate-x-1/2 -translate-y-full mb-2 bg-slate-950 border border-slate-700/80 rounded-2xl p-3 shadow-2xl backdrop-blur-md min-w-[160px] animate-in fade-in zoom-in-95 duration-100"
          style="left: {(tooltipX / svgWidth) * 100}%; top: {(tooltipY / svgHeight) * 100}%;"
        >
          <div class="text-[11px] font-mono font-bold text-orange-400 mb-1 border-b border-slate-800 pb-1">
            📅 {hoveredPoint.date}
          </div>
          <div class="space-y-1 text-xs">
            <div class="flex justify-between items-center text-slate-300">
              <span>Net Sales:</span>
              <span class="font-mono font-bold text-emerald-400">
                {((hoveredPoint.net_sales_cents || 0) / 100).toFixed(2)} €
              </span>
            </div>
            <div class="flex justify-between items-center text-slate-300">
              <span>Total Gross:</span>
              <span class="font-mono font-bold text-white">
                {((hoveredPoint.total_sales_cents || 0) / 100).toFixed(2)} €
              </span>
            </div>
            <div class="flex justify-between items-center text-slate-300">
              <span>Orders:</span>
              <span class="font-mono font-bold text-sky-400">{hoveredPoint.orders_count || 0}</span>
            </div>
            <div class="flex justify-between items-center text-slate-300">
              <span>Items Sold:</span>
              <span class="font-mono font-bold text-amber-400">{hoveredPoint.products_sold || 0}</span>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Leaderboards: Top Categories & Top Products -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
    <!-- Top Categories Leaderboard -->
    <div class="p-6 rounded-3xl bg-slate-900/60 border border-slate-800 shadow-xl space-y-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Award size={18} class="text-orange-400" />
          <h3 class="text-base font-bold text-white">Top Categories</h3>
        </div>
        <span class="text-xs text-slate-500 font-mono">By gross volume</span>
      </div>

      {#if topCategories.length === 0}
        <div class="py-12 text-center text-slate-500 text-xs">
          No category purchases recorded for this period.
        </div>
      {:else}
        <div class="divide-y divide-slate-800/80">
          {#each topCategories as cat, idx}
            <div class="py-3 flex items-center justify-between text-xs">
              <div class="flex items-center gap-3">
                <span class="w-6 h-6 rounded-full bg-slate-800 flex items-center justify-center font-mono font-bold text-slate-400 text-[11px]">
                  {idx + 1}
                </span>
                <div>
                  <div class="font-bold text-white">{cat.category || cat.category_name}</div>
                  <div class="text-[10px] text-slate-400 font-mono">{cat.items_sold} items sold</div>
                </div>
              </div>
              <div class="text-right font-mono font-bold text-emerald-400">
                {(((cat.sales_cents || cat.gross_sales_cents || 0)) / 100).toFixed(2)} €
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Top Products Leaderboard -->
    <div class="p-6 rounded-3xl bg-slate-900/60 border border-slate-800 shadow-xl space-y-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Award size={18} class="text-orange-400" />
          <h3 class="text-base font-bold text-white">Top Products</h3>
        </div>
        <span class="text-xs text-slate-500 font-mono">By gross volume</span>
      </div>

      {#if topProducts.length === 0}
        <div class="py-12 text-center text-slate-500 text-xs">
          No product purchases recorded for this period.
        </div>
      {:else}
        <div class="divide-y divide-slate-800/80">
          {#each topProducts as prod, idx}
            <div class="py-3 flex items-center justify-between text-xs">
              <div class="flex items-center gap-3">
                <span class="w-6 h-6 rounded-full bg-slate-800 flex items-center justify-center font-mono font-bold text-slate-400 text-[11px]">
                  {idx + 1}
                </span>
                <div>
                  <div class="font-bold text-white max-w-[220px] truncate">{prod.title || prod.product_title}</div>
                  <div class="text-[10px] text-slate-400 font-mono">{prod.items_sold} items sold</div>
                </div>
              </div>
              <div class="text-right font-mono font-bold text-emerald-400">
                {(((prod.sales_cents || prod.gross_sales_cents || 0)) / 100).toFixed(2)} €
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
