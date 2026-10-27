<script>
  import { CheckCircle2, Download, Package, ArrowRight, ShieldCheck, ExternalLink, CreditCard, Mail, FileText } from 'lucide-svelte';

  export let data;
  $: order = data.order || {};
  $: items = data.items || [];
  $: digitalItems = items.filter((i) => i.is_digital);
  $: physicalItems = items.filter((i) => !i.is_digital);
  $: isDigitalOnly = items.length > 0 && physicalItems.length === 0;
</script>

<svelte:head>
  <title>Order Confirmed | {order.order_number}</title>
</svelte:head>

<div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
  <!-- Success Banner -->
  <div class="text-center mb-10">
    <div class="w-16 h-16 rounded-full bg-emerald-500/20 border border-emerald-500/30 text-emerald-400 flex items-center justify-center mx-auto mb-4 shadow-xl shadow-emerald-500/10">
      <CheckCircle2 size={36} />
    </div>

    <span class="px-3 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-bold uppercase tracking-wider">
      Payment Authorized & Confirmed
    </span>

    <h1 class="text-3xl sm:text-4xl font-black text-white tracking-tight mt-3">
      Thank You for Your Order!
    </h1>

    <p class="text-sm text-slate-400 mt-2">
      Order <span class="text-white font-mono font-bold">#{order.order_number}</span> has been processed with strict ACID lock integrity.
    </p>
  </div>

  <!-- Dedicated Payment Confirmation & Settlement Card -->
  <div class="mb-8 p-6 rounded-2xl bg-slate-900 border border-emerald-500/30 shadow-2xl space-y-4">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800 pb-4">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-emerald-500/15 border border-emerald-500/30 text-emerald-400 flex items-center justify-center">
          <CreditCard size={20} />
        </div>
        <div>
          <div class="text-xs font-bold uppercase tracking-wider text-slate-400">Payment Status</div>
          <div class="flex items-center gap-2 mt-0.5">
            <span class="text-emerald-400 font-black text-base uppercase">PAID & SETTLED</span>
            <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 flex items-center gap-1">
              <CheckCircle2 size={10} /> Succeeded
            </span>
          </div>
        </div>
      </div>

      <div class="text-left sm:text-right">
        <div class="text-xs font-bold uppercase tracking-wider text-slate-400">Total Charged</div>
        <div class="text-2xl font-mono font-black text-white mt-0.5">
          {((order.total_cents || 0) / 100).toFixed(2)} €
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs">
      <div class="p-3 rounded-xl bg-slate-950/80 border border-slate-800/80">
        <div class="text-slate-400 font-semibold mb-0.5">Payment Method:</div>
        <div class="text-white font-bold capitalize">
          {order.payment_provider ? order.payment_provider.replace('_', ' ') : 'Stripe Credit Card'}
        </div>
      </div>
      <div class="p-3 rounded-xl bg-slate-950/80 border border-slate-800/80">
        <div class="text-slate-400 font-semibold mb-0.5">Receipt Reference:</div>
        <div class="text-white font-mono font-bold">{order.order_number}</div>
      </div>
      <div class="p-3 rounded-xl bg-slate-950/80 border border-slate-800/80">
        <div class="text-slate-400 font-semibold mb-0.5">Order Fulfillment:</div>
        <div class="font-bold {isDigitalOnly ? 'text-emerald-400' : 'text-amber-400'}">
          {isDigitalOnly ? 'Instant Digital Access (Completed)' : (order.order_status || 'Processing Shipment')}
        </div>
      </div>
    </div>

    {#if order.discount_cents && order.discount_cents > 0}
      <div class="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-300 text-xs flex items-center justify-between">
        <span class="font-semibold">Promo Discount Applied ({order.coupon_code || 'PROMO'}):</span>
        <span class="font-mono font-bold">-{(order.discount_cents / 100).toFixed(2)} €</span>
      </div>
    {/if}

    <div class="flex items-center gap-2 pt-2 text-xs text-slate-400 border-t border-slate-800/60">
      <Mail size={14} class="text-emerald-400 flex-shrink-0" />
      <span>An official payment confirmation receipt has been dispatched to <strong class="text-white">{order.customer_email}</strong>.</span>
    </div>
  </div>

  <!-- Digital Downloads Box (If any) -->
  {#if digitalItems.length > 0}
    <div class="mb-8 p-6 rounded-2xl bg-gradient-to-r from-sky-950/80 to-blue-950/80 border border-sky-800/80 shadow-2xl space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <Download size={24} class="text-sky-400 flex-shrink-0" />
          <div>
            <h2 class="text-base font-bold text-white flex items-center gap-2">
              <span>Your Digital Downloads are Ready!</span>
              {#if isDigitalOnly}
                <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">Auto-Completed</span>
              {/if}
            </h2>
            <p class="text-xs text-sky-200">Instant digital delivery — no physical shipping needed. Re-download any time from your account.</p>
          </div>
        </div>

        <a
          href="/account/downloads"
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-sky-500/20 hover:bg-sky-500/30 text-sky-200 border border-sky-500/40 text-xs font-semibold transition-colors self-start sm:self-auto"
        >
          <span>My Downloads Hub</span>
          <ExternalLink size={12} />
        </a>
      </div>

      <div class="space-y-3">
        {#each digitalItems as item}
          <div class="p-4 rounded-xl bg-slate-900/90 border border-sky-800/40 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
            <div>
              <div class="text-sm font-bold text-white">{item.product_title}</div>
              <div class="text-xs text-slate-400">{item.variant_title} &bull; SKU: {item.sku}</div>
            </div>

            <div class="flex flex-wrap items-center gap-2">
              {#if item.files && item.files.length > 0}
                {#each item.files as f}
                  <a
                    href={f.url}
                    target="_blank"
                    rel="noreferrer"
                    download
                    class="px-3 py-2 rounded-xl bg-sky-500 hover:bg-sky-400 text-white text-xs font-bold flex items-center gap-1.5 shadow-md shadow-sky-500/20 transition-all"
                  >
                    <Download size={13} />
                    <span>{f.name}</span>
                  </a>
                {/each}
              {:else if item.download_url}
                <a
                  href={item.download_url}
                  target="_blank"
                  rel="noreferrer"
                  download
                  class="px-4 py-2.5 rounded-xl bg-sky-500 hover:bg-sky-400 text-white text-xs font-bold flex items-center gap-2 shadow-lg shadow-sky-500/20 transition-all flex-shrink-0"
                >
                  <Download size={14} />
                  <span>Download Asset</span>
                </a>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Order Details Card -->
  <div class="p-6 sm:p-8 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
    <div class="flex flex-wrap items-center justify-between gap-4 border-b border-slate-800 pb-5">
      <div>
        <div class="text-xs text-slate-400 uppercase font-semibold">Order Reference</div>
        <div class="text-lg font-mono font-bold text-white">{order.order_number}</div>
      </div>
      <div>
        <div class="text-xs text-slate-400 uppercase font-semibold">Order Status</div>
        <div class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-md text-xs font-bold uppercase {isDigitalOnly ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400' : 'bg-amber-500/10 border-amber-500/20 text-amber-400'} mt-1">
          {isDigitalOnly ? 'Completed' : (order.order_status || 'Processing')}
        </div>
      </div>
      <div>
        <div class="text-xs text-slate-400 uppercase font-semibold">Customer</div>
        <div class="text-sm font-bold text-white">{order.customer_name}</div>
      </div>
    </div>

    <!-- Items List -->
    <div>
      <h3 class="text-xs font-bold text-slate-300 uppercase tracking-wider mb-3">Item Breakdown</h3>
      <div class="divide-y divide-slate-800/80">
        {#each items as item}
          <div class="py-3 flex items-center justify-between text-xs">
            <div>
              <div class="font-bold text-white">{item.product_title}</div>
              <div class="text-slate-400">{item.variant_title} &times; {item.quantity} (SKU: {item.sku})</div>
            </div>
            <div class="font-mono font-bold text-white">
              {((item.unit_price_cents * item.quantity) / 100).toFixed(2)} €
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Actions -->
    <div class="pt-6 border-t border-slate-800 flex items-center justify-between gap-4">
      <a
        href="/"
        class="w-full sm:w-auto px-6 py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md flex items-center justify-center gap-2"
      >
        <span>Continue Shopping</span>
        <ArrowRight size={14} />
      </a>

      <a
        href="/track"
        class="text-xs text-slate-400 hover:text-white flex items-center gap-1.5 transition-colors"
      >
        <span>Track Shipping Progress</span>
        <ArrowRight size={13} class="text-orange-400" />
      </a>
    </div>
  </div>
</div>

