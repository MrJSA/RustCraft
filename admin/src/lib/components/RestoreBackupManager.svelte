<script>
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { RotateCcw, Search, Upload, CheckCircle2, AlertCircle, ShieldAlert, Loader2 } from 'lucide-svelte';

  $: isSuperadmin = $page.data.admin?.role === 'superadmin';

  // Uploads go through the admin server, which accepts requests up to 50 MB
  const MAX_UPLOAD_BYTES = 50 * 1024 * 1024;

  let hasSavedToken = false;
  let repo = '';
  let token = '';
  let listing = false;
  let backups = null;
  let selected = null;
  let uploadFile = null;
  let fileInput;
  let keepBackupsOff = false;
  let understood = false;
  let error = '';

  // Progress of a running restore
  let restore = null;
  let restarting = false;
  let pollTimer;

  async function api(path, options = {}) {
    const res = await fetch(`/api/v1/admin/backup/${path}`, {
      ...options,
      headers: { 'Content-Type': 'application/json', ...(options.headers || {}) }
    });
    const body = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(body.error || `Request failed (${res.status})`);
    return body;
  }

  async function load() {
    try {
      const s = await api('settings');
      repo = s.github_repo || '';
      hasSavedToken = !!s.has_token;
      const r = await api('restore/status');
      if (r.state === 'running') {
        restore = r;
        startPolling();
      }
    } catch (_) {
      // The backup box above shows loading errors
    }
  }

  async function listBackups() {
    listing = true;
    error = '';
    backups = null;
    selected = null;
    try {
      const res = await api('restore/list', { method: 'POST', body: JSON.stringify({ github_repo: repo, github_token: token }) });
      repo = res.github_repo;
      backups = res.backups;
      selected = backups[0] || null;
    } catch (e) {
      error = e.message;
    } finally {
      listing = false;
    }
  }

  function pickFile(event) {
    error = '';
    uploadFile = event.target.files?.[0] || null;
    if (uploadFile && uploadFile.size > MAX_UPLOAD_BYTES) {
      error = `This file is ${fmtSize(uploadFile.size)} — files up to 50 MB can be uploaded here. Restore larger backups straight from GitHub above.`;
      uploadFile = null;
      fileInput.value = '';
    }
  }

  function confirmRestore(name) {
    return confirm(
      `Restore "${name}"?\n\nALL current data of this shop (orders, customers, products, settings, admin accounts) is replaced by the backup. This cannot be undone.`
    );
  }

  async function restoreFromGitHub() {
    if (!selected || !confirmRestore(selected.title || selected.file)) return;
    error = '';
    try {
      restore = await api('restore/github', {
        method: 'POST',
        body: JSON.stringify({
          github_repo: repo,
          github_token: token,
          asset_id: selected.asset_id,
          file: selected.file,
          keep_backups_off: keepBackupsOff
        })
      });
      startPolling();
    } catch (e) {
      error = e.message;
    }
  }

  async function restoreFromUpload() {
    if (!uploadFile || !confirmRestore(uploadFile.name)) return;
    error = '';
    restore = { state: 'running', step: 'Uploading the backup…', file: uploadFile.name };
    try {
      const query = new URLSearchParams({ file: uploadFile.name, keep_backups_off: String(keepBackupsOff) });
      restore = await api(`restore/upload?${query}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/zip' },
        body: uploadFile
      });
      startPolling();
    } catch (e) {
      restore = null;
      error = e.message;
    }
  }

  function startPolling() {
    if (pollTimer) return;
    pollTimer = setInterval(poll, 2000);
  }

  function stopPolling() {
    clearInterval(pollTimer);
    pollTimer = null;
  }

  async function poll() {
    let res;
    try {
      // While the database is being replaced, requests wait for it; don't stack them up
      res = await fetch('/api/v1/admin/backup/restore/status', { signal: AbortSignal.timeout(10000) });
    } catch (_) {
      return;
    }
    // The restored database has its own admin accounts, so this session ends with the restore
    if (res.status === 401 || res.status === 403) {
      finished();
      return;
    }
    if (!res.ok) {
      // 502 while the backend restarts after a successful restore
      if (restore?.state === 'success') restarting = true;
      return;
    }
    const s = await res.json().catch(() => null);
    if (!s) return;
    if (restarting && s.state === 'idle') {
      finished();
      return;
    }
    restore = s;
    if (s.state === 'failed') stopPolling();
  }

  function finished() {
    stopPolling();
    restore = { ...(restore || {}), state: 'success', step: '' };
    restarting = true;
    setTimeout(goToLogin, 8000);
  }

  async function goToLogin() {
    await fetch('/api/v1/admin/auth/logout', { method: 'POST' }).catch(() => {});
    window.location.href = '/login';
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

  $: busy = restore?.state === 'running' || restore?.state === 'success';

  onMount(() => {
    if (isSuperadmin) load();
  });
  onDestroy(stopPolling);

  const inputClass = 'w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs font-mono focus:outline-none focus:border-orange-500';
</script>

{#if isSuperadmin}
  <section class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-5">
    <div class="flex items-start gap-3">
      <div class="w-12 h-12 rounded-2xl bg-amber-600/15 text-amber-400 border border-amber-500/20 flex items-center justify-center flex-shrink-0">
        <RotateCcw size={24} />
      </div>
      <div>
        <h3 class="text-base font-bold text-white">Restore a Backup</h3>
        <p class="text-xs text-slate-400 mt-1 leading-relaxed max-w-2xl">
          Brings the shop back to the state of a backup — after a server crash, on a new server or to undo a mistake.
          Pick a backup from your GitHub repository or upload a backup ZIP file.
        </p>
      </div>
    </div>

    {#if restore}
      <!-- Progress -->
      <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-2 text-xs">
        <div class="text-[10px] uppercase tracking-wider font-bold text-slate-500">Restoring {restore.file || 'backup'}</div>
        {#if restore.state === 'running'}
          <div class="font-bold text-amber-300 flex items-center gap-1.5"><Loader2 size={14} class="animate-spin" /> {restore.step}</div>
          <p class="text-[11px] text-slate-500">Keep this page open. The shop may not respond for a moment while the database is replaced.</p>
        {:else if restore.state === 'failed'}
          <div class="font-bold text-rose-400 flex items-start gap-1.5"><AlertCircle size={14} class="flex-shrink-0 mt-0.5" /> <span>{restore.message}</span></div>
          <p class="text-[11px] text-slate-500">The shop's data was not changed.</p>
          <button type="button" on:click={() => (restore = null)} class="px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-bold">Back</button>
        {:else}
          <div class="font-bold text-emerald-400 flex items-start gap-1.5"><CheckCircle2 size={14} class="flex-shrink-0 mt-0.5" /> <span>{restore.message || 'Backup restored.'}</span></div>
          <p class="text-[11px] text-slate-300 leading-relaxed">
            {#if restarting}The shop is restarting.{:else}The shop restarts now.{/if}
            Log in again with the admin username and password <strong>of the restored shop</strong> (the accounts came back with the backup).
          </p>
          <button type="button" on:click={goToLogin} class="px-3.5 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-bold">Go to login</button>
        {/if}
      </div>
    {/if}

    {#if !busy}
      {#if error}
        <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-start gap-2">
          <AlertCircle size={16} class="flex-shrink-0" /> <span>{error}</span>
        </div>
      {/if}

      <div class="p-4 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-200 text-xs leading-relaxed flex items-start gap-2.5">
        <ShieldAlert size={16} class="flex-shrink-0 mt-0.5 text-amber-400" />
        <span>
          Restoring <strong>replaces all current data</strong> — orders, customers, products, settings and admin accounts — with the content of the backup.
          Anything that happened after the backup is lost. If this shop holds data you still need, click <em>Back up now</em> above first.
        </span>
      </div>

      <!-- From GitHub -->
      <div class="space-y-3">
        <div class="text-xs font-bold text-slate-300">1. From your GitHub backup repository</div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
          <div>
            <label for="restore-repo" class="block font-semibold text-slate-400 mb-1">GitHub repository</label>
            <input id="restore-repo" type="text" bind:value={repo} placeholder="my-account/shop-backups" autocomplete="off" class={inputClass} />
          </div>
          <div>
            <label for="restore-token" class="block font-semibold text-slate-400 mb-1">Access token</label>
            <input
              id="restore-token"
              type="password"
              bind:value={token}
              placeholder={hasSavedToken ? 'saved token is used — leave empty' : 'github_pat_…'}
              autocomplete="new-password"
              class={inputClass}
            />
            <p class="text-[11px] text-slate-500 mt-1">On a newly installed shop, use the token of your backup repository (it is not saved).</p>
          </div>
        </div>
        <button type="button" on:click={listBackups} disabled={listing || !repo.trim() || (!token.trim() && !hasSavedToken)} class="px-4 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2">
          {#if listing}<Loader2 size={14} class="animate-spin" />{:else}<Search size={14} />{/if} Show backups
        </button>

        {#if backups}
          {#if backups.length === 0}
            <p class="text-xs text-slate-400">This repository contains no backups yet.</p>
          {:else}
            <div class="max-h-72 overflow-y-auto rounded-xl border border-slate-800 divide-y divide-slate-800">
              {#each backups as b}
                <label class="flex items-center gap-3 px-3.5 py-2.5 text-xs cursor-pointer {selected === b ? 'bg-sky-500/10' : 'hover:bg-slate-800/50'}">
                  <input type="radio" bind:group={selected} value={b} class="accent-sky-600" />
                  <span class="flex-1 min-w-0">
                    <span class="block font-semibold text-slate-200 truncate">{b.title || b.file}</span>
                    <span class="block text-[11px] text-slate-500 truncate">{fmtDate(b.created_at)} · {b.file}</span>
                  </span>
                  <span class="text-[11px] text-slate-400 whitespace-nowrap">{fmtSize(b.size_bytes)}</span>
                </label>
              {/each}
            </div>
            <button type="button" on:click={restoreFromGitHub} disabled={!selected || !understood} class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2">
              <RotateCcw size={14} /> Restore selected backup
            </button>
          {/if}
        {/if}
      </div>

      <!-- From a file -->
      <div class="space-y-3 pt-4 border-t border-slate-800">
        <div class="text-xs font-bold text-slate-300">2. Or from a backup file</div>
        <p class="text-[11px] text-slate-500">A <code>shop-backup-….zip</code> downloaded from GitHub (up to 50 MB).</p>
        <div class="flex flex-col sm:flex-row sm:items-center gap-3">
          <input bind:this={fileInput} type="file" accept=".zip,application/zip" on:change={pickFile} class="text-xs text-slate-300 file:mr-3 file:px-3.5 file:py-2 file:rounded-xl file:border-0 file:bg-slate-800 file:text-white file:text-xs file:font-bold hover:file:bg-slate-700" />
          <button type="button" on:click={restoreFromUpload} disabled={!uploadFile || !understood} class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2 self-start">
            <Upload size={14} /> Upload and restore
          </button>
        </div>
      </div>

      <div class="space-y-2.5 pt-4 border-t border-slate-800">
        <label class="flex items-start gap-2.5 text-xs text-slate-300 cursor-pointer">
          <input type="checkbox" bind:checked={keepBackupsOff} class="w-4 h-4 mt-0.5 accent-sky-600" />
          <span>Switch automatic backups off after restoring. <span class="text-slate-500">Tick this for a test or local copy, so it does not write backups into the same repository as your live shop.</span></span>
        </label>
        <label class="flex items-start gap-2.5 text-xs font-semibold text-white cursor-pointer">
          <input type="checkbox" bind:checked={understood} class="w-4 h-4 mt-0.5 accent-orange-600" />
          <span>I understand that all current data of this shop is replaced by the backup.</span>
        </label>
      </div>
    {/if}
  </section>
{/if}
