<script>
  // Double opt-in sign-up: the subscription only starts after the link in the confirmation email
  // was clicked. Consent text must match the one stored by the backend.
  import { Mail } from 'lucide-svelte';

  const CONSENT = 'I would like to receive the newsletter with offers and news by email. I can unsubscribe at any time with the link in every newsletter.';

  let email = '';
  let consent = false;
  let sending = false;
  let message = '';
  let error = '';

  async function subscribe() {
    error = '';
    if (!consent) {
      error = 'Please tick the box to agree to receiving the newsletter.';
      return;
    }
    sending = true;
    try {
      const res = await fetch('/api/v1/newsletter/subscribe', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, consent, source: 'footer' })
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(data.error || 'The sign-up did not work. Please try again.');
      message = data.message;
      email = '';
      consent = false;
    } catch (e) {
      error = e.message;
    } finally {
      sending = false;
    }
  }
</script>

<div class="p-5 rounded-2xl bg-slate-900/60 border border-slate-800 space-y-3">
  <h4 class="text-xs font-bold text-white uppercase tracking-wider flex items-center gap-2"><Mail size={14} class="text-orange-400" /> Newsletter</h4>
  {#if message}
    <p class="text-xs text-emerald-300">{message}</p>
  {:else}
    <form on:submit|preventDefault={subscribe} class="space-y-2.5">
      <div class="flex gap-2">
        <label for="newsletter-email" class="sr-only">Email address</label>
        <input
          id="newsletter-email"
          type="email"
          bind:value={email}
          required
          placeholder="your.email@example.com"
          class="flex-1 min-w-0 px-3 py-2 rounded-xl bg-slate-950 border border-slate-700 text-white text-xs placeholder-slate-500 focus:outline-none focus:border-orange-500"
        />
        <button type="submit" disabled={sending} class="px-4 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-xs font-bold">
          {sending ? '…' : 'Subscribe'}
        </button>
      </div>
      <label class="flex items-start gap-2 text-[11px] text-slate-400 cursor-pointer">
        <input type="checkbox" bind:checked={consent} class="mt-0.5 accent-orange-600" />
        <span>{CONSENT} More in our <a href="/policies/privacy-policy" class="underline hover:text-white">privacy policy</a>.</span>
      </label>
      {#if error}<p class="text-[11px] text-rose-400">{error}</p>{/if}
    </form>
  {/if}
</div>
