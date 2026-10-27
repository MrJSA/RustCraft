<script>
  import { Sparkles, Star, Mail, Percent, Building2, Download, Truck, ShieldAlert, CheckCircle2, AlertCircle, Scale } from 'lucide-svelte';

  export let settings = {};

  let reviewsEnabled = !!settings.reviews_enabled;
  let newsletterEnabled = !!settings.newsletter_enabled;
  let salePricesEnabled = !!settings.sale_prices_enabled;
  let reverseChargeEnabled = !!settings.reverse_charge_enabled;
  let downloadLimit = settings.download_limit ?? 10;
  let downloadDays = settings.download_days ?? 0;
  let defaultDeliveryTime = settings.default_delivery_time ?? '2–4 business days';
  let gpsrDefaultManufacturer = settings.gpsr_default_manufacturer || '';

  $: canReverseCharge = settings.tax_mode !== 'kleingewerbe' && !!(settings.vat_id || '').trim();

  let saving = false;
  let notice = '';
  let error = '';

  function useShopAsManufacturer() {
    const name = settings.legal_name || settings.store_name || '';
    const lines = [name, settings.company_address || '', settings.support_email || ''].filter((l) => l.trim());
    gpsrDefaultManufacturer = lines.join('\n');
  }

  async function save() {
    saving = true;
    notice = '';
    error = '';
    try {
      // Only this page's own fields
      const res = await fetch('/api/v1/admin/settings/system', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          reviews_enabled: reviewsEnabled,
          newsletter_enabled: newsletterEnabled,
          sale_prices_enabled: salePricesEnabled,
          reverse_charge_enabled: reverseChargeEnabled,
          download_limit: Math.max(0, parseInt(downloadLimit) || 0),
          download_days: Math.max(0, parseInt(downloadDays) || 0),
          default_delivery_time: defaultDeliveryTime,
          gpsr_default_manufacturer: gpsrDefaultManufacturer
        })
      });
      const body = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(body.error || 'Saving failed');
      notice = 'Saved. The shop uses the new settings right away.';
    } catch (e) {
      error = e.message;
    } finally {
      saving = false;
    }
  }

  const input = 'w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500';
  const card = 'p-5 rounded-2xl bg-slate-900 border border-slate-800 space-y-3';
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5"><Sparkles size={22} class="text-orange-500" /> Shop Features & Legal</h2>
    <p class="text-xs text-slate-400 mt-1">Optional features and the information German / EU law requires. Every option is explained below.</p>
  </div>

  {#if notice}<div class="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs flex items-center gap-2"><CheckCircle2 size={15} /> {notice}</div>{/if}
  {#if error}<div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-2"><AlertCircle size={15} /> {error}</div>{/if}

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-5">
    <section class={card}>
      <label class="flex items-start gap-3 cursor-pointer">
        <input type="checkbox" bind:checked={reviewsEnabled} class="w-4 h-4 mt-0.5 accent-orange-600" />
        <span>
          <span class="text-sm font-bold text-white flex items-center gap-2"><Star size={15} class="text-amber-400" /> Product reviews</span>
          <span class="block text-[11px] text-slate-400 mt-1">Customers who bought a product can rate it. Every review waits for your approval (Products → Product Reviews). The shop states that only verified buyers can review (§ 5b UWG).</span>
        </span>
      </label>
    </section>

    <section class={card}>
      <label class="flex items-start gap-3 cursor-pointer">
        <input type="checkbox" bind:checked={newsletterEnabled} class="w-4 h-4 mt-0.5 accent-orange-600" />
        <span>
          <span class="text-sm font-bold text-white flex items-center gap-2"><Mail size={15} class="text-sky-400" /> Newsletter (double opt-in)</span>
          <span class="block text-[11px] text-slate-400 mt-1">Sign-up in the footer and an optional, unticked box at checkout. Subscribers only receive newsletters after confirming their address by email; the time and IP of sign-up and confirmation are stored as proof. Every newsletter has an unsubscribe link. Write and send newsletters under Storefront & Design → Newsletter.</span>
        </span>
      </label>
    </section>

    <section class={card}>
      <label class="flex items-start gap-3 cursor-pointer">
        <input type="checkbox" bind:checked={salePricesEnabled} class="w-4 h-4 mt-0.5 accent-orange-600" />
        <span>
          <span class="text-sm font-bold text-white flex items-center gap-2"><Percent size={15} class="text-rose-400" /> Sale prices</span>
          <span class="block text-[11px] text-slate-400 mt-1">Set a sale price per product/variant and an optional start and end date. As required by § 11 PAngV the shop shows the <strong class="text-slate-200">lowest price of the 30 days before the reduction</strong> as the comparison price (it records every price automatically). A reduction is only advertised when such a price exists.</span>
        </span>
      </label>
    </section>

    <section class={card}>
      <label class="flex items-start gap-3 {canReverseCharge ? 'cursor-pointer' : 'opacity-60'}">
        <input type="checkbox" bind:checked={reverseChargeEnabled} disabled={!canReverseCharge && !reverseChargeEnabled} class="w-4 h-4 mt-0.5 accent-orange-600" />
        <span>
          <span class="text-sm font-bold text-white flex items-center gap-2"><Building2 size={15} class="text-indigo-400" /> VAT-free orders for EU businesses</span>
          <span class="block text-[11px] text-slate-400 mt-1">Business customers from other EU countries enter their VAT ID at checkout. It is checked live with the EU VIES service (the result is stored as proof); valid IDs get a VAT-free order: intra-Community supply for goods, reverse charge for digital products, with the required notes on the invoice. Remember to report these sales in your recapitulative statement (Zusammenfassende Meldung).</span>
          {#if !canReverseCharge}
            <span class="block text-[11px] text-amber-300 mt-1">Needs a VAT ID and a tax mode with VAT (not available for small businesses under § 19 UStG).</span>
          {/if}
        </span>
      </label>
    </section>

    <section class={card}>
      <div class="text-sm font-bold text-white flex items-center gap-2"><Download size={15} class="text-sky-400" /> Digital downloads</div>
      <p class="text-[11px] text-slate-400">Customers get personal download links; the files themselves are never publicly reachable. Limits stop shared links from being used by others. You can reset them per order.</p>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label for="dl-limit" class="block text-[11px] font-semibold text-slate-400 mb-1">Downloads per file (0 = unlimited)</label>
          <input id="dl-limit" type="number" min="0" max="1000" bind:value={downloadLimit} class={input} />
        </div>
        <div>
          <label for="dl-days" class="block text-[11px] font-semibold text-slate-400 mb-1">Link valid for days (0 = unlimited)</label>
          <input id="dl-days" type="number" min="0" max="3650" bind:value={downloadDays} class={input} />
        </div>
      </div>
    </section>

    <section class={card}>
      <div class="text-sm font-bold text-white flex items-center gap-2"><Truck size={15} class="text-emerald-400" /> Delivery time (fallback)</div>
      <p class="text-[11px] text-slate-400">Delivery times come from your shipping rates (field "delivery time" per rate) and depend on the destination country: customers see them at checkout before ordering (Art. 246a § 1 Nr. 7 EGBGB), and logged-in customers with a saved address also on the product page. This text is only used at checkout when a rate has no delivery time. Products can state their own time (e.g. made to order).</p>
      <input bind:value={defaultDeliveryTime} maxlength="120" placeholder="e.g. 2–4 business days" class={input} />
    </section>

    <section class="{card} lg:col-span-2">
      <div class="text-sm font-bold text-white flex items-center gap-2"><ShieldAlert size={15} class="text-orange-400" /> Product safety (GPSR) — default manufacturer</div>
      <p class="text-[11px] text-slate-400">
        Since 13 December 2024 every online offer of a physical product must name the manufacturer (name, postal and electronic address) and, if the manufacturer is outside the EU, the responsible person in the EU, plus warnings/safety information where needed.
        Enter the details here if you make most products yourself; products from other manufacturers get their own details in the product form.
      </p>
      <textarea bind:value={gpsrDefaultManufacturer} rows="3" placeholder="Company name&#10;Street 1, 12345 City, Country&#10;email@example.com" class={input}></textarea>
      <button type="button" on:click={useShopAsManufacturer} class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-white text-[11px] font-bold">Use my shop's details</button>
    </section>
  </div>

  <section class="p-5 rounded-2xl bg-indigo-500/5 border border-indigo-500/20 space-y-2 text-[11px] text-slate-300">
    <div class="text-sm font-bold text-white flex items-center gap-2"><Scale size={15} class="text-indigo-400" /> Always active (no setting needed)</div>
    <ul class="list-disc list-inside space-y-1 text-slate-400">
      <li><strong class="text-slate-200">Withdrawal button</strong> — "Withdraw from contract here" in the footer of every page, with a confirmation step and an automatic receipt by email (EU law since 19 June 2026). Withdrawals appear under Orders → Withdrawals.</li>
      <li><strong class="text-slate-200">Invoices are stored unchangeable</strong> when the payment is confirmed; refunds create a credit note (GoBD).</li>
      <li><strong class="text-slate-200">VAT note and shipping link</strong> next to every price; the unit price (per kg / l / m / m²) for products with a net content.</li>
      <li><strong class="text-slate-200">Privacy policy</strong> sections for the features you use are filled in automatically.</li>
      <li>The EU online dispute resolution (ODR) platform was shut down on 20 July 2025 — the shop no longer links to it.</li>
    </ul>
  </section>

  <div class="flex justify-end">
    <button type="button" on:click={save} disabled={saving} class="px-6 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-xs font-bold">
      {saving ? 'Saving…' : 'Save'}
    </button>
  </div>
</div>
