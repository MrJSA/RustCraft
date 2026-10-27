<script>
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { RefreshCw, DownloadCloud, Globe, CheckCircle2, AlertCircle, Loader2, Server } from 'lucide-svelte';

  $: isSuperadmin = $page.data.admin?.role === 'superadmin';

  let status = null; // { current_version, current_commit, job }
  let check = null; // { latest_version, update_available, release_notes }
  let job = null;
  let error = '';
  let busy = false;
  let pollTimer;

  let shopDomain = '';
  let adminDomain = '';
  let httpsProxy = true;
  let domainError = '';
  let domainNotice = '';

  async function api(path, options = {}) {
    const res = await fetch(`/api/v1/admin/system/${path}`, {
      ...options,
      headers: { 'Content-Type': 'application/json', ...(options.headers || {}) }
    });
    const body = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(body.error || `Request failed (${res.status})`);
    return body;
  }

  async function loadStatus() {
    try {
      status = await api('status');
      job = status.job;
      if (job?.running) startPolling();
      const cfg = await api('domains');
      shopDomain = cfg.shop_domain || '';
      adminDomain = cfg.admin_domain || '';
      httpsProxy = cfg.https_proxy ?? true;
    } catch (e) {
      error = e.message;
    }
  }

  async function checkForUpdates() {
    busy = true;
    error = '';
    try {
      check = await api('check', { method: 'POST' });
    } catch (e) {
      error = e.message;
    } finally {
      busy = false;
    }
  }

  async function startUpdate() {
    if (!confirm(`Update the shop to version ${check.latest_version}? A database backup is made first. The shop restarts during the update (usually a few minutes).`)) return;
    error = '';
    try {
      await api('update', { method: 'POST' });
      startPolling();
    } catch (e) {
      error = e.message;
    }
  }

  async function applyDomains() {
    domainError = '';
    domainNotice = '';
    if (!confirm('Apply the new domains? The shop and admin restart and the admin will afterwards be reachable at the new admin domain.')) return;
    try {
      await api('domains', {
        method: 'PUT',
        body: JSON.stringify({ shop_domain: shopDomain.trim(), admin_domain: adminDomain.trim(), https_proxy: httpsProxy })
      });
      domainNotice = 'Applying… the admin will restart. Afterwards open it at ' + (httpsProxy ? 'https://' : 'http://') + adminDomain.trim();
      startPolling();
    } catch (e) {
      domainError = e.message;
    }
  }

  function startPolling() {
    clearInterval(pollTimer);
    pollTimer = setInterval(async () => {
      try {
        job = await api('job');
        if (!job.running) {
          clearInterval(pollTimer);
          status = await api('status');
          check = null;
        }
      } catch (_) {
        // The admin/backend restarts during an update — keep polling until it is back
      }
    }, 2000);
  }

  onMount(() => {
    if (isSuperadmin) loadStatus();
  });
  onDestroy(() => clearInterval(pollTimer));

  const inputClass = 'w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs font-mono focus:outline-none focus:border-orange-500';
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5">
      <Server size={22} class="text-orange-500" /> System, Updates & Domains
    </h2>
    <p class="text-xs text-slate-400 mt-1">Keep the shop up to date with the latest release from GitHub and choose the domains it runs on.</p>
  </div>

  {#if !isSuperadmin}
    <div class="p-4 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs">Only superadmins can update the shop or change its domains.</div>
  {:else}
    {#if error}
      <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
        <AlertCircle size={16} /> <span>{error}</span>
      </div>
    {/if}

    <!-- Version & updates -->
    <section class="p-6 rounded-3xl bg-slate-900 border border-slate-800 space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <div class="text-[11px] text-slate-500 uppercase tracking-wider font-bold">Installed version</div>
          <div class="text-2xl font-black text-white font-mono">{status?.current_version ? `v${status.current_version}` : '—'}</div>
          {#if status?.current_commit}<div class="text-[10px] text-slate-500 font-mono">commit {status.current_commit}</div>{/if}
        </div>
        <button type="button" on:click={checkForUpdates} disabled={busy || job?.running} class="px-4 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2">
          <RefreshCw size={14} class={busy ? 'animate-spin' : ''} /> Check for updates
        </button>
      </div>

      {#if check}
        {#if check.update_available}
          <div class="p-4 rounded-2xl bg-emerald-500/10 border border-emerald-500/30 space-y-3">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div class="text-sm text-emerald-300 font-bold">Version v{check.latest_version} is available</div>
              <button type="button" on:click={startUpdate} disabled={job?.running} class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2">
                <DownloadCloud size={15} /> Update now
              </button>
            </div>
            {#if check.release_notes}
              <pre class="text-[11px] text-slate-300 whitespace-pre-wrap font-sans">{check.release_notes}</pre>
            {/if}
          </div>
        {:else}
          <div class="p-3 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-300 flex items-center gap-2">
            <CheckCircle2 size={15} class="text-emerald-400" /> The shop is up to date{check.latest_version ? ` (latest release v${check.latest_version})` : ' — no releases published yet'}.
          </div>
        {/if}
      {/if}

      {#if job && job.kind}
        <div class="space-y-2">
          <div class="text-xs font-bold flex items-center gap-2 {job.running ? 'text-amber-300' : job.success ? 'text-emerald-400' : 'text-rose-400'}">
            {#if job.running}<Loader2 size={14} class="animate-spin" /> Running: {job.kind}…
            {:else if job.success}<CheckCircle2 size={14} /> Last task ({job.kind}) finished successfully
            {:else}<AlertCircle size={14} /> Last task ({job.kind}) failed{/if}
          </div>
          <pre class="max-h-72 overflow-auto p-3 rounded-xl bg-black/60 border border-slate-800 text-[10px] leading-relaxed text-slate-300 font-mono">{(job.log || []).join('\n')}</pre>
        </div>
      {/if}
    </section>

    <!-- Domains -->
    <section class="p-6 rounded-3xl bg-slate-900 border border-slate-800 space-y-4">
      <div>
        <h3 class="text-sm font-bold text-white flex items-center gap-2"><Globe size={16} class="text-orange-400" /> Domains / subdomains</h3>
        <p class="text-[11px] text-slate-400 mt-1 leading-relaxed">
          Point the DNS A/AAAA records of both (sub)domains to this server first. With the built-in HTTPS proxy, ports 80 and 443 must be free
          and certificates are created automatically. If your server already runs nginx, Apache, Traefik or Caddy, turn the built-in proxy off and
          forward the domains to ports 8080 (shop) and 4000 (admin) as described in INSTALL.md.
        </p>
      </div>
      {#if domainError}<div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs">{domainError}</div>{/if}
      {#if domainNotice}<div class="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs">{domainNotice}</div>{/if}
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
        <div>
          <label for="shop-domain" class="block font-semibold text-slate-400 mb-1">Shop domain</label>
          <input id="shop-domain" type="text" bind:value={shopDomain} placeholder="example.com or shop.example.com" class={inputClass} />
        </div>
        <div>
          <label for="admin-domain" class="block font-semibold text-slate-400 mb-1">Admin domain</label>
          <input id="admin-domain" type="text" bind:value={adminDomain} placeholder="admin.example.com" class={inputClass} />
        </div>
      </div>
      <label class="flex items-center gap-2.5 text-xs text-slate-300 cursor-pointer">
        <input type="checkbox" bind:checked={httpsProxy} class="w-4 h-4 accent-orange-600" />
        Use the built-in HTTPS proxy (automatic Let's Encrypt certificates)
      </label>
      <div class="flex justify-end">
        <button type="button" on:click={applyDomains} disabled={job?.running || !shopDomain.trim() || !adminDomain.trim()} class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-xs font-bold">
          Apply domains
        </button>
      </div>
    </section>
  {/if}
</div>
