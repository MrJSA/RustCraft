<script>
  import { Download, AlertTriangle, ArrowRight, Layers, Bell } from "lucide-svelte";
  import { page } from "$app/stores";
  import { lowestPricing, formatEuro, reductionPercent } from "$lib/pricing.js";
  import PriceNote from "$lib/components/PriceNote.svelte";

  export let item;
  export let variants = [];
  // Store settings for the VAT note (taken from the layout when not passed)
  export let store = null;

  $: allVariants = variants && variants.length ? variants : item.variants || [];
  $: pricing = lowestPricing(item, allVariants);
  $: priceVaries = new Set(allVariants.map((v) => v.pricing?.price_cents ?? v.price_override_cents ?? item.base_price_cents)).size > 1;
  $: storeInfo = store || $page.data?.store || {};

  $: isOutOfStock = item.product_type === 'physical' && allVariants.length > 0 && allVariants.every(v => v.stock_quantity <= 0);
  $: hasLowStock = item.product_type === 'physical' && allVariants.some(v => v.stock_quantity > 0 && v.stock_quantity <= v.low_stock_threshold);
</script>

<div class="group relative rounded-2xl bg-slate-900/60 border border-slate-800/80 hover:border-orange-500/40 transition-all duration-300 flex flex-col overflow-hidden hover:shadow-2xl hover:shadow-orange-950/20 h-full">
  <!-- Thumbnail Container -->
  <a
    href="/products/{item.slug}"
    class="relative aspect-square overflow-hidden bg-slate-950 block"
  >
    {#if item.image_url}
      <img
        src={item.image_url}
        alt={item.title}
        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500 ease-out {isOutOfStock ? 'opacity-60 grayscale' : ''}"
        loading="lazy"
      />
    {:else}
      <div class="w-full h-full flex items-center justify-center text-4xl text-slate-700">
        📦
      </div>
    {/if}

    <!-- Badges -->
    <div class="absolute top-2.5 left-2.5 flex flex-col gap-1 z-10">
      {#if pricing.on_sale}
        <span class="inline-flex items-center px-2 py-0.5 rounded text-[9px] font-black bg-rose-600 text-white shadow-sm">
          Sale −{reductionPercent(pricing)} %
        </span>
      {/if}
      {#if item.product_type === "digital"}
        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[9px] font-bold bg-sky-500/90 text-white backdrop-blur-md shadow-sm">
          <Download size={10} /> Digital
        </span>
      {:else if isOutOfStock}
        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[9px] font-bold bg-rose-600/90 text-white backdrop-blur-md shadow-sm">
          Out of Stock
        </span>
      {:else if hasLowStock}
        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[9px] font-bold bg-amber-500 text-slate-950 shadow-sm animate-pulse">
          <AlertTriangle size={10} /> Low Stock
        </span>
      {/if}
    </div>

    <!-- Category Pill -->
    {#if item.subcategory || item.category}
      <div class="absolute bottom-2.5 left-2.5 z-10">
        <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-slate-950/85 text-orange-400 border border-slate-800">
          {item.subcategory || item.category}
        </span>
      </div>
    {/if}
  </a>

  <!-- Details & Price -->
  <div class="p-3.5 sm:p-4 flex-1 flex flex-col justify-between">
    <div>
      <a href="/products/{item.slug}" class="block group-hover:text-orange-400 transition-colors">
        <h3 class="text-xs sm:text-sm font-bold text-white leading-snug line-clamp-2">
          {item.title}
        </h3>
      </a>

      {#if item.description}
        <p class="text-[11px] text-slate-400 mt-1 line-clamp-2 leading-relaxed">
          {item.description}
        </p>
      {/if}
    </div>

    <div class="mt-3 pt-3 border-t border-slate-800/80 flex items-center justify-between gap-2">
      <div class="min-w-0">
        {#if priceVaries}
          <span class="text-[9px] text-slate-400 block uppercase font-mono tracking-wider">From</span>
        {/if}
        <span class="text-sm sm:text-base font-black font-mono {pricing.on_sale ? 'text-rose-400' : 'text-white'}">
          {formatEuro(pricing.price_cents)}
        </span>
        {#if pricing.on_sale}
          <span class="block text-[10px] text-slate-500 font-mono" title="Lowest price in the 30 days before the reduction">
            <span class="line-through">{formatEuro(pricing.reference_cents)}</span> · −{reductionPercent(pricing)} %
          </span>
        {/if}
        {#if pricing.unit_price}
          <span class="block text-[10px] text-slate-500 font-mono">({pricing.unit_price})</span>
        {/if}
        <PriceNote store={storeInfo} digital={item.product_type === 'digital'} compact />
      </div>

      <a
        href="/products/{item.slug}"
        class="px-2.5 py-1.5 rounded-lg bg-orange-600 hover:bg-orange-500 text-white text-[11px] font-bold transition-all shadow-md shadow-orange-600/20 flex items-center gap-1"
      >
        {#if isOutOfStock}
          <Bell size={11} />
          <span>Notify</span>
        {:else}
          <span>View</span>
          <ArrowRight size={11} />
        {/if}
      </a>
    </div>
  </div>
</div>
