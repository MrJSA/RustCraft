<script>
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { Star, Check, X, Trash2, AlertCircle, MessageSquare } from 'lucide-svelte';

  let reviews = [];
  let filter = 'pending';
  let loading = true;
  let error = '';

  $: enabled = $page.data.settings?.reviews_enabled;

  async function api(path, options = {}) {
    const res = await fetch(`/api/v1/admin/${path}`, { ...options, headers: { 'Content-Type': 'application/json' } });
    const body = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(body.error || `Request failed (${res.status})`);
    return body;
  }

  async function load() {
    loading = true;
    error = '';
    try {
      reviews = await api(`reviews${filter ? `?status=${filter}` : ''}`);
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  }

  async function moderate(r, status) {
    try {
      await api(`reviews/${r.id}`, { method: 'PUT', body: JSON.stringify({ status }) });
      await load();
    } catch (e) {
      error = e.message;
    }
  }

  async function remove(r) {
    if (!confirm('Delete this review permanently?')) return;
    try {
      await api(`reviews/${r.id}`, { method: 'DELETE' });
      await load();
    } catch (e) {
      error = e.message;
    }
  }

  onMount(load);
</script>

<div class="space-y-6">
  <div class="flex flex-col sm:flex-row sm:items-end justify-between gap-3">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5"><MessageSquare size={24} class="text-orange-500" /> Product Reviews</h1>
      <p class="text-xs text-slate-400 mt-1">Reviews from verified buyers. Only approved reviews are shown in the shop — approve negative reviews too, hiding them would be misleading (§ 5b UWG).</p>
    </div>
    <div class="flex gap-2">
      {#each [['pending', 'Waiting'], ['approved', 'Approved'], ['rejected', 'Rejected'], ['', 'All']] as [value, label]}
        <button type="button" on:click={() => { filter = value; load(); }} class="px-3 py-1.5 rounded-lg text-xs font-bold {filter === value ? 'bg-orange-600 text-white' : 'bg-slate-900 text-slate-400 border border-slate-800 hover:text-white'}">{label}</button>
      {/each}
    </div>
  </div>

  <div class="p-3.5 rounded-xl text-xs border {enabled ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-300' : 'bg-amber-500/10 border-amber-500/20 text-amber-300'}">
    {#if enabled}
      Reviews are switched on in the shop.
    {:else}
      Reviews are switched off — customers cannot write reviews. Turn them on in <a href="/settings?tab=features" class="underline">Settings → Shop Features & Legal</a>.
    {/if}
  </div>

  {#if error}<div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-2"><AlertCircle size={15} /> {error}</div>{/if}

  {#if loading}
    <p class="text-xs text-slate-400">Loading…</p>
  {:else if reviews.length === 0}
    <p class="text-sm text-slate-400">No reviews here.</p>
  {:else}
    <div class="space-y-3">
      {#each reviews as r}
        <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 space-y-2">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div class="flex items-center gap-3">
              <span class="flex">{#each [1, 2, 3, 4, 5] as s}<Star size={14} class={s <= r.rating ? 'fill-amber-400 text-amber-400' : 'text-slate-600'} />{/each}</span>
              <span class="text-xs font-bold text-white">{r.product_title || 'Deleted product'}</span>
              <span class="text-[10px] px-2 py-0.5 rounded-full font-bold uppercase {r.status === 'approved' ? 'bg-emerald-500/15 text-emerald-400' : r.status === 'rejected' ? 'bg-rose-500/15 text-rose-400' : 'bg-amber-500/15 text-amber-300'}">{r.status}</span>
            </div>
            <span class="text-[11px] text-slate-500">{new Date(r.created_at).toLocaleString()} · {r.author_name} · {r.customer_email}</span>
          </div>
          {#if r.title}<div class="text-sm font-bold text-white">{r.title}</div>{/if}
          {#if r.body}<p class="text-sm text-slate-300 whitespace-pre-line">{r.body}</p>{/if}
          <div class="flex gap-2 pt-1">
            {#if r.status !== 'approved'}
              <button type="button" on:click={() => moderate(r, 'approved')} class="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-bold flex items-center gap-1"><Check size={13} /> Approve</button>
            {/if}
            {#if r.status !== 'rejected'}
              <button type="button" on:click={() => moderate(r, 'rejected')} class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-white text-xs font-bold flex items-center gap-1"><X size={13} /> Reject</button>
            {/if}
            <button type="button" on:click={() => remove(r)} class="px-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 hover:border-rose-500/40 text-slate-400 hover:text-rose-300 text-xs font-bold flex items-center gap-1"><Trash2 size={13} /> Delete</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
