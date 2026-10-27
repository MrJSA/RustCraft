<script>
  import { Search, Box, CheckCircle2, Clock, Truck, AlertCircle } from 'lucide-svelte';

  let orderNumber = '';
  let email = '';
  let isLoading = false;
  let orderData = null;
  let errorMessage = '';

  async function handleLookup() {
    if (!orderNumber.trim() || !email.trim()) return;
    isLoading = true;
    errorMessage = '';
    orderData = null;

    try {
      const res = await fetch(
        `/api/v1/orders/lookup/${encodeURIComponent(orderNumber.trim())}?email=${encodeURIComponent(email.trim())}`
      );
      if (!res.ok) {
        throw new Error('Order not found. Please check the order number and the email address used for the order.');
      }
      orderData = await res.json();
    } catch (e) {
      errorMessage = e.message;
    } finally {
      isLoading = false;
    }
  }
</script>

<svelte:head>
  <title>Track Order</title>
</svelte:head>

<div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
  <div class="text-center mb-8">
    <div class="w-12 h-12 rounded-2xl bg-orange-500/10 border border-orange-500/20 text-orange-400 flex items-center justify-center mx-auto mb-3">
      <Box size={24} />
    </div>
    <h1 class="text-3xl font-extrabold text-white tracking-tight">Track Your Shipment</h1>
    <p class="text-xs text-slate-400 mt-2">Enter your order number and the email address you used at checkout.</p>
  </div>

  <!-- Search form -->
  <form on:submit|preventDefault={handleLookup} class="flex flex-col sm:flex-row gap-2 max-w-2xl mx-auto mb-10">
    <input
      type="text"
      aria-label="Order number"
      bind:value={orderNumber}
      placeholder="e.g. ORD-20260923-..."
      required
      class="flex-1 px-4 py-3 rounded-xl bg-slate-900 border border-slate-800 text-white placeholder-slate-500 text-sm focus:outline-none focus:border-orange-500"
    />
    <input
      type="email"
      aria-label="Email address used for the order"
      autocomplete="email"
      bind:value={email}
      placeholder="Email used for the order"
      required
      class="flex-1 px-4 py-3 rounded-xl bg-slate-900 border border-slate-800 text-white placeholder-slate-500 text-sm focus:outline-none focus:border-orange-500"
    />
    <button
      type="submit"
      disabled={isLoading}
      class="px-6 py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-md transition-all disabled:opacity-50"
    >
      {isLoading ? 'Searching...' : 'Track'}
    </button>
  </form>

  {#if errorMessage}
    <div class="p-4 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs flex items-center gap-3">
      <AlertCircle size={18} class="text-rose-400" />
      <span>{errorMessage}</span>
    </div>
  {/if}

  {#if orderData}
    <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
      <div class="flex items-center justify-between border-b border-slate-800 pb-4">
        <div>
          <div class="text-xs text-slate-400">Order Reference</div>
          <div class="text-base font-mono font-bold text-white">{orderData.order.order_number}</div>
        </div>
        <div class="text-right">
          <span class="px-2.5 py-1 rounded-md text-xs font-bold uppercase bg-orange-500/10 border border-orange-500/20 text-orange-400">
            {orderData.order.order_status}
          </span>
        </div>
      </div>

      <!-- Tracking Number if present -->
      {#if orderData.order.tracking_number}
        <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-between text-xs">
          <div class="flex items-center gap-2 text-slate-300">
            <Truck size={16} class="text-orange-400" />
            <span>Carrier Tracking: <strong class="font-mono text-white">{orderData.order.tracking_number}</strong></span>
          </div>
        </div>
      {/if}

      <!-- Items in this order -->
      <div>
        <h3 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Items in Shipment</h3>
        <div class="divide-y divide-slate-800">
          {#each orderData.items as item}
            <div class="py-2.5 flex items-center justify-between text-xs">
              <div>
                <div class="font-bold text-white">{item.product_title}</div>
                <div class="text-slate-400">{item.variant_title} (SKU: {item.sku}) &times; {item.quantity}</div>
              </div>
              <div class="font-mono font-bold text-white">{((item.unit_price_cents * item.quantity) / 100).toFixed(2)} €</div>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>
