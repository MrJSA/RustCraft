<script>
  // Legally required next to every price: VAT status and whether shipping costs apply (PAngV)
  import { vatNote } from '$lib/pricing.js';

  export let store = {};
  export let digital = false;
  export let compact = false;
</script>

<p class="{compact ? 'text-[10px]' : 'text-xs'} text-slate-400 leading-snug">
  {vatNote(store)}{#if !digital}, plus <a href="/policies/shipment-policy" class="underline hover:text-orange-400">shipping</a>{/if}
</p>
