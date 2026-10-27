<script>
  import { onMount } from 'svelte';
  import { customer } from '$lib/stores/customer.js';
  import { goto } from '$app/navigation';
  import { 
    Download, ArrowLeft, ExternalLink, Box, Sparkles, 
    CheckCircle2, RefreshCw, FileText, Layers, ShieldCheck 
  } from 'lucide-svelte';

  let downloads = [];
  let isLoading = true;
  let searchTerm = '';

  onMount(async () => {
    if (!$customer || !$customer.isLoggedIn) {
      goto('/account/login?redirect=/account/downloads');
      return;
    }

    try {
      const res = await fetch('/api/v1/customer/downloads', {
        headers: { Authorization: `Bearer ${$customer.token}` }
      });
      if (res.ok) {
        downloads = await res.json();
      }
    } catch (e) {
      console.error('Failed to load customer downloads:', e);
    } finally {
      isLoading = false;
    }
  });

  $: filteredDownloads = downloads.filter(d => {
    if (!searchTerm.trim()) return true;
    const term = searchTerm.toLowerCase();
    return (
      (d.product_title && d.product_title.toLowerCase().includes(term)) ||
      (d.order_number && d.order_number.toLowerCase().includes(term)) ||
      (d.files && d.files.some(f => f.name.toLowerCase().includes(term)))
    );
  });
</script>

<svelte:head>
  <title>My Digital Downloads | RustCraft</title>
</svelte:head>

