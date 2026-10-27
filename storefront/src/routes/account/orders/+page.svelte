<script>
  import { onMount } from 'svelte';
  import { customer } from '$lib/stores/customer.js';
  import { goto } from '$app/navigation';
  import { 
    Box, ArrowLeft, ExternalLink, Clock, CheckCircle2, Truck, 
    ChevronDown, ChevronUp, Download, MapPin, CreditCard, 
    FileText, AlertCircle, PackageCheck, Layers
  } from 'lucide-svelte';

  let orders = [];
  let isLoading = true;
  let selectedOrderId = null;

  onMount(async () => {
    if (!$customer || !$customer.isLoggedIn) {
      goto('/account/login');
      return;
    }

    try {
      const res = await fetch('/api/v1/customer/orders', {
        headers: { Authorization: `Bearer ${$customer.token}` }
      });
      if (res.ok) {
        orders = await res.json();
        if (orders.length > 0) {
          selectedOrderId = orders[0].id;
        }
      }
    } catch (e) {
      console.error('Failed to load customer orders:', e);
    } finally {
      isLoading = false;
    }
  });

  function toggleOrder(id) {
    if (selectedOrderId === id) {
      selectedOrderId = null;
    } else {
      selectedOrderId = id;
    }
  }

  function formatPrice(cents) {
    return ((cents || 0) / 100).toFixed(2) + ' €';
  }

  function getStatusStep(status) {
    switch (status) {
      case 'pending': return 1;
      case 'processing': return 2;
      case 'shipped': return 3;
      case 'delivered': return 4;
      default: return 1;
    }
  }
</script>

<svelte:head>
  <title>My Orders | RustCraft</title>
</svelte:head>

