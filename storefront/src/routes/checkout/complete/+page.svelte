<script>
  // Return target for payments that leave the shop (3-D Secure, PayPal / Amazon Pay via Stripe,
  // hosted Stripe Checkout). The order itself is created server-side from the stored checkout,
  // so nothing here depends on browser storage surviving the redirect.
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { cart } from '$lib/stores/cart.js';
  import { submitRememberedNewsletterOptIn } from '$lib/newsletter.js';
  import { AlertCircle, Loader2 } from 'lucide-svelte';

  export let data;
  $: storeName = data.store?.store_name || 'Shop';

  let status = 'verifying'; // verifying | error
  let errorMessage = '';

  onMount(async () => {
    const params = $page.url.searchParams;
    const paymentIntentId = params.get('payment_intent');
    const sessionId = params.get('session_id');
    const redirectStatus = params.get('redirect_status');

    if (!paymentIntentId && !sessionId) {
      status = 'error';
      errorMessage = 'Missing payment reference.';
      return;
    }
    if (redirectStatus === 'failed') {
      status = 'error';
      errorMessage = 'The payment was not completed. No money has been charged — please try again or choose another payment method.';
      return;
    }

    try {
      const res = await fetch('/api/v1/checkout/stripe/complete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(paymentIntentId ? { payment_intent_id: paymentIntentId } : { session_id: sessionId })
      });
      if (!res.ok) {
        const text = await res.text();
        let msg = text;
        try {
          const j = JSON.parse(text);
          msg = j.error || j.message || text;
        } catch (_) {}
        throw new Error(msg || 'Your payment could not be confirmed.');
      }
      const order = await res.json();
      cart.clear();
      submitRememberedNewsletterOptIn();
      goto(`/order-success/${encodeURIComponent(order.order_number)}?token=${encodeURIComponent(order.access_token || "")}`, { replaceState: true });
    } catch (e) {
      status = 'error';
      errorMessage = e.message || 'An error occurred while confirming your payment.';
    }
  });
</script>

<svelte:head>
  <title>Confirming Payment | {storeName}</title>
</svelte:head>

<div class="min-h-[60vh] flex items-center justify-center px-4 py-16">
  <div class="max-w-md w-full p-8 rounded-3xl bg-slate-900 border border-slate-800 text-center space-y-5 shadow-2xl">
    {#if status === 'verifying'}
      <div class="w-16 h-16 rounded-2xl bg-orange-500/10 border border-orange-500/20 flex items-center justify-center mx-auto">
        <Loader2 size={32} class="animate-spin text-orange-500" />
      </div>
      <div class="space-y-2">
        <h1 class="text-xl font-black text-white tracking-tight">Confirming your payment…</h1>
        <p class="text-xs text-slate-400">Please do not close or refresh this page.</p>
      </div>
    {:else}
      <div class="w-16 h-16 rounded-2xl bg-rose-500/10 border border-rose-500/20 flex items-center justify-center mx-auto">
        <AlertCircle size={32} class="text-rose-500" />
      </div>
      <div class="space-y-2">
        <h1 class="text-xl font-black text-rose-300 tracking-tight">Payment not confirmed</h1>
        <p class="text-xs text-slate-400 leading-relaxed">{errorMessage}</p>
      </div>
      <a href="/checkout" class="inline-block px-5 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-bold transition-all">
        Return to Checkout
      </a>
    {/if}
  </div>
</div>
