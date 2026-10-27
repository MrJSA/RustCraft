<script>
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { Mail, Send, Trash2, AlertCircle, CheckCircle2, Users, FlaskConical } from 'lucide-svelte';

  let subscribers = [];
  let campaigns = [];
  let error = '';
  let notice = '';
  let subject = '';
  let body = '';
  let testTo = $page.data.admin?.email || '';
  let busy = false;
  let pollTimer;

  $: enabled = $page.data.settings?.newsletter_enabled;
  $: confirmed = subscribers.filter((s) => s.status === 'confirmed');
  $: pending = subscribers.filter((s) => s.status === 'pending');

  async function api(path, options = {}) {
    const res = await fetch(`/api/v1/admin/newsletter/${path}`, { ...options, headers: { 'Content-Type': 'application/json' } });
    const data = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(data.error || `Request failed (${res.status})`);
    return data;
  }

  async function load() {
    try {
      [subscribers, campaigns] = await Promise.all([api('subscribers'), api('campaigns')]);
      clearInterval(pollTimer);
      if (campaigns.some((c) => c.status === 'sending')) pollTimer = setInterval(load, 3000);
    } catch (e) {
      error = e.message;
    }
  }

  async function sendTest() {
    error = notice = '';
    busy = true;
    try {
      notice = (await api('test', { method: 'POST', body: JSON.stringify({ subject, body_markdown: body, to: testTo }) })).message;
    } catch (e) {
      error = e.message;
    } finally {
      busy = false;
    }
  }

  async function sendNow() {
    error = notice = '';
    if (!confirm(`Send "${subject}" to ${confirmed.length} confirmed subscriber(s) now? This cannot be undone.`)) return;
    busy = true;
    try {
      const { id } = await api('campaigns', { method: 'POST', body: JSON.stringify({ subject, body_markdown: body }) });
      const r = await api(`campaigns/${id}/send`, { method: 'POST' });
      notice = `Sending to ${r.recipients} subscriber(s)…`;
      subject = '';
      body = '';
      await load();
    } catch (e) {
      error = e.message;
    } finally {
      busy = false;
    }
  }

  async function removeSubscriber(s) {
    if (!confirm(`Delete ${s.email} and the proof of consent permanently? (Use only on a deletion request — an unsubscribed entry already stops all mailings.)`)) return;
    try {
      await api(`subscribers/${s.id}`, { method: 'DELETE' });
      await load();
    } catch (e) {
      error = e.message;
    }
  }

  const fmt = (d) => (d ? new Date(d).toLocaleString() : '—');
  onMount(load);
  onDestroy(() => clearInterval(pollTimer));
  const input = 'w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500';
</script>

<div class="space-y-6">
  <div>
    <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5"><Mail size={24} class="text-orange-500" /> Newsletter</h1>
    <p class="text-xs text-slate-400 mt-1">Only subscribers who confirmed their address (double opt-in) receive newsletters. Every newsletter contains an unsubscribe link.</p>
  </div>

  {#if !enabled}
    <div class="p-3.5 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs">The newsletter is switched off — the sign-up form is hidden. Turn it on in <a href="/settings?tab=features" class="underline">Settings → Shop Features & Legal</a>.</div>
  {/if}
  {#if error}<div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-2"><AlertCircle size={15} /> {error}</div>{/if}
  {#if notice}<div class="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs flex items-center gap-2"><CheckCircle2 size={15} /> {notice}</div>{/if}

  <section class="p-6 rounded-2xl bg-slate-900 border border-slate-800 space-y-3">
    <h2 class="text-sm font-bold text-white">Write a newsletter</h2>
    <input bind:value={subject} maxlength="200" placeholder="Subject" class={input} />
    <textarea bind:value={body} rows="10" placeholder={'## Headline\n\nText with **bold**, *italic* and [links](https://example.com).\n\n- list item\n- list item'} class="{input} font-mono"></textarea>
    <p class="text-[11px] text-slate-500">Simple formatting: <code>## Headline</code>, <code>**bold**</code>, <code>*italic*</code>, <code>[text](https://link)</code>, lines starting with <code>- </code> become a list. The shop layout, logo and unsubscribe link are added automatically.</p>
    <div class="flex flex-col sm:flex-row gap-2">
      <input bind:value={testTo} type="email" placeholder="Test recipient" class="{input} sm:max-w-xs" />
      <button type="button" on:click={sendTest} disabled={busy || !subject.trim() || !body.trim()} class="px-4 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2"><FlaskConical size={14} /> Send test</button>
      <button type="button" on:click={sendNow} disabled={busy || !enabled || !subject.trim() || !body.trim() || confirmed.length === 0} class="sm:ml-auto px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2"><Send size={14} /> Send to {confirmed.length} subscriber(s)</button>
    </div>
  </section>

  {#if campaigns.length > 0}
    <section class="p-6 rounded-2xl bg-slate-900 border border-slate-800 space-y-2">
      <h2 class="text-sm font-bold text-white">Sent newsletters</h2>
      {#each campaigns as c}
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-1 py-2 border-t border-slate-800 text-xs">
          <span class="text-white font-semibold">{c.subject}</span>
          <span class="text-slate-400">
            {c.status === 'sending' ? 'Sending…' : c.status === 'sent' ? `Sent ${fmt(c.sent_at)}` : 'Draft'} · {c.sent_count}/{c.recipients} delivered{#if c.failed_count} · <span class="text-rose-400">{c.failed_count} failed</span>{/if}
          </span>
        </div>
      {/each}
    </section>
  {/if}

  <section class="p-6 rounded-2xl bg-slate-900 border border-slate-800 space-y-3">
    <h2 class="text-sm font-bold text-white flex items-center gap-2"><Users size={15} /> Subscribers · {confirmed.length} confirmed · {pending.length} waiting for confirmation</h2>
    <p class="text-[11px] text-slate-500">Proof of consent: time and IP address of the sign-up and of the confirmation. Unconfirmed sign-ups are deleted after 30 days.</p>
    {#if subscribers.length === 0}
      <p class="text-xs text-slate-400">No subscribers yet.</p>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-xs">
          <thead class="text-left text-slate-500">
            <tr><th class="py-2 pr-3">Email</th><th class="pr-3">Status</th><th class="pr-3">Signed up</th><th class="pr-3">Confirmed</th><th class="pr-3">Source</th><th></th></tr>
          </thead>
          <tbody>
            {#each subscribers as s}
              <tr class="border-t border-slate-800 text-slate-300">
                <td class="py-2 pr-3 font-mono">{s.email}</td>
                <td class="pr-3"><span class="px-2 py-0.5 rounded-full text-[10px] font-bold {s.status === 'confirmed' ? 'bg-emerald-500/15 text-emerald-400' : s.status === 'pending' ? 'bg-amber-500/15 text-amber-300' : 'bg-slate-800 text-slate-400'}">{s.status}</span></td>
                <td class="pr-3">{fmt(s.requested_at)}{#if s.requested_ip}<span class="block text-[10px] text-slate-500">{s.requested_ip}</span>{/if}</td>
                <td class="pr-3">{fmt(s.confirmed_at)}{#if s.confirmed_ip}<span class="block text-[10px] text-slate-500">{s.confirmed_ip}</span>{/if}</td>
                <td class="pr-3">{s.source}</td>
                <td class="text-right"><button type="button" on:click={() => removeSubscriber(s)} class="p-1.5 rounded-lg text-slate-500 hover:text-rose-400" title="Delete"><Trash2 size={13} /></button></td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>
</div>