<div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-10 space-y-6">
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800 pb-5">
    <div>
      <a href="/" class="inline-flex items-center gap-1.5 text-xs text-slate-400 hover:text-white mb-2 transition-colors">
        <ArrowLeft size={14} />
        <span>Back to Store</span>
      </a>
      <h1 class="text-2xl sm:text-3xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Box size={28} class="text-orange-500" />
        <span>Order History & Details</span>
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Select any order to inspect purchased products, fulfillment status, digital download files, and shipping addresses.
      </p>
    </div>

    <div class="flex items-center gap-3">
      <a 
        href="/account/downloads"
        class="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl bg-sky-950/60 hover:bg-sky-900/60 text-sky-300 border border-sky-800/60 text-xs font-semibold transition-colors shadow-sm"
      >
        <Download size={14} />
        <span>Digital Downloads Library</span>
      </a>
      <div class="text-xs text-slate-400 font-mono hidden sm:block">
        Logged in as: <strong class="text-white">{$customer?.email || 'Customer'}</strong>
      </div>
    </div>
  </div>

  {#if isLoading}
    <div class="text-center py-20 text-xs text-slate-400 flex flex-col items-center gap-3">
      <div class="w-7 h-7 border-2 border-orange-500 border-t-transparent rounded-full animate-spin"></div>
      <span>Loading your orders & shipments...</span>
    </div>
  {:else if orders.length === 0}
    <div class="p-12 text-center rounded-3xl bg-slate-900 border border-slate-800 text-slate-400 space-y-3 shadow-xl">
      <Box size={40} class="mx-auto text-slate-600" />
      <p class="text-base font-bold text-slate-200">No Orders Found</p>
      <p class="text-xs text-slate-500 max-w-md mx-auto">
        You haven't placed any purchases yet. Browse our hardware catalog and instant digital tools to get started.
      </p>
      <a href="/" class="inline-block mt-3 px-5 py-2.5 rounded-xl bg-orange-600 text-white text-xs font-bold hover:bg-orange-500 transition-colors shadow-lg shadow-orange-600/20">
        Explore Catalog
      </a>
    </div>
  {:else}
    <div class="space-y-4">
      {#each orders as o}
        {@const isSelected = selectedOrderId === o.id}
        {@const step = getStatusStep(o.order_status)}
        <div class="rounded-2xl bg-slate-900 border {isSelected ? 'border-orange-500/70 shadow-orange-500/5' : 'border-slate-800 hover:border-slate-700'} shadow-xl transition-all overflow-hidden">
          <!-- Order Header Bar (Clickable) -->
          <button
            type="button"
            on:click={() => toggleOrder(o.id)}
            class="w-full p-5 sm:p-6 text-left flex flex-col sm:flex-row sm:items-center justify-between gap-4 bg-slate-900 hover:bg-slate-850 transition-colors"
          >
            <div class="flex items-start sm:items-center gap-3.5">
              <div class="w-10 h-10 rounded-xl bg-slate-800 border border-slate-700 flex items-center justify-center text-orange-400 flex-shrink-0">
                <PackageCheck size={20} />
              </div>
              <div>
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="font-mono font-bold text-white text-sm sm:text-base">{o.order_number}</span>
                  <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider {o.order_status === 'shipped' || o.order_status === 'delivered' ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'}">
                    {o.order_status}
                  </span>
                  {#if o.items && o.items.some(i => i.is_digital)}
                    <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-sky-500/10 text-sky-400 border border-sky-500/20 flex items-center gap-1">
                      <span>⚡ Digital Content</span>
                    </span>
                  {/if}
                </div>
                <div class="text-xs text-slate-400 flex items-center gap-2 mt-1">
                  <span>Placed on {new Date(o.created_at).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}</span>
                  <span>&bull;</span>
                  <span>{o.items ? o.items.length : 0} {o.items && o.items.length === 1 ? 'product' : 'products'}</span>
                </div>
              </div>
            </div>

            <div class="flex items-center justify-between sm:justify-end gap-5">
              <div class="text-right">
                <div class="text-base font-black font-mono text-white">{formatPrice(o.total_cents)}</div>
                <div class="text-[10px] text-emerald-400 font-bold uppercase tracking-wider flex items-center gap-1 justify-end">
                  <CheckCircle2 size={10} />
                  <span>{o.payment_status} ({o.payment_provider})</span>
                </div>
              </div>

              <div class="w-8 h-8 rounded-lg bg-slate-800 flex items-center justify-center text-slate-400 transition-transform">
                {#if isSelected}
                  <ChevronUp size={16} />
                {:else}
                  <ChevronDown size={16} />
                {/if}
              </div>
            </div>
          </button>

          <!-- Expanded Order Details Drawer -->
          {#if isSelected}
            <div class="border-t border-slate-800 bg-slate-950/70 p-5 sm:p-6 space-y-6 animate-in fade-in duration-200">
              <!-- Step Fulfillment Progress Tracker -->
              <div class="p-4 rounded-xl bg-slate-900 border border-slate-800">
                <div class="text-[11px] font-bold text-slate-400 uppercase tracking-wider mb-3">Fulfillment Status Tracker</div>
                <div class="grid grid-cols-4 gap-2 text-center text-xs">
                  <div class="space-y-1.5">
                    <div class="w-7 h-7 mx-auto rounded-full flex items-center justify-center font-bold {step >= 1 ? 'bg-orange-500 text-white' : 'bg-slate-800 text-slate-500'}">1</div>
                    <div class="text-[11px] font-semibold {step >= 1 ? 'text-white' : 'text-slate-500'}">Order Placed</div>
                  </div>
                  <div class="space-y-1.5">
                    <div class="w-7 h-7 mx-auto rounded-full flex items-center justify-center font-bold {step >= 2 ? 'bg-orange-500 text-white' : 'bg-slate-800 text-slate-500'}">2</div>
                    <div class="text-[11px] font-semibold {step >= 2 ? 'text-white' : 'text-slate-500'}">Processing</div>
                  </div>
                  <div class="space-y-1.5">
                    <div class="w-7 h-7 mx-auto rounded-full flex items-center justify-center font-bold {step >= 3 ? 'bg-orange-500 text-white' : 'bg-slate-800 text-slate-500'}">3</div>
                    <div class="text-[11px] font-semibold {step >= 3 ? 'text-white' : 'text-slate-500'}">Shipped</div>
                  </div>
                  <div class="space-y-1.5">
                    <div class="w-7 h-7 mx-auto rounded-full flex items-center justify-center font-bold {step >= 4 ? 'bg-emerald-500 text-white' : 'bg-slate-800 text-slate-500'}">4</div>
                    <div class="text-[11px] font-semibold {step >= 4 ? 'text-emerald-400' : 'text-slate-500'}">Delivered</div>
                  </div>
                </div>

                {#if o.tracking_number}
                  <div class="mt-4 pt-3 border-t border-slate-800 flex flex-wrap items-center justify-between gap-2 text-xs">
                    <div class="flex items-center gap-2 text-sky-400 font-mono">
                      <Truck size={15} />
                      <span>Tracking Code: <strong>{o.tracking_number}</strong></span>
                    </div>
                    <a
                      href="/track?num={encodeURIComponent(o.order_number)}"
                      class="px-3 py-1.5 rounded-lg bg-sky-500/10 hover:bg-sky-500/20 text-sky-300 border border-sky-500/30 font-semibold text-xs inline-flex items-center gap-1.5 transition-colors"
                    >
                      <span>Track Shipment Live</span>
                      <ExternalLink size={12} />
                    </a>
                  </div>
                {/if}
              </div>

              <!-- Purchased Products List -->
              <div>
                <h3 class="text-xs font-bold text-slate-300 uppercase tracking-wider mb-3 flex items-center gap-2">
                  <Box size={14} class="text-orange-400" />
                  <span>Purchased Products & Items</span>
                </h3>

                <div class="rounded-xl bg-slate-900 border border-slate-800 overflow-hidden divide-y divide-slate-800">
                  {#if !o.items || o.items.length === 0}
                    <div class="p-4 text-xs text-slate-500 text-center">No item details recorded for this order.</div>
                  {:else}
                    {#each o.items as item}
                      <div class="p-4 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                        <div class="flex items-center gap-3 min-w-0">
                          <div class="w-11 h-11 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-center text-slate-500 font-bold text-xs flex-shrink-0">
                            {#if item.is_digital}
                              ⚡
                            {:else}
                              📦
                            {/if}
                          </div>
                          <div class="min-w-0">
                            <div class="font-bold text-white text-xs sm:text-sm truncate">{item.product_title}</div>
                            <div class="text-[11px] text-slate-400 flex items-center gap-2 flex-wrap">
                              <span>Version: {item.variant_title}</span>
                              <span>&bull;</span>
                              <span class="font-mono text-slate-500">SKU: {item.sku}</span>
                              <span>&bull;</span>
                              <span>Qty: <strong>{item.quantity}</strong> &times; {formatPrice(item.unit_price_cents)}</span>
                            </div>
                          </div>
                        </div>

                        <div class="flex items-center justify-between sm:justify-end gap-4 w-full sm:w-auto">
                          {#if item.is_digital}
                            {#if item.files && item.files.length > 0}
                              <div class="flex flex-wrap items-center gap-1.5">
                                {#each item.files as f}
                                  <a
                                    href={f.url}
                                    target="_blank"
                                    rel="noreferrer"
                                    download
                                    class="px-2.5 py-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 text-white font-bold text-[11px] flex items-center gap-1 shadow-sm transition-colors"
                                  >
                                    <Download size={12} />
                                    <span>{f.name}</span>
                                  </a>
                                {/each}
                              </div>
                            {:else if item.download_url}
                              <a
                                href={item.download_url}
                                target="_blank"
                                rel="noreferrer"
                                download
                                class="px-3 py-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 text-white font-bold text-xs flex items-center gap-1.5 shadow-sm transition-colors"
                              >
                                <Download size={13} />
                                <span>Download Asset</span>
                              </a>
                            {/if}
                          {/if}

                          <div class="font-mono font-bold text-white text-xs sm:text-sm flex-shrink-0">
                            {formatPrice(item.total_price_cents)}
                          </div>
                        </div>
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>

              <!-- Shipping Address & Financial Summary (2 Columns) -->
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <!-- Shipping Address Box -->
                <div class="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-2">
                  <h4 class="text-xs font-bold text-slate-300 uppercase tracking-wider flex items-center gap-2">
                    <MapPin size={14} class="text-orange-400" />
                    <span>Shipping Address</span>
                  </h4>

                  {#if o.shipping_address && (o.shipping_address.street_address || o.shipping_address.full_name)}
                    <div class="text-xs text-slate-300 space-y-0.5 pt-1">
                      <div class="font-bold text-white">{o.shipping_address.full_name || o.customer_name}</div>
                      <div>{o.shipping_address.street_address}</div>
                      {#if o.shipping_address.apartment_suite}
                        <div>{o.shipping_address.apartment_suite}</div>
                      {/if}
                      <div>{o.shipping_address.postal_code} {o.shipping_address.city}</div>
                      {#if o.shipping_address.state_province}
                        <div>{o.shipping_address.state_province}</div>
                      {/if}
                      <div class="text-slate-400 font-semibold">{o.shipping_address.country_code}</div>
                    </div>
                  {:else}
                    <div class="text-xs text-slate-400 italic pt-1">
                      {#if o.items && o.items.every(i => i.is_digital)}
                        <span>⚡ Digital Order — Delivered instantly online via customer email ({o.customer_email}). No physical address needed.</span>
                      {:else}
                        <span>Address recorded via customer account.</span>
                      {/if}
                    </div>
                  {/if}
                </div>

                <!-- Financial Calculation Breakdown -->
                <div class="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-2 text-xs">
                  <h4 class="text-xs font-bold text-slate-300 uppercase tracking-wider flex items-center gap-2">
                    <CreditCard size={14} class="text-orange-400" />
                    <span>Cost Breakdown</span>
                  </h4>

                  <div class="space-y-1.5 pt-1 text-slate-400">
                    <div class="flex justify-between">
                      <span>Subtotal</span>
                      <span class="font-mono text-slate-200">{formatPrice(o.subtotal_cents)}</span>
                    </div>
                    <div class="flex justify-between">
                      <span>Shipping</span>
                      {#if o.shipping_cost_cents === 0}
                        <span class="font-mono text-emerald-400 font-bold">0.00 € (Digital / Free)</span>
                      {:else}
                        <span class="font-mono text-slate-200">{formatPrice(o.shipping_cost_cents)}</span>
                      {/if}
                    </div>
                    <div class="flex justify-between">
                      <span>VAT / Taxes</span>
                      <span class="font-mono text-slate-200">{formatPrice(o.tax_cents)}</span>
                    </div>
                    <div class="border-t border-slate-800 pt-2 flex justify-between font-bold text-white text-sm">
                      <span>Total Billed</span>
                      <span class="font-mono text-orange-400">{formatPrice(o.total_cents)}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
