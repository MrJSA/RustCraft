<script>
  import { onMount } from 'svelte';
  import { 
    Image, Upload, Trash2, Copy, Check, Search, FileText, 
    HardDrive, ExternalLink, RefreshCw, AlertCircle
  } from 'lucide-svelte';

  let mediaItems = [];
  let loading = true;
  let uploading = false;
  let searchQuery = '';
  let copiedId = null;
  let uploadError = '';
  let fileInput;

  onMount(loadMedia);

  export async function loadMedia() {
    loading = true;
    try {
      const token = localStorage.getItem('admin_token');
      const res = await fetch('/api/v1/admin/media', {
        headers: { 
          ...(token ? { Authorization: `Bearer ${token}` } : {}) 
        }
      });
      if (res.ok) {
        mediaItems = await res.json();
      }
    } catch (e) {
      console.error('Failed to load media items:', e);
    } finally {
      loading = false;
    }
  }

  async function handleFileUpload(e) {
    const files = e.target.files;
    if (!files || files.length === 0) return;

    uploading = true;
    uploadError = '';
    const token = localStorage.getItem('admin_token');

    for (let i = 0; i < files.length; i++) {
      const formData = new FormData();
      formData.append('file', files[i]);

      try {
        const res = await fetch('/api/v1/admin/media/upload', {
          method: 'POST',
          headers: { 
            ...(token ? { Authorization: `Bearer ${token}` } : {}) 
          },
          body: formData
        });

        if (!res.ok) {
          const errText = await res.text();
          try {
            const errJson = JSON.parse(errText);
            uploadError = errJson.error || errJson.message || errText;
          } catch {
            uploadError = errText || `Upload failed with status code ${res.status}`;
          }
        }
      } catch (err) {
        uploadError = 'Connection error during image upload. Check network or server status.';
      }
    }

    uploading = false;
    if (fileInput) fileInput.value = '';
    loadMedia();
  }

  async function deleteMedia(id) {
    if (!confirm('Are you sure you want to permanently delete this media asset?')) return;
    const token = localStorage.getItem('admin_token');

    try {
      const res = await fetch(`/api/v1/admin/media/${id}`, {
        method: 'DELETE',
        headers: { 
          ...(token ? { Authorization: `Bearer ${token}` } : {}) 
        }
      });
      if (res.ok) {
        mediaItems = mediaItems.filter(m => m.id !== id);
      }
    } catch (e) {
      console.error('Failed to delete media:', e);
    }
  }

  function copyToClipboard(url, id) {
    navigator.clipboard.writeText(url);
    copiedId = id;
    setTimeout(() => copiedId = null, 2000);
  }

  function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  $: filteredItems = mediaItems.filter(m => 
    (m.original_name && m.original_name.toLowerCase().includes(searchQuery.toLowerCase())) ||
    (m.filename && m.filename.toLowerCase().includes(searchQuery.toLowerCase()))
  );
</script>

