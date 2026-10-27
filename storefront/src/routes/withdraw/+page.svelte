<script>
  // Withdrawal function for contracts concluded online (Art. 11a Consumer Rights Directive,
  // applicable since 19 June 2026): enter details → confirm withdrawal → confirmation by email.
  import { page } from '$app/stores';
  import { customer } from '$lib/stores/customer.js';
  import Seo from '$lib/components/Seo.svelte';
  import { Undo2, CheckCircle2, AlertCircle } from 'lucide-svelte';

  let step = 'details';
  let orderNumber = $page.url.searchParams.get('order') || '';
  let customerName = $customer?.full_name || '';
  let customerEmail = $customer?.email || '';
  let items = '';
  let message = '';
  let sending = false;
  let error = '';
  let result = null;

  function review() {
    error = '';
    if (!orderNumber.trim() || !customerName.trim() || !customerEmail.includes('@')) {
      error = 'Please enter your name, your email address and the order number.';
      return;
    }
    step = 'confirm';
  }

  async function confirmWithdrawal() {
    sending = true;
    error = '';
    try {
      const res = await fetch('/api/v1/withdrawals', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ order_number: orderNumber, customer_name: customerName, customer_email: customerEmail, items, message })
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(data.error || 'The withdrawal could not be sent. Please try again or contact us by email.');
      result = data;
      step = 'done';
    } catch (e) {
      error = e.message;
    } finally {
      sending = false;
    }
  }

  const input = 'w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-700 text-white text-sm focus:outline-none focus:border-orange-500';
</script>

<Seo title="Withdraw from contract" description="Withdraw from a contract concluded in this shop." noindex />

<div class="max-w-2xl mx-auto px-4 py-12 space-y-6">
  <div>
    <h1 class="text-3xl font-black text-white tracking-tight flex items-center gap-3"><Undo2 size={28} class="text-orange-400" /> Withdraw from contract</h1>
    <p class="mt-2 text-sm text-slate-400">
      Use this form to withdraw from a contract you concluded in our shop. You receive a confirmation of receipt by email right away.
      Details on your right of withdrawal are in our <a href="/policies/revocation-policy" class="underline hover:text-orange-400">cancellation policy</a>.
    </p>
  </div>

  {#if error}
    <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-300 text-sm flex items-start gap-2"><AlertCircle size={16} class="flex-shrink-0 mt-0.5" /> {error}</div>
  {/if}

  {#if step === 'details'}
    <form on:submit|preventDefault={review} class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800 space-y-4">
      <div>
        <label for="w-order" class="block text-xs font-semibold text-slate-300 mb-1">Order number *</label>
        <input id="w-order" bind:value={orderNumber} required placeholder="e.g. ORD-10042" class={input} />
      </div>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div>
          <label for="w-name" class="block text-xs font-semibold text-slate-300 mb-1">Your name *</label>
          <input id="w-name" bind:value={customerName} required autocomplete="name" class={input} />
        </div>
        <div>
          <label for="w-email" class="block text-xs font-semibold text-slate-300 mb-1">Email address used for the order *</label>
          <input id="w-email" type="email" bind:value={customerEmail} required autocomplete="email" class={input} />
        </div>
      </div>
      <div>
        <label for="w-items" class="block text-xs font-semibold text-slate-300 mb-1">Which items? (leave empty for the whole order)</label>
        <textarea id="w-items" bind:value={items} rows="3" placeholder="e.g. 1 × Pro Mechanical Keyboard" class={input}></textarea>
      </div>
      <div>
        <label for="w-message" class="block text-xs font-semibold text-slate-300 mb-1">Message (optional)</label>
        <textarea id="w-message" bind:value={message} rows="3" class={input}></textarea>
      </div>
      <p class="text-[11px] text-slate-500">We process these details to handle your withdrawal (see our <a href="/policies/privacy-policy" class="underline">privacy policy</a>).</p>
      <button type="submit" class="w-full py-3 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-sm font-bold">Continue</button>
    </form>
  {:else if step === 'confirm'}
    <div class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800 space-y-4">
      <h2 class="text-lg font-bold text-white">Please check your withdrawal</h2>
      <dl class="text-sm grid grid-cols-3 gap-y-2">
        <dt class="text-slate-400">Order</dt><dd class="col-span-2 text-white font-mono">{orderNumber}</dd>
        <dt class="text-slate-400">Name</dt><dd class="col-span-2 text-white">{customerName}</dd>
        <dt class="text-slate-400">Email</dt><dd class="col-span-2 text-white">{customerEmail}</dd>
        <dt class="text-slate-400">Items</dt><dd class="col-span-2 text-white whitespace-pre-line">{items.trim() || 'The whole order'}</dd>
        {#if message.trim()}<dt class="text-slate-400">Message</dt><dd class="col-span-2 text-white whitespace-pre-line">{message}</dd>{/if}
      </dl>
      <p class="text-sm text-slate-300">I hereby withdraw from the contract concluded for the items listed above.</p>
      <div class="flex flex-col sm:flex-row gap-3">
        <button type="button" on:click={() => (step = 'details')} class="px-5 py-3 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-sm font-bold">Change details</button>
        <button type="button" on:click={confirmWithdrawal} disabled={sending} class="flex-1 py-3 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-sm font-black">
          {sending ? 'Sending…' : 'Confirm withdrawal'}
        </button>
      </div>
    </div>
  {:else if step === 'done' && result}
    <div class="p-6 rounded-2xl bg-emerald-500/10 border border-emerald-500/30 space-y-3">
      <h2 class="text-lg font-bold text-emerald-300 flex items-center gap-2"><CheckCircle2 size={20} /> Withdrawal received</h2>
      <p class="text-sm text-slate-200">
        We received your withdrawal on {new Date(result.received_at).toLocaleString('en-GB')} (reference <span class="font-mono">{result.reference}</span>).
        {#if result.confirmation_email_sent}
          A confirmation has been sent to {customerEmail}.
        {:else}
          Please keep this page as proof — the confirmation email could not be sent right now.
        {/if}
      </p>
    </div>
  {/if}
</div>
