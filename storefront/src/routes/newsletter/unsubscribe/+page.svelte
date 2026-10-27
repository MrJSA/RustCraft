<script>
  import { page } from '$app/stores';
  import Seo from '$lib/components/Seo.svelte';
  import { CheckCircle2, MailX } from 'lucide-svelte';

  let done = false;
  let message = '';
  let sending = false;

  async function unsubscribe() {
    sending = true;
    try {
      const res = await fetch('/api/v1/newsletter/unsubscribe', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token: $page.url.searchParams.get('token') || '' })
      });
      const data = await res.json().catch(() => ({}));
      message = data.message || 'You have been unsubscribed.';
      done = true;
    } finally {
      sending = false;
    }
  }
</script>

<Seo title="Unsubscribe from the newsletter" noindex />

<div class="max-w-lg mx-auto px-4 py-20 text-center space-y-4">
  {#if done}
    <CheckCircle2 size={40} class="mx-auto text-emerald-400" />
    <h1 class="text-2xl font-black text-white">Unsubscribed</h1>
    <p class="text-slate-300">{message}</p>
  {:else}
    <MailX size={40} class="mx-auto text-orange-400" />
    <h1 class="text-2xl font-black text-white">Unsubscribe from the newsletter?</h1>
    <p class="text-slate-300">You will not receive any more newsletters from us.</p>
    <button on:click={unsubscribe} disabled={sending} class="px-6 py-3 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-sm font-bold">
      {sending ? 'Unsubscribing…' : 'Unsubscribe'}
    </button>
  {/if}
</div>