<div class="space-y-6">
  <!-- Page Header -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-slate-800 pb-4">
    <div>
      <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2">
        <Image size={22} class="text-orange-500" />
        <span>Media Library & Assets</span>
      </h2>
      <p class="text-xs text-slate-400 mt-1">
        Centralized persistent storage. Images are automatically converted to compressed WebP for instant storefront loading.
      </p>
    </div>

    <div class="flex items-center gap-3">
      <button
        type="button"
        on:click={loadMedia}
        class="p-2.5 rounded-xl bg-slate-900 border border-slate-800 hover:bg-slate-800 text-slate-300 hover:text-white transition-colors"
        title="Refresh Media"
      >
        <RefreshCw size={15} class={loading ? 'animate-spin' : ''} />
      </button>

      <button
        type="button"
        on:click={() => fileInput.click()}
        disabled={uploading}
        class="px-4 py-2.5 rounded-xl bg-gradient-to-r from-orange-600 to-amber-600 hover:from-orange-500 hover:to-amber-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-2 transition-all disabled:opacity-50"
      >
        <Upload size={15} />
        <span>{uploading ? 'Processing & Compressing...' : 'Upload Media'}</span>
      </button>
      <input
        type="file"
        bind:this={fileInput}
        on:change={handleFileUpload}
        accept="image/*"
        multiple
        class="hidden"
      />
    </div>
  </div>

  {#if uploadError}
    <div class="p-3 rounded-xl bg-rose-500/20 text-rose-400 border border-rose-500/30 text-xs flex items-center gap-2">
      <AlertCircle size={16} />
      <span>{uploadError}</span>
    </div>
  {/if}

  <!-- Search & Stats Bar -->
  <div class="p-4 rounded-2xl bg-slate-900/60 border border-slate-800 flex flex-col sm:flex-row items-center justify-between gap-4">
    <div class="relative w-full sm:w-80">
      <Search size={15} class="absolute left-3.5 top-3 text-slate-500 pointer-events-none" />
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Filter media assets by name..."
        class="w-full pl-10 pr-4 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs placeholder-slate-500 focus:outline-none focus:border-orange-500"
      />
    </div>

    <div class="flex items-center gap-4 text-xs text-slate-400 font-mono">
      <span>Total Assets: <strong class="text-white">{mediaItems.length}</strong></span>
      <span>&bull;</span>
      <span>Showing: <strong class="text-orange-400">{filteredItems.length}</strong></span>
    </div>
  </div>

  <!-- Media Gallery Grid -->
  {#if loading && mediaItems.length === 0}
    <div class="py-24 text-center text-slate-400 font-mono text-xs">
      <RefreshCw size={24} class="mx-auto animate-spin text-orange-500 mb-2" />
      <span>Loading media repository...</span>
    </div>
  {:else if filteredItems.length === 0}
    <div class="py-20 text-center rounded-3xl bg-slate-900/40 border border-slate-800/80 space-y-3">
      <div class="w-16 h-16 rounded-full bg-slate-800/60 flex items-center justify-center mx-auto text-slate-500">
        <Image size={28} />
      </div>
      <h3 class="text-base font-bold text-white">No media files found</h3>
      <p class="text-xs text-slate-400 max-w-sm mx-auto">
        Upload product photography, logos, or hero banners. They will be stored in your persistent Docker volume.
      </p>
      <button
        type="button"
        on:click={() => fileInput.click()}
        class="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold transition-colors"
      >
        <Upload size={14} />
        <span>Upload First Image</span>
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
      {#each filteredItems as item}
        <div class="group rounded-2xl bg-slate-900 border border-slate-800/90 hover:border-orange-500/50 overflow-hidden flex flex-col transition-all shadow-md hover:shadow-xl">
          <!-- Thumbnail Aspect Square with Transparency Checkerboard -->
          <div class="relative aspect-square overflow-hidden flex items-center justify-center checkerboard-pattern">
            <img
              src={item.url}
              alt={item.original_name}
              class="w-full h-full object-contain p-2.5 group-hover:scale-105 transition-transform duration-300"
              loading="lazy"
            />

            <!-- Format Badge (PNG / WEBP / SVG / etc) -->
            <div class="absolute top-2 left-2 pointer-events-none">
              <span class="px-1.5 py-0.5 rounded-md bg-slate-950/80 backdrop-blur-sm border border-slate-800 text-[9px] font-mono font-bold text-slate-300 uppercase">
                {item.mime_type ? item.mime_type.split('/')[1] : (item.filename ? item.filename.split('.').pop() : 'IMG')}
              </span>
            </div>

            <div class="absolute inset-0 bg-slate-950/70 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
              <button
                type="button"
                on:click={() => copyToClipboard(item.url, item.id)}
                class="p-2 rounded-lg bg-slate-900/90 hover:bg-orange-600 text-white transition-colors"
                title="Copy Image URL"
              >
                {#if copiedId === item.id}
                  <Check size={14} class="text-emerald-400" />
                {:else}
                  <Copy size={14} />
                {/if}
              </button>
              <a
                href={item.url}
                target="_blank"
                class="p-2 rounded-lg bg-slate-900/90 hover:bg-orange-600 text-white transition-colors"
                title="View Full Resolution"
              >
                <ExternalLink size={14} />
              </a>
              <button
                type="button"
                on:click={() => deleteMedia(item.id)}
                class="p-2 rounded-lg bg-slate-900/90 hover:bg-rose-600 text-white transition-colors"
                title="Delete Media"
              >
                <Trash2 size={14} />
              </button>
            </div>
          </div>

          <!-- Metadata -->
          <div class="p-3 text-xs flex-1 flex flex-col justify-between space-y-1">
            <p class="font-bold text-white truncate" title={item.original_name}>
              {item.original_name}
            </p>
            <div class="flex items-center justify-between text-[10px] text-slate-400 font-mono">
              <span>{item.mime_type ? item.mime_type.split('/')[1].toUpperCase() : 'IMAGE'}</span>
              <span>{formatBytes(item.size_bytes || item.file_size_bytes || 0)}</span>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .checkerboard-pattern {
    background-color: #0b0f19;
    background-image: 
      linear-gradient(45deg, #1e293b 25%, transparent 25%), 
      linear-gradient(-45deg, #1e293b 25%, transparent 25%), 
      linear-gradient(45deg, transparent 75%, #1e293b 75%), 
      linear-gradient(-45deg, transparent 75%, #1e293b 75%);
    background-size: 16px 16px;
    background-position: 0 0, 0 8px, 8px -8px, -8px 0px;
  }
</style>
