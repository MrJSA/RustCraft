<script>
  import { Truck, ShieldCheck, ArrowLeft, ExternalLink, Globe, CheckCircle2 } from 'lucide-svelte';
  import Seo from '$lib/components/Seo.svelte';

  export let data;
  $: page = data.page || {};
  $: providers = data.providers || [];
  $: storeName = data.store?.store_name || 'Shop';

  let regionNames = null;
  try {
    regionNames = new Intl.DisplayNames(['en'], { type: 'region' });
  } catch (_) {}
  function countryName(code) {
    try {
      return (code === 'XK' ? 'Kosovo' : regionNames?.of(code)) || code;
    } catch (_) {
      return code;
    }
  }
  const countryNames = (codes) => (Array.isArray(codes) ? codes.map(countryName).sort((a, b) => a.localeCompare(b)) : []);

  // Simple and safe client markdown renderer
  function parseMarkdown(md) {
    if (!md) return '';
    let html = md
      // Escaping basic HTML tags
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      // Headings
      .replace(/^### (.*$)/gim, '<h3 class="text-lg font-bold text-white mt-6 mb-2 tracking-tight">$1</h3>')
      .replace(/^## (.*$)/gim, '<h2 class="text-xl font-extrabold text-white mt-8 mb-3 tracking-tight border-b border-slate-800 pb-2">$1</h2>')
      .replace(/^# (.*$)/gim, '<h1 class="text-3xl font-black text-white mb-6 tracking-tight bg-gradient-to-r from-white via-slate-100 to-slate-400 bg-clip-text text-transparent">$1</h1>')
      // Bold & Italic
      .replace(/\*\*(.*?)\*\*/gim, '<strong class="font-bold text-slate-100">$1</strong>')
      .replace(/\*(.*?)\*/gim, '<em class="italic text-slate-300">$1</em>')
      // Code tags
      .replace(/`(.*?)`/gim, '<code class="px-1.5 py-0.5 rounded bg-slate-900 border border-slate-800 font-mono text-orange-400 text-xs">$1</code>')
      // Links
      .replace(/\[(.*?)\]\((.*?)\)/gim, '<a href="$2" target="_blank" rel="noopener noreferrer" class="text-orange-400 hover:text-orange-300 underline underline-offset-2">$1</a>')
      // Unordered lists
      .replace(/^\s*-\s+(.*$)/gim, '<li class="ml-4 list-disc text-slate-300 leading-relaxed">$1</li>')
      // Markdown hard line breaks (two trailing spaces)
      .replace(/ {2,}\n/g, '<br>\n')
      // Paragraphs
      .replace(/\n\n/gim, '</p><p class="my-4 text-slate-300 leading-relaxed text-sm">');

    return `<p class="my-4 text-slate-300 leading-relaxed text-sm">${html}</p>`;
  }
</script>

<Seo title={`${page.title} | ${storeName}`} description={page.content_markdown} path={`/policies/${page.slug}`} />

<div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
  <!-- Back navigation -->
  <a
    href="/"
    class="inline-flex items-center gap-1.5 text-xs text-slate-400 hover:text-white mb-8 transition-colors group"
  >
    <ArrowLeft size={14} class="group-hover:-translate-x-0.5 transition-transform" />
    <span>Back to Store</span>
  </a>

  <!-- Content Card -->
  <div class="p-6 sm:p-10 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-8">
    <!-- Rendered Markdown Body -->
    <div class="prose prose-invert max-w-none text-slate-300 leading-relaxed">
      {@html parseMarkdown(page.content_markdown)}
    </div>

    <!-- Dynamic Live Shipping Rates Table (Only on Shipment Policy page) -->
    {#if page.slug === 'shipment-policy'}
      <div class="mt-10 pt-8 border-t border-slate-800 space-y-6">
        <div class="flex items-center gap-3">
          <div class="p-2.5 rounded-xl bg-orange-500/10 border border-orange-500/20 text-orange-400">
            <Truck size={22} />
          </div>
          <div>
            <h3 class="text-lg font-bold text-white">Live Courier Schedules & Rate Cards</h3>
            <p class="text-xs text-slate-400">Real-time shipping price categories configured in back-office:</p>
          </div>
        </div>

        {#if providers.length === 0}
          <div class="p-6 rounded-2xl bg-slate-950 border border-slate-800 text-center text-xs text-slate-400">
            No shipping providers are currently published.
          </div>
        {:else}
          <div class="space-y-6">
            {#each providers as prov}
              <div class="rounded-2xl bg-slate-950 border border-slate-800/80 overflow-hidden shadow-lg">
                <!-- Provider Header -->
                <div class="p-4 bg-slate-900/80 border-b border-slate-800 flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <span class="font-bold text-sm text-white">{prov.name}</span>
                    <span class="text-[10px] font-mono uppercase px-2 py-0.5 rounded bg-slate-800 text-orange-400 font-semibold border border-slate-700">
                      {prov.code}
                    </span>
                  </div>
                  <span class="text-[11px] text-emerald-400 font-medium flex items-center gap-1">
                    <CheckCircle2 size={12} /> Active Carrier
                  </span>
                </div>

                <!-- Provider Zones & Price Categories Table -->
                <div class="p-4 space-y-4">
                  {#if !prov.zones || prov.zones.length === 0}
                    <div class="text-xs text-slate-500 py-2">No regional zones configured for this carrier.</div>
                  {:else}
                    {#each prov.zones as zone}
                      <div class="border border-slate-800/60 rounded-xl p-4 bg-slate-900/40">
                        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 mb-3">
                          <div class="flex items-center gap-2">
                            <Globe size={14} class="text-orange-400 flex-shrink-0" />
                            <span class="text-xs font-bold text-white">{zone.zone_name}</span>
                          </div>
                          <!-- Countries: count only; all names on hover and when opened -->
                          {#if Array.isArray(zone.country_codes) && zone.country_codes.length > 0}
                            {@const names = countryNames(zone.country_codes)}
                            <details class="group sm:text-right">
                              <summary title={names.join(', ')} class="cursor-pointer list-none inline-flex items-center gap-1 px-2 py-0.5 rounded-lg bg-slate-800 border border-slate-700 text-[11px] text-slate-300 hover:text-white">
                                {names.length === 1 ? names[0] : `Ships to ${names.length} countries`}
                                {#if names.length > 1}<span class="text-slate-500 group-open:rotate-180 transition-transform">▾</span>{/if}
                              </summary>
                              {#if names.length > 1}
                                <p class="mt-2 max-w-md text-[11px] leading-relaxed text-slate-400 text-left">{names.join(', ')}</p>
                              {/if}
                            </details>
                          {:else if zone.is_default}
                            <span class="text-[11px] text-slate-400">All other countries</span>
                          {/if}
                        </div>

                        <!-- Price Categories / Rates -->
                        <div class="overflow-x-auto">
                          <table class="w-full text-left text-xs">
                            <thead>
                              <tr class="border-b border-slate-800 text-[11px] text-slate-400 uppercase">
                                <th class="pb-2 font-semibold">Package Tier / Service</th>
                                <th class="pb-2 font-semibold">Weight Range</th>
                                <th class="pb-2 font-semibold">Estimated Delivery</th>
                                <th class="pb-2 font-semibold text-right">Price</th>
                              </tr>
                            </thead>
                            <tbody class="divide-y divide-slate-800/60">
                              {#if !zone.rates || zone.rates.length === 0}
                                <tr>
                                  <td colspan="4" class="py-2 text-slate-500 text-center">No rate options defined</td>
                                </tr>
                              {:else}
                                {#each zone.rates as rate}
                                  <tr class="text-slate-300">
                                    <td class="py-2.5 font-medium text-white">{rate.name}</td>
                                    <td class="py-2.5 text-slate-400 font-mono text-[11px]">
                                      {(rate.min_weight_g / 1000).toFixed(1)}kg - {(rate.max_weight_g / 1000).toFixed(1)}kg
                                    </td>
                                    <td class="py-2.5 text-slate-400">{rate.estimated_delivery_days}</td>
                                    <td class="py-2.5 text-right font-mono font-bold text-white">
                                      {(rate.price_cents / 100).toFixed(2)} €
                                    </td>
                                  </tr>
                                {/each}
                              {/if}
                            </tbody>
                          </table>
                        </div>
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Last updated badge -->
    <div class="pt-6 border-t border-slate-800 flex items-center justify-between text-xs text-slate-500">
      <span>Official store policy document</span>
      <span>Last modified: {new Date(page.updated_at || Date.now()).toLocaleDateString()}</span>
    </div>
  </div>
</div>
