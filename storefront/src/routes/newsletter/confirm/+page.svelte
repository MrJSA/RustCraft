<script>
  // Confirmation runs in the browser (not on a plain GET of the link), so email scanners that
  // open links automatically cannot confirm a subscription on someone's behalf.
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import Seo from '$lib/components/Seo.svelte';
  import { CheckCircle2, AlertCircle, Loader2 } from 'lucide-svelte';

  let state = 'working';
  let message = '';

  onMount(async () => {
    const token = $page.url.searchParams.get('token') || '';
    try {
      const res = await fetch('/api/v1/newsletter/confirm', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token })
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(data.error || 'The confirmation did not work.');
      message = data.message;
      state = 'ok';
    } catch (e) {
      message = e.message;
      state = 'error';
    }
  });
</script>

<Seo title="Newsletter confirmation" noindex />

<div class="max-w-lg mx-auto px-4 py-20 text-center space-y-4">
  {#if state === 'working'}
    <Loader2 size={36} class="mx-auto text-orange-400 animate-spin" />
    <p class="text-slate-300">Confirming your subscription…</p>
  {:else if state === 'ok'}
    <CheckCircle2 size={40} class="mx-auto text-emerald-400" />
    <h1 class="text-2xl font-black text-white">Subscription confirmed</h1>
    <p class="text-slate-300">{message}</p>
    <a href="/" class="inline-block mt-2 px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-sm font-bold">Continue shopping</a>
  {:else}
    <AlertCircle size={40} class="mx-auto text-rose-400" />
    <h1 class="text-2xl font-black text-white">Confirmation failed</h1>
    <p class="text-slate-300">{message}</p>
  {/if}
</div>
