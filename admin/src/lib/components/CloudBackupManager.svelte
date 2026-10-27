<script>
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { CloudUpload, CheckCircle2, AlertCircle, Loader2, ExternalLink, PlugZap, Unplug, HelpCircle, Clock } from 'lucide-svelte';

  $: isSuperadmin = $page.data.admin?.role === 'superadmin';

  let loaded = false;
  let status = null;
  let error = '';
  let notice = '';
  let saving = false;
  let testing = false;
  let showHelp = false;
  let pollTimer;

  // Form
  let enabled = false;
  let repo = '';
  let token = '';
  let frequency = 'daily';
  let hourUtc = 2;
  let keepCount = 30;
  let includeMedia = true;

  const frequencies = [
    { value: 'daily', label: 'Daily', hint: 'every night' },
    { value: 'weekly', label: 'Weekly', hint: 'every Sunday' },
    { value: 'monthly', label: 'Monthly', hint: 'on the 1st of the month' }
  ];

  // Options are stored in UTC but shown in the admin's own local time
  const hourOptions = Array.from({ length: 24 }, (_, utc) => {
    const d = new Date(Date.UTC(2026, 0, 15, utc, 0, 0));
    return { utc, local: d.getHours(), label: d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) };
  }).sort((a, b) => a.local - b.local);

  async function api(path, options = {}) {
    const res = await fetch(`/api/v1/admin/backup/${path}`, {
      ...options,
      headers: { 'Content-Type': 'application/json', ...(options.headers || {}) }
    });
    const body = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(body.error || `Request failed (${res.status})`);
    return body;
  }

  function applyStatus(s, resetForm = false) {
    status = s;
    if (resetForm) {
      enabled = s.enabled;
      repo = s.github_repo || '';
      frequency = s.frequency || 'daily';
      hourUtc = s.backup_hour_utc ?? 2;
      keepCount = s.keep_count ?? 30;
      includeMedia = s.include_media ?? true;
    }
    if (s.running) startPolling();
  }

  async function load() {
    try {
      applyStatus(await api('settings'), true);
    } catch (e) {
      error = e.message;
    } finally {
      loaded = true;
    }
  }

  async function testConnection() {
    testing = true;
    error = '';
    notice = '';
    try {
      const res = await api('test', { method: 'POST', body: JSON.stringify({ github_repo: repo, github_token: token }) });
      repo = res.github_repo;
      notice = res.message;
    } catch (e) {
      error = e.message;
    } finally {
      testing = false;
    }
  }

  async function save() {
    saving = true;
    error = '';
    notice = '';
    try {
      const s = await api('settings', {
        method: 'PUT',
        body: JSON.stringify({
          enabled,
          github_repo: repo,
          github_token: token,
          frequency,
          backup_hour_utc: Number(hourUtc),
          keep_count: Number(keepCount),
          include_media: includeMedia
        })
      });
      token = '';
      applyStatus(s, true);
      notice = s.running
        ? 'Saved. The first backup is being created now — this can take a few minutes.'
        : enabled
          ? 'Saved. Automatic backups are switched on.'
          : 'Saved. Automatic backups are switched off.';
    } catch (e) {
      error = e.message;
    } finally {
      saving = false;
    }
  }

  async function runNow() {
    error = '';
    notice = '';
    try {
      await api('run', { method: 'POST' });
      status = { ...status, running: true };
      startPolling();
    } catch (e) {
      error = e.message;
    }
  }

  async function disconnect() {
    if (!confirm('Disconnect the GitHub repository? Automatic backups stop and the saved token is deleted. Backups already stored on GitHub are kept.')) return;
    error = '';
    notice = '';
    try {
      applyStatus(await api('disconnect', { method: 'POST' }), true);
      notice = 'Disconnected. The token was deleted from the shop.';
    } catch (e) {
      error = e.message;
    }
  }

  function startPolling() {
    if (pollTimer) return;
    pollTimer = setInterval(async () => {
      try {
        const s = await api('settings');
        status = s;
        if (!s.running) {
          clearInterval(pollTimer);
          pollTimer = null;
        }
      } catch (_) {
        // keep polling
      }
    }, 3000);
  }

  function fmtDate(value) {
    return value ? new Date(value).toLocaleString([], { dateStyle: 'medium', timeStyle: 'short' }) : '—';
  }

  function fmtSize(bytes) {
    if (!bytes) return '';
    if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
    if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
    return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  }

  $: running = !!status?.running;
  $: tokenExpiresSoon = status?.token_expires_at && new Date(status.token_expires_at.replace(' UTC', 'Z').replace(' ', 'T')) - Date.now() < 14 * 864e5;

  onMount(() => {
    if (isSuperadmin) load();
  });
  onDestroy(() => clearInterval(pollTimer));

  const inputClass = 'w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs font-mono focus:outline-none focus:border-orange-500';
</script>