<div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-10 space-y-6">
  <!-- Top Header Navigation -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800 pb-5">
    <div>
      <div class="flex items-center gap-3 mb-2 text-xs">
        <a href="/" class="inline-flex items-center gap-1.5 text-slate-400 hover:text-white transition-colors">
          <ArrowLeft size={14} />
          <span>Store</span>
        </a>
        <span class="text-slate-600">/</span>
        <a href="/account/orders" class="text-slate-400 hover:text-white transition-colors">
          <span>Account Orders</span>
        </a>
        <span class="text-slate-600">/</span>
        <span class="text-sky-400 font-semibold">Digital Downloads</span>
      </div>

      <h1 class="text-2xl sm:text-3xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Download size={28} class="text-sky-400" />
        <span>My Digital Downloads Library</span>
      </h1>
      <p class="text-xs text-slate-400 mt-1 max-w-2xl">
        Access, re-download, and inspect all digital assets, CAD models, software licenses, and documentation from your purchases. 
        Files automatically update to the latest revisions released by the creator.
      </p>
    </div>

    <div class="text-xs text-slate-400 font-mono">
      Account: <strong class="text-white">{$customer?.email || 'Customer'}</strong>
    </div>
  </div>

  <!-- Dynamic Live Version Guarantee Banner -->
  <div class="p-4 rounded-2xl bg-sky-950/30 border border-sky-800/40 text-xs text-sky-200 flex items-start gap-3 shadow-lg">
    <div class="p-2 rounded-xl bg-sky-500/10 text-sky-400 flex-shrink-0 mt-0.5">
      <RefreshCw size={16} class="animate-spin-slow" />
    </div>
    <div class="space-y-1">
      <p class="font-bold text-sky-100 flex items-center gap-2">
        <span>Always-Latest Files Guarantee</span>
        <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-sky-500/20 text-sky-300">Live Sync</span>
      </p>
      <p class="text-slate-300 leading-relaxed text-[11px]">
        Whenever the store owner revises blueprints, updates firmware binaries, or publishes new digital asset packages, your download links instantly deliver the updated and changed files with no additional fee.
      </p>
    </div>
  </div>

  <!-- Search / Filter Bar (if downloads exist) -->
  {#if downloads.length > 0}
    <div class="flex items-center justify-between gap-4">
      <div class="relative flex-1 max-w-sm">
        <input
          type="text"
          bind:value={searchTerm}
          placeholder="Search by product, file, or order #..."
          class="w-full px-3.5 py-2 rounded-xl bg-slate-900 border border-slate-800 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-sky-500 transition-colors"
        />
      </div>
      <div class="text-xs text-slate-400 font-mono">
        Showing <strong>{filteredDownloads.length}</strong> of {downloads.length} purchases
      </div>
    </div>
  {/if}

  <!-- Loading State -->
  {#if isLoading}
    <div class="text-center py-20 text-xs text-slate-400 flex flex-col items-center gap-3">
      <div class="w-8 h-8 border-2 border-sky-500 border-t-transparent rounded-full animate-spin"></div>
      <span>Fetching your digital library & asset files...</span>
    </div>
  {:else if downloads.length === 0}
    <!-- Empty State -->
    <div class="p-12 text-center rounded-3xl bg-slate-900 border border-slate-800 text-slate-400 space-y-4 shadow-xl">
      <div class="w-16 h-16 rounded-2xl bg-slate-800/80 border border-slate-700/60 flex items-center justify-center mx-auto text-slate-500">
        <Download size={32} />
      </div>
      <p class="text-lg font-bold text-slate-200">No Digital Purchases Found</p>
      <p class="text-xs text-slate-400 max-w-md mx-auto leading-relaxed">
        You haven't purchased any instant digital products or software licenses yet. Once you order any digital goods, all downloadable assets will appear in this hub for lifetime access.
      </p>
      <a 
        href="/" 
        class="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-sky-600 text-white text-xs font-bold hover:bg-sky-500 transition-colors shadow-lg shadow-sky-600/20"
      >
        <span>Browse Products Catalog</span>
        <ExternalLink size={13} />
      </a>
    </div>
  {:else if filteredDownloads.length === 0}
    <div class="p-8 text-center rounded-2xl bg-slate-900 border border-slate-800 text-slate-400 text-xs">
      No digital products match your filter "{searchTerm}".
    </div>
  {:else}
    <!-- Download Items List -->
    <div class="space-y-4">
      {#each filteredDownloads as item}
        <div class="p-5 sm:p-6 rounded-2xl bg-slate-900 border border-slate-800 hover:border-slate-700 transition-all shadow-xl space-y-4">
          <!-- Item Header Row -->
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800/80 pb-4">
            <div class="flex items-center gap-3.5 min-w-0">
              <div class="w-12 h-12 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center overflow-hidden flex-shrink-0">
                {#if item.image_url}
                  <img src={item.image_url} alt={item.product_title} class="w-full h-full object-cover" />
                {:else}
                  <Sparkles size={20} class="text-sky-400" />
                {/if}
              </div>

              <div class="min-w-0">
                <h3 class="text-base font-bold text-white truncate">{item.product_title}</h3>
                <div class="flex items-center gap-2 text-[11px] text-slate-400 flex-wrap mt-0.5">
                  <span class="font-mono text-slate-300">Order #{item.order_number}</span>
                  <span>&bull;</span>
                  <span>Purchased on {new Date(item.purchase_date).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })}</span>
                  <span>&bull;</span>
                  <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                    Payment Verified
                  </span>
                </div>
              </div>
            </div>

            <div class="flex items-center gap-2 self-start sm:self-auto">
              <a 
                href="/account/orders" 
                class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-750 text-slate-300 hover:text-white text-xs font-semibold border border-slate-700 transition-colors"
              >
                View Full Order
              </a>
            </div>
          </div>

          <!-- Included Downloadable Files List -->
          <div>
            <div class="text-[11px] font-bold uppercase tracking-wider text-slate-400 mb-2.5 flex items-center gap-1.5">
              <FileText size={13} class="text-sky-400" />
              <span>Available File Packages ({item.files ? item.files.length : 0})</span>
            </div>

            {#if !item.files || item.files.length === 0}
              <div class="p-4 rounded-xl bg-slate-950 border border-slate-800/80 text-xs text-slate-500 flex items-center justify-between">
                <span>Direct download link being generated by merchant. Check back shortly.</span>
              </div>
            {:else}
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
                {#each item.files as file}
                  <div class="p-3.5 rounded-xl bg-slate-950/80 border border-slate-800 hover:border-sky-500/50 transition-all flex items-center justify-between gap-3 group">
                    <div class="min-w-0 flex items-center gap-2.5">
                      <div class="w-8 h-8 rounded-lg bg-sky-950/60 border border-sky-800/50 flex items-center justify-center text-sky-400 flex-shrink-0 group-hover:scale-105 transition-transform">
                        <Download size={14} />
                      </div>
                      <div class="min-w-0">
                        <div class="text-xs font-bold text-white truncate">{file.name}</div>
                        <div class="text-[10px] text-slate-400 truncate max-w-[220px]">
                          Latest version{#if file.downloads_left !== null && file.downloads_left !== undefined} · {file.downloads_left} downloads left{/if}{#if file.expires_at} · until {new Date(file.expires_at).toLocaleDateString('en-GB')}{/if}
                        </div>
                      </div>
                    </div>

                    <a
                      href={file.url}
                      target="_blank"
                      rel="noreferrer"
                      download
                      class="px-3.5 py-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 text-white text-xs font-bold flex items-center gap-1.5 shadow-sm transition-all flex-shrink-0 hover:shadow-sky-500/20"
                    >
                      <Download size={12} />
                      <span>Download</span>
                    </a>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
