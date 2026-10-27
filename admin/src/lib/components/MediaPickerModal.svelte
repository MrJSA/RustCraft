<script>
  import { onMount } from 'svelte';
  import { Image, Upload, X, Search, Check } from 'lucide-svelte';

  export let open = false;
  export let onSelect = (url) => {};
  export let onClose = () => {};

  let mediaItems = [];
  let loading = false;
  let uploading = false;
  let searchQuery = '';
  let fileInput;

  $: if (open) {
    loadMedia();
  }

  async function loadMedia() {
    loading = true;
    try {
      const token = localStorage.getItem('admin_token');
      const res = await fetch('/api/v1/admin/media', {
        headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) }
      });
      if (res.ok) {
        mediaItems = await res.json();
      }
    } catch (e) {
      console.error('Failed to load media in picker:', e);
    } finally {
      loading = false;
    }
  }

  async function handleUpload(e) {
    const files = e.target.files;
    if (!files || files.length === 0) return;

    uploading = true;
    const token = localStorage.getItem('admin_token');
    const formData = new FormData();
    formData.append('file', files[0]);

    try {
      const res = await fetch('/api/v1/admin/media/upload', {
        method: 'POST',
        headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) },
        body: formData
      });

      if (res.ok) {
        const item = await res.json();
        onSelect(item.url);
        onClose();
      }
    } catch (err) {
      console.error('Upload failed:', err);
    } finally {
      uploading = false;
      if (fileInput) fileInput.value = '';
    }
  }

  $: filteredItems = mediaItems.filter(m => 
    (m.original_name && m.original_name.toLowerCase().includes(searchQuery.toLowerCase())) ||
    (m.filename && m.filename.toLowerCase().includes(searchQuery.toLowerCase()))
  );
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-3xl rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl flex flex-col max-h-[85vh] overflow-hidden">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Image size={18} class="text-orange-500" />
          <h3 class="text-sm font-bold text-white">Select or Upload Media</h3>
        </div>
        <button
          on:click={onClose}
          class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Controls & Upload -->
      <div class="p-4 border-b border-slate-800/80 bg-slate-950/40 flex items-center justify-between gap-3">
        <div class="relative flex-1">
          <Search size={14} class="absolute left-3 top-2.5 text-slate-500 pointer-events-none" />
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Search media..."
            class="w-full pl-9 pr-3 py-1.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs placeholder-slate-500 focus:outline-none focus:border-orange-500"
          />
        </div>

        <button
          on:click={() => fileInput.click()}
          disabled={uploading}
          class="px-3.5 py-1.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs flex items-center gap-1.5 transition-colors disabled:opacity-50"
        >
          <Upload size={13} />
          <span>{uploading ? 'Compressing...' : 'Upload New'}</span>
        </button>
        <input
          type="file"
          bind:this={fileInput}
          on:change={handleUpload}
          accept="image/*"
          class="hidden"
        />
      </div>

      <!-- Gallery Grid -->
      <div class="flex-1 overflow-y-auto p-4">
        {#if loading && mediaItems.length === 0}
          <div class="py-16 text-center text-xs text-slate-400 font-mono">Loading media assets...</div>
        {:else if filteredItems.length === 0}
          <div class="py-16 text-center text-xs text-slate-400">
            No media assets found. Upload an image to select it.
          </div>
        {:else}
          <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 gap-3">
            {#each filteredItems as item}
              <button
                type="button"
                on:click={() => {
                  onSelect(item.url);
                  onClose();
                }}
                class="group aspect-square rounded-xl bg-slate-950 border border-slate-800 hover:border-orange-500 overflow-hidden relative transition-all focus:outline-none focus:ring-2 focus:ring-orange-500 checkerboard-pattern"
              >
                <img
                  src={item.url}
                  alt={item.original_name}
                  class="w-full h-full object-contain p-2 group-hover:scale-105 transition-transform"
                />
                <div class="absolute inset-x-0 bottom-0 bg-slate-950/80 p-1 text-[9px] text-slate-300 font-mono truncate text-center opacity-0 group-hover:opacity-100 transition-opacity">
                  {item.original_name}
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-3 border-t border-slate-800 bg-slate-950/60 flex justify-end">
        <button
          on:click={onClose}
          class="px-4 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}

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