<section class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-5">
  <div class="flex flex-col sm:flex-row sm:items-start justify-between gap-3">
    <div class="flex items-start gap-3">
      <div class="w-12 h-12 rounded-2xl bg-sky-600/15 text-sky-400 border border-sky-500/20 flex items-center justify-center flex-shrink-0">
        <CloudUpload size={24} />
      </div>
      <div>
        <h3 class="text-base font-bold text-white">Automatic Backups to GitHub</h3>
        <p class="text-xs text-slate-400 mt-1 leading-relaxed max-w-2xl">
          Saves a complete copy of the shop — the whole database (orders, customers, products, settings) and all uploaded files —
          to your own <strong class="text-slate-200">private</strong> GitHub repository. If the server ever fails, a new server can be restored from it.
        </p>
      </div>
    </div>
    {#if status}
      <span class="self-start text-[10px] font-bold px-2.5 py-1 rounded-full border whitespace-nowrap {status.enabled ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30' : 'bg-slate-800 text-slate-400 border-slate-700'}">
        {status.enabled ? 'Automatic backups ON' : 'Automatic backups OFF'}
      </span>
    {/if}
  </div>

  {#if !isSuperadmin}
    <div class="p-4 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs">Only superadmins can manage backups.</div>
  {:else if !loaded}
    <div class="text-xs text-slate-400 flex items-center gap-2"><Loader2 size={14} class="animate-spin" /> Loading backup settings…</div>
  {:else}
    {#if error}
      <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-start gap-2">
        <AlertCircle size={16} class="flex-shrink-0" /> <span>{error}</span>
      </div>
    {/if}
    {#if notice}
      <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-start gap-2">
        <CheckCircle2 size={16} class="flex-shrink-0" /> <span>{notice}</span>
      </div>
    {/if}

    <!-- Status -->
    {#if status?.has_token || status?.last_status}
      <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs">
        <div>
          <div class="text-[10px] uppercase tracking-wider font-bold text-slate-500">Last backup</div>
          {#if running}
            <div class="mt-1 font-bold text-amber-300 flex items-center gap-1.5"><Loader2 size={13} class="animate-spin" /> Creating backup…</div>
          {:else if status.last_status === 'failed'}
            <div class="mt-1 font-bold text-rose-400 flex items-center gap-1.5"><AlertCircle size={13} /> Failed {fmtDate(status.last_attempt_at)}</div>
          {:else if status.last_success_at}
            <div class="mt-1 font-bold text-emerald-400 flex items-center gap-1.5"><CheckCircle2 size={13} /> {fmtDate(status.last_success_at)}</div>
          {:else}
            <div class="mt-1 text-slate-400">No backup yet</div>
          {/if}
          {#if status.last_success_at && status.last_status === 'failed'}
            <div class="text-[11px] text-slate-500 mt-0.5">Last successful: {fmtDate(status.last_success_at)}</div>
          {/if}
        </div>
        <div>
          <div class="text-[10px] uppercase tracking-wider font-bold text-slate-500">Next automatic backup</div>
          <div class="mt-1 text-slate-200 flex items-center gap-1.5"><Clock size={13} class="text-slate-500" /> {status.enabled ? fmtDate(status.next_run_at) : 'switched off'}</div>
        </div>
        <div class="flex sm:justify-end items-start gap-2 flex-wrap">
          <button type="button" on:click={runNow} disabled={running || !status.has_token} class="px-3.5 py-2 rounded-xl bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-1.5">
            <CloudUpload size={14} /> Back up now
          </button>
          {#if status.releases_url}
            <a href={status.releases_url} target="_blank" rel="noopener noreferrer" class="px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-bold flex items-center gap-1.5">
              <ExternalLink size={14} /> View backups
            </a>
          {/if}
        </div>
        {#if status.last_message && !running}
          <div class="sm:col-span-3 text-[11px] leading-relaxed {status.last_status === 'failed' ? 'text-rose-300' : 'text-slate-400'}">
            {status.last_message}
            {#if status.last_status === 'success' && status.last_url}
              <a href={status.last_url} target="_blank" rel="noopener noreferrer" class="text-sky-400 hover:underline ml-1">Open on GitHub</a>
            {/if}
          </div>
        {/if}
        {#if tokenExpiresSoon}
          <div class="sm:col-span-3 p-2.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-300 text-[11px]">
            The GitHub token expires on {status.token_expires_at}. Create a new token and paste it below, otherwise backups will stop.
          </div>
        {/if}
      </div>
    {/if}

    <!-- Setup help -->
    <div class="rounded-2xl border border-slate-800">
      <button type="button" on:click={() => (showHelp = !showHelp)} class="w-full px-4 py-3 flex items-center gap-2 text-xs font-bold text-slate-300 hover:text-white">
        <HelpCircle size={15} class="text-sky-400" /> How do I set this up? (about 5 minutes)
      </button>
      {#if showHelp}
        <ol class="px-5 pb-4 space-y-2.5 text-xs text-slate-400 list-decimal list-inside leading-relaxed">
          <li>
            Create a new, empty repository on GitHub:
            <a href="https://github.com/new" target="_blank" rel="noopener noreferrer" class="text-sky-400 hover:underline">github.com/new</a>
            — e.g. named <code class="text-slate-200">shop-backups</code>, visibility <strong class="text-slate-200">Private</strong>.
          </li>
          <li>
            Create an access token:
            <a href="https://github.com/settings/personal-access-tokens/new" target="_blank" rel="noopener noreferrer" class="text-sky-400 hover:underline">github.com/settings/personal-access-tokens/new</a>
            <ul class="list-disc list-inside ml-4 mt-1 space-y-0.5">
              <li>Expiration: the longest available (or <em>No expiration</em>)</li>
              <li>Repository access: <em>Only select repositories</em> → your backup repository</li>
              <li>Permissions → Repository permissions → <strong class="text-slate-200">Contents: Read and write</strong></li>
            </ul>
          </li>
          <li>Copy the token (starts with <code class="text-slate-200">github_pat_</code>), paste it below together with the repository name, click <em>Test connection</em> and then <em>Save</em>.</li>
        </ol>
        <p class="px-5 pb-4 text-[11px] text-slate-500">The full guide including how to restore a backup is in <code>BACKUP.md</code> of the shop software.</p>
      {/if}
    </div>

    <!-- Connection -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
      <div>
        <label for="backup-repo" class="block font-semibold text-slate-400 mb-1">GitHub repository</label>
        <input id="backup-repo" type="text" bind:value={repo} placeholder="my-account/shop-backups" autocomplete="off" class={inputClass} />
        <p class="text-[11px] text-slate-500 mt-1">Name as <code>owner/repository</code> or simply paste the link to the repository.</p>
      </div>
      <div>
        <label for="backup-token" class="block font-semibold text-slate-400 mb-1">Access token</label>
        <input
          id="backup-token"
          type="password"
          bind:value={token}
          placeholder={status?.has_token ? `saved (${status.token_hint || '••••'}) — leave empty to keep` : 'github_pat_…'}
          autocomplete="new-password"
          class={inputClass}
        />
        <p class="text-[11px] text-slate-500 mt-1">Stored only in the shop's database and never shown again.</p>
      </div>
    </div>
    <div class="flex flex-wrap gap-2">
      <button type="button" on:click={testConnection} disabled={testing || !repo.trim() || (!token.trim() && !status?.has_token)} class="px-4 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2">
        {#if testing}<Loader2 size={14} class="animate-spin" />{:else}<PlugZap size={14} />{/if} Test connection
      </button>
      {#if status?.has_token}
        <button type="button" on:click={disconnect} class="px-4 py-2.5 rounded-xl bg-slate-950 hover:bg-rose-500/10 border border-slate-800 hover:border-rose-500/30 text-slate-400 hover:text-rose-300 text-xs font-bold flex items-center gap-2">
          <Unplug size={14} /> Disconnect
        </button>
      {/if}
    </div>

    <!-- Schedule -->
    <div class="space-y-2">
      <div class="text-xs font-semibold text-slate-400">How often?</div>
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
        {#each frequencies as f}
          <label class="p-3.5 rounded-xl border cursor-pointer transition-all {frequency === f.value ? 'bg-sky-500/10 border-sky-500/50' : 'bg-slate-950 border-slate-800 hover:border-slate-700'}">
            <input type="radio" bind:group={frequency} value={f.value} class="sr-only" />
            <div class="text-sm font-bold {frequency === f.value ? 'text-white' : 'text-slate-300'}">{f.label}</div>
            <div class="text-[11px] text-slate-500">{f.hint}</div>
          </label>
        {/each}
      </div>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
      <div>
        <label for="backup-hour" class="block font-semibold text-slate-400 mb-1">At what time?</label>
        <select id="backup-hour" bind:value={hourUtc} class={inputClass}>
          {#each hourOptions as h}
            <option value={h.utc}>{h.label}</option>
          {/each}
        </select>
        <p class="text-[11px] text-slate-500 mt-1">Your local time. Best at night when few customers are shopping.</p>
      </div>
      <div>
        <label for="backup-keep" class="block font-semibold text-slate-400 mb-1">How many backups to keep?</label>
        <input id="backup-keep" type="number" min="1" max="365" bind:value={keepCount} class={inputClass} />
        <p class="text-[11px] text-slate-500 mt-1">Older backups are deleted automatically (e.g. 30 daily backups = one month of history).</p>
      </div>
    </div>

    <label class="flex items-start gap-2.5 text-xs text-slate-300 cursor-pointer">
      <input type="checkbox" bind:checked={includeMedia} class="w-4 h-4 mt-0.5 accent-sky-600" />
      <span>Include uploaded files (product images, logos, digital downloads). <span class="text-slate-500">Recommended — without them a restored shop has no pictures.</span></span>
    </label>

    <div class="pt-4 border-t border-slate-800 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
      <label class="flex items-center gap-2.5 text-sm font-bold text-white cursor-pointer">
        <input type="checkbox" bind:checked={enabled} class="w-5 h-5 accent-emerald-600" />
        Automatic backups switched on
      </label>
      <button type="button" on:click={save} disabled={saving} class="px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2">
        {#if saving}<Loader2 size={14} class="animate-spin" /> Checking & saving…{:else}Save backup settings{/if}
      </button>
    </div>
  {/if}
</section>
