<script>
  import { FileText, Save, Check, Eye, ExternalLink, Bold, Italic, Heading, List, Link, Code, Info } from 'lucide-svelte';
  import { page } from '$app/stores';

  export let pages = [];

  let selectedSlug = pages[0] ? pages[0].slug : 'shipment-policy';
  let title = '';
  let contentMarkdown = '';
  let isSaving = false;
  let saveSuccess = false;

  $: activePage = pages.find((p) => p.slug === selectedSlug);

  $: if (activePage) {
    title = activePage.title;
    contentMarkdown = activePage.content_markdown;
  }

  function insertMarkdown(prefix, suffix = '') {
    contentMarkdown += `${prefix}${suffix}`;
  }

  function parseMarkdown(md) {
    if (!md) return '';
    let html = md
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/^### (.*$)/gim, '<h3 class="text-base font-bold text-white mt-4 mb-1">$1</h3>')
      .replace(/^## (.*$)/gim, '<h2 class="text-lg font-bold text-white mt-5 mb-2 border-b border-slate-800 pb-1">$1</h2>')
      .replace(/^# (.*$)/gim, '<h1 class="text-2xl font-black text-white mb-4">$1</h1>')
      .replace(/\*\*(.*?)\*\*/gim, '<strong class="font-bold text-white">$1</strong>')
      .replace(/\*(.*?)\*/gim, '<em class="italic text-slate-300">$1</em>')
      .replace(/`(.*?)`/gim, '<code class="px-1.5 py-0.5 rounded bg-slate-950 font-mono text-orange-400 text-xs">$1</code>')
      .replace(/\[(.*?)\]\((.*?)\)/gim, '<a href="$2" target="_blank" class="text-orange-400 underline">$1</a>')
      .replace(/^\s*-\s+(.*$)/gim, '<li class="ml-4 list-disc text-slate-300">$1</li>')
      .replace(/\n\n/gim, '</p><p class="my-3 text-slate-300 leading-relaxed text-xs">');

    return `<p class="my-3 text-slate-300 leading-relaxed text-xs">${html}</p>`;
  }

  async function handleSave() {
    isSaving = true;
    saveSuccess = false;
    try {
      const res = await fetch(`/api/v1/admin/pages/${selectedSlug}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          title,
          content_markdown: contentMarkdown,
          is_published: true
        })
      });

      if (res.ok) {
        saveSuccess = true;
        pages = pages.map((p) => (p.slug === selectedSlug ? { ...p, title, content_markdown: contentMarkdown } : p));
        setTimeout(() => (saveSuccess = false), 3000);
      }
    } catch (e) {
      console.error('Failed to save page:', e);
    } finally {
      isSaving = false;
    }
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5">
        <FileText size={22} class="text-orange-500" />
        Legal Notices & Store Policy CMS (Markdown)
      </h2>
      <p class="text-xs text-slate-400 mt-1">
        Direct Markdown content management for Shipment Policy, Terms, Privacy, Cookies, and Legal Notices.
      </p>
    </div>

    <div class="flex items-center gap-3">
      <a
        href="{$page.data.shopUrl}/policies/{selectedSlug}"
        target="_blank"
        class="px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold flex items-center gap-1.5 transition-colors border border-slate-700"
      >
        <Eye size={14} />
        <span>View Live on Store</span>
        <ExternalLink size={12} class="text-orange-400" />
      </a>

      <button
        on:click={handleSave}
        disabled={isSaving}
        class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-1.5 transition-all"
      >
        {#if saveSuccess}
          <Check size={14} class="text-white" />
          <span>Saved!</span>
        {:else}
          <Save size={14} />
          <span>{isSaving ? 'Saving...' : 'Save Policy'}</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Store Identity Variable Tags Hint Banner -->
  <div class="p-3.5 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 text-xs flex items-start gap-2.5">
    <Info size={16} class="text-indigo-400 flex-shrink-0 mt-0.5" />
    <div class="text-slate-300 space-y-1">
      <p class="font-bold text-white">Dynamic Store Identity Placeholders Available:</p>
      <p class="text-[11px] text-slate-400">
        You can use placeholders that automatically resolve from your Store Settings:
        <code class="px-1.5 py-0.5 rounded bg-slate-900 border border-slate-700 text-orange-400 font-mono">{"{{STORE_NAME}}"}</code>,
        <code class="px-1.5 py-0.5 rounded bg-slate-900 border border-slate-700 text-orange-400 font-mono">{"{{COMPANY_ADDRESS}}"}</code>,
        <code class="px-1.5 py-0.5 rounded bg-slate-900 border border-slate-700 text-orange-400 font-mono">{"{{SUPPORT_EMAIL}}"}</code>,
        <code class="px-1.5 py-0.5 rounded bg-slate-900 border border-slate-700 text-orange-400 font-mono">{"{{PHONE}}"}</code>,
        <code class="px-1.5 py-0.5 rounded bg-slate-900 border border-slate-700 text-orange-400 font-mono">{"{{VAT_ID}}"}</code>,
        <code class="px-1.5 py-0.5 rounded bg-slate-900 border border-slate-700 text-orange-400 font-mono">{"{{TAX_NOTICE}}"}</code>.
      </p>
      <p class="text-[11px] text-slate-400">
        Also: {"{{LEGAL_NAME}}"}, {"{{STORE_OWNER}}"}, {"{{COMMERCIAL_REGISTER}}"}, and the address parts {"{{STREET_LINE}}"} (street + number), {"{{STREET}}"}, {"{{HOUSE_NUMBER}}"}, {"{{ADDRESS_EXTRA}}"}, {"{{CITY_LINE}}"} (postal code + city), {"{{POSTAL_CODE}}"}, {"{{CITY}}"}, {"{{COUNTRY}}"}.
        A line whose placeholders are all empty is hidden on the shop.
      </p>
    </div>
  </div>

  <!-- Page Selector Pills -->
  <div class="flex flex-wrap items-center gap-2">
    {#each pages as page}
      <button
        on:click={() => (selectedSlug = page.slug)}
        class="px-4 py-2 rounded-xl text-xs font-bold transition-all {selectedSlug === page.slug
          ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/30'
          : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
      >
        {page.title}
      </button>
    {/each}
  </div>

  <!-- Editor Container -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <!-- Left Column: Markdown Editor -->
    <div class="space-y-4">
      <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 space-y-4">
        <div>
          <label class="block text-slate-400 font-semibold text-xs mb-1.5">Document Title</label>
          <input
            type="text"
            bind:value={title}
            class="w-full px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-bold text-sm focus:outline-none focus:border-orange-500"
          />
        </div>

        <!-- Markdown Formatting Bar -->
        <div class="flex flex-wrap items-center gap-1.5 p-1.5 rounded-xl bg-slate-950 border border-slate-800">
          <button
            type="button"
            on:click={() => insertMarkdown('## ')}
            class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
            title="Heading 2"
          >
            <Heading size={14} />
          </button>
          <button
            type="button"
            on:click={() => insertMarkdown('**', '**')}
            class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
            title="Bold"
          >
            <Bold size={14} />
          </button>
          <button
            type="button"
            on:click={() => insertMarkdown('*', '*')}
            class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
            title="Italic"
          >
            <Italic size={14} />
          </button>
          <button
            type="button"
            on:click={() => insertMarkdown('- ')}
            class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
            title="Bullet List"
          >
            <List size={14} />
          </button>
          <button
            type="button"
            on:click={() => insertMarkdown('[Link text](url)')}
            class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
            title="Add Link"
          >
            <Link size={14} />
          </button>
          <button
            type="button"
            on:click={() => insertMarkdown('`', '`')}
            class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
            title="Inline Code"
          >
            <Code size={14} />
          </button>
        </div>

        <div>
          <label class="block text-slate-400 font-semibold text-xs mb-1.5">Markdown Source</label>
          <textarea
            bind:value={contentMarkdown}
            rows="22"
            class="w-full px-3.5 py-3 rounded-xl bg-slate-950 border border-slate-800 text-slate-200 font-mono text-xs focus:outline-none focus:border-orange-500 leading-relaxed resize-y"
          ></textarea>
        </div>
      </div>
    </div>

    <!-- Right Column: Live Storefront HTML Preview -->
    <div>
      <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 space-y-4 h-full flex flex-col">
        <div class="flex items-center justify-between border-b border-slate-800 pb-3">
          <span class="text-xs font-bold text-slate-400 uppercase tracking-wider">Live Render Preview</span>
          <span class="text-[10px] font-mono text-slate-500">HTML5 Render</span>
        </div>

        <div class="prose prose-invert prose-sm max-w-none flex-1 overflow-y-auto pr-2">
          <h1 class="text-2xl font-black text-white mb-4">{title}</h1>
          {@html parseMarkdown(contentMarkdown)}
        </div>
      </div>
    </div>
  </div>
</div>
