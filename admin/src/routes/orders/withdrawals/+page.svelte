<script>
  import { onMount } from 'svelte';
  import { Undo2, CheckCircle2, AlertCircle, Mail } from 'lucide-svelte';

  let list = [];
  let loading = true;
  let error = '';
  let notes = {};

  async function api(path, options = {}) {
    const res = await fetch(`/api/v1/admin/${path}`, { ...options, headers: { 'Content-Type': 'application/json' } });
    const body = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(body.error || `Request failed (${res.status})`);
    return body;
  }

  async function load() {
    loading = true;
    try {
      list = await api('withdrawals');
      notes = Object.fromEntries(list.map((w) => [w.id, w.admin_note]));
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  }

  async function update(w, status) {
    try {
      await api(`withdrawals/${w.id}`, { method: 'PUT', body: JSON.stringify({ status, admin_note: notes[w.id] ?? '' }) });
      await load();
    } catch (e) {
      error = e.message;
    }
  }

  onMount(load);
</script>

<div class="space-y-6">
  <div>
    <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5"><Undo2 size={24} class="text-orange-500" /> Withdrawals</h1>
    <p class="text-xs text-slate-400 mt-1">
      Declarations sent with the withdrawal button in the shop. The customer automatically received a confirmation of receipt by email.
      Refund the payment within 14 days (for goods: once they are back or the customer proved sending them) via the order's refund button — the credit note is created automatically.
    </p>
  </div>

  {#if error}<div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-2"><AlertCircle size={15} /> {error}</div>{/if}

  {#if loading}
    <p class="text-xs text-slate-400">Loading…</p>
  {:else if list.length === 0}
    <p class="text-sm text-slate-400">No withdrawals received.</p>
  {:else}
    <div class="space-y-3">
      {#each list as w}
        <div class="p-5 rounded-2xl bg-slate-900 border {w.status === 'processed' ? 'border-slate-800' : 'border-amber-500/30'} space-y-3">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div class="flex items-center gap-3">
              <span class="font-mono font-bold text-white">{w.order_number}</span>
              <span class="text-[10px] px-2 py-0.5 rounded-full font-bold uppercase {w.status === 'processed' ? 'bg-emerald-500/15 text-emerald-400' : 'bg-amber-500/15 text-amber-300'}">{w.status === 'processed' ? 'Processed' : 'Open'}</span>
              {#if !w.order_id}<span class="text-[10px] text-rose-300">No order with this number and email found — please check manually</span>{/if}
            </div>
            <span class="text-[11px] text-slate-500">Received {new Date(w.received_at).toLocaleString()}</span>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs text-slate-300">
            <div><span class="text-slate-500">Customer:</span> {w.customer_name} · <a href="mailto:{w.customer_email}" class="underline">{w.customer_email}</a></div>
            <div class="flex items-center gap-1"><Mail size={12} class="text-slate-500" /> {w.confirmation_sent_at ? `Confirmation sent ${new Date(w.confirmation_sent_at).toLocaleString()}` : 'Confirmation email could not be sent — please confirm manually'}</div>
            <div class="sm:col-span-2"><span class="text-slate-500">Items:</span> {w.items || 'The whole order'}</div>
            {#if w.message}<div class="sm:col-span-2"><span class="text-slate-500">Message:</span> {w.message}</div>{/if}
          </div>
          <div class="flex flex-col sm:flex-row gap-2">
            <input bind:value={notes[w.id]} placeholder="Internal note (e.g. parcel received, refunded on …)" class="flex-1 px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500" />
            {#if w.status !== 'processed'}
              <button type="button" on:click={() => update(w, 'processed')} class="px-4 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-bold flex items-center gap-1"><CheckCircle2 size={14} /> Mark processed</button>
            {:else}
              <button type="button" on:click={() => update(w, 'received')} class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-bold">Reopen</button>
            {/if}
            <button type="button" on:click={() => update(w, null)} class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-bold">Save note</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
