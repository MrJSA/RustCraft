<script>
  import {
    Menu,
    Plus,
    Trash2,
    Check,
    ExternalLink,
    ArrowRight,
    X,
    Link,
    HelpCircle,
    Compass,
    ArrowUp,
    ArrowDown,
    Edit2,
    CornerDownRight,
    FolderPlus,
    Building,
    Share2,
    Globe,
    Save,
    CheckCircle2,
    AlertCircle,
    Layout,
    ShieldCheck
  } from 'lucide-svelte';

  export let menuItems = [];
  export let categories = [];
  export let pages = [];
  export let settings = {};

  let activeTab = 'header'; // 'header' or 'footer'
  let isModalOpen = false;
  let isEditModalOpen = false;
  let editingItemId = null;
  let editLabel = '';
  let editUrl = '/';
  let editSortOrder = 1;
  let editParentId = '';

  let newLabel = '';
  let newUrl = '/';
  let newSortOrder = 1;
  let newParentId = '';
  let selectedPreset = '';
  let isSaving = false;

  $: currentTabItems = menuItems.filter(i => (i.location || 'header') === activeTab);

  $: topLevelItems = currentTabItems
    .filter(i => !i.parent_id)
    .sort((a, b) => a.sort_order - b.sort_order);

  function getChildren(parentId) {
    return currentTabItems
      .filter(i => i.parent_id === parentId)
      .sort((a, b) => a.sort_order - b.sort_order);
  }

  function getItemDepth(item) {
    if (!item || !item.parent_id) return 1;
    const parent = currentTabItems.find(i => i.id === item.parent_id);
    if (!parent || !parent.parent_id) return 2;
    return 3;
  }

  $: parentOptions = currentTabItems
    .filter(i => getItemDepth(i) <= 2)
    .sort((a, b) => (a.sort_order || 0) - (b.sort_order || 0))
    .map(i => {
      const depth = getItemDepth(i);
      let label = i.label;
      if (depth === 2) {
        const p = currentTabItems.find(x => x.id === i.parent_id);
        label = `↳ [${p ? p.label : 'Parent'}] > ${i.label} (Layer 2 -> creates 3rd Layer)`;
      } else {
        label = `${i.label} (Layer 1 Top)`;
      }
      return { id: i.id, label, depth };
    });

  function getDescendantIds(itemId) {
    const children = currentTabItems.filter(i => i.parent_id === itemId);
    let ids = children.map(c => c.id);
    for (const c of children) {
      ids = [...ids, ...getDescendantIds(c.id)];
    }
    return ids;
  }

  // Predefined store routes
  const standardRoutes = [
    { label: 'Home / Overview', url: '/' },
    { label: 'All Products Catalog', url: '/?category=' },
    { label: 'Track Order', url: '/track' },
    { label: 'Customer Login / Register', url: '/account/login' },
    { label: 'Shopping Cart Checkout', url: '/checkout' }
  ];

  $: categoryRoutes = categories.map(c => ({
    label: `Category: ${c.name}`,
    url: `/?category=${encodeURIComponent(c.name)}`
  }));

  $: cmsPageRoutes = pages.map(p => ({
    label: `Policy / Page: ${p.title}`,
    url: `/policies/${p.slug}`
  }));

  $: allPossibleLinks = [
    ...standardRoutes,
    ...categoryRoutes,
    ...cmsPageRoutes
  ];

  function handlePresetSelect(e) {
    const val = e.target.value;
    if (!val) return;
    const found = allPossibleLinks.find(l => l.url === val);
    if (found) {
      newLabel = found.label.replace(/^Category: |^Policy \/ Page: /, '');
      newUrl = found.url;
    }
  }

  function openCreateModal(parentId = '') {
    newLabel = '';
    newUrl = '/';
    selectedPreset = '';
    newParentId = parentId || '';
    if (newParentId) {
      const children = getChildren(newParentId);
      newSortOrder = children.length + 1;
    } else {
      newSortOrder = topLevelItems.length + 1;
    }
    isModalOpen = true;
  }

  async function handleCreateItem() {
    isSaving = true;
    const token = localStorage.getItem('admin_token');

    try {
      const res = await fetch('/api/v1/admin/menu', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          label: newLabel,
          url: newUrl,
          sort_order: parseInt(newSortOrder) || 1,
          is_active: true,
          location: activeTab,
          parent_id: newParentId ? newParentId : null
        })
      });

      if (res.ok) {
        await reloadMenu();
        isModalOpen = false;
      }
    } catch (e) {
      console.error('Failed to create menu item:', e);
    } finally {
      isSaving = false;
    }
  }

  async function reloadMenu() {
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch('/api/v1/admin/menu', {
        headers: {
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (res.ok) {
        menuItems = await res.json();
      }
    } catch (e) {
      console.error('Failed to reload menu:', e);
    }
  }

  async function handleDeleteItem(id) {
    if (!confirm('Are you sure you want to remove this navigation item? (Any nested dropdown items will also be removed)')) return;
    const token = localStorage.getItem('admin_token');

    try {
      const res = await fetch(`/api/v1/admin/menu/${id}`, {
        method: 'DELETE',
        headers: {
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (res.ok) {
        menuItems = menuItems.filter((i) => i.id !== id && i.parent_id !== id);
      }
    } catch (e) {
      console.error('Failed to delete item:', e);
    }
  }

  async function handleToggleActive(item) {
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch(`/api/v1/admin/menu/${item.id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          label: item.label,
          url: item.url,
          sort_order: item.sort_order,
          is_active: !item.is_active,
          location: item.location || activeTab,
          parent_id: item.parent_id || null
        })
      });
      if (res.ok) {
        await reloadMenu();
      }
    } catch (e) {
      console.error('Failed to toggle active status:', e);
    }
  }

  function openEditModal(item) {
    editingItemId = item.id;
    editLabel = item.label;
    editUrl = item.url;
    editSortOrder = item.sort_order;
    editParentId = item.parent_id || '';
    isEditModalOpen = true;
  }

  async function handleEditItem() {
    isSaving = true;
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch(`/api/v1/admin/menu/${editingItemId}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          label: editLabel,
          url: editUrl,
          sort_order: parseInt(editSortOrder) || 1,
          is_active: true,
          location: activeTab,
          parent_id: editParentId ? editParentId : null
        })
      });
      if (res.ok) {
        await reloadMenu();
        isEditModalOpen = false;
      }
    } catch (e) {
      console.error('Failed to update menu item:', e);
    } finally {
      isSaving = false;
    }
  }

  async function moveItem(list, index, direction) {
    const newIndex = index + direction;
    if (newIndex < 0 || newIndex >= list.length) return;
    const itemsCopy = [...list];
    const temp = itemsCopy[index];
    itemsCopy[index] = itemsCopy[newIndex];
    itemsCopy[newIndex] = temp;

    const reorderedPayload = itemsCopy.map((item, idx) => ({
      id: item.id,
      sort_order: idx + 1,
      parent_id: item.parent_id || null
    }));

    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch('/api/v1/admin/menu/reorder', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({ items: reorderedPayload })
      });
      if (res.ok) {
        await reloadMenu();
      }
    } catch (e) {
      console.error('Failed to reorder menu items:', e);
    }
  }

  // --- Footer & Social Architecture State ---
  let footerConfig = {
    branding_mode: 'full',
    menu_layout: 'columns',
    columns: [
      { title: 'Customer Service', links: [] },
      { title: 'Legal & Policies', links: [] },
      { title: 'Store & Support', links: [] }
    ],
    show_socials: true,
    enabled_socials: ['github', 'twitter', 'discord'],
    social_links: {
      github: 'https://github.com',
      twitter: 'https://x.com',
      instagram: '',
      youtube: '',
      facebook: '',
      discord: 'https://discord.gg',
      whatsapp: ''
    },
    show_payments: true,
    copyright_format: 'standard',
    custom_copyright: ''
  };

  let footerConfigInitialized = false;
  $: if (settings && settings.footer_config && !footerConfigInitialized) {
    footerConfigInitialized = true;
    footerConfig = {
      ...footerConfig,
      ...settings.footer_config
    };
    if (!footerConfig.columns || !Array.isArray(footerConfig.columns)) {
      footerConfig.columns = [];
    }
    if (!footerConfig.social_links) {
      footerConfig.social_links = {};
    }
    if (!footerConfig.enabled_socials) {
      footerConfig.enabled_socials = [];
    }
  }

  let isSavingFooter = false;
  let footerSuccessNotice = '';
  let footerErrorNotice = '';

  const socialPlatforms = [
    { id: 'github', name: 'GitHub', placeholder: 'https://github.com/your-username' },
    { id: 'twitter', name: 'Twitter / X', placeholder: 'https://x.com/your-handle' },
    { id: 'instagram', name: 'Instagram', placeholder: 'https://instagram.com/your-profile' },
    { id: 'youtube', name: 'YouTube', placeholder: 'https://youtube.com/@your-channel' },
    { id: 'facebook', name: 'Facebook', placeholder: 'https://facebook.com/your-page' },
    { id: 'discord', name: 'Discord', placeholder: 'https://discord.gg/your-invite' },
    { id: 'whatsapp', name: 'WhatsApp', placeholder: 'https://wa.me/your-phone-number' }
  ];

  function toggleSocial(platformId) {
    if (footerConfig.enabled_socials.includes(platformId)) {
      footerConfig.enabled_socials = footerConfig.enabled_socials.filter(id => id !== platformId);
    } else {
      footerConfig.enabled_socials = [...footerConfig.enabled_socials, platformId];
    }
  }

  async function handleSaveFooter() {
    isSavingFooter = true;
    footerSuccessNotice = '';
    footerErrorNotice = '';

    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch('/api/v1/admin/settings/system', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({ footer_config: footerConfig })
      });

      if (res.ok) {
        settings = { ...settings, footer_config: footerConfig };
        footerSuccessNotice = 'Footer architecture, social channels & payment badges saved!';
        setTimeout(() => footerSuccessNotice = '', 4000);
      } else {
        const err = await res.json().catch(() => ({}));
        footerErrorNotice = err.error || 'Failed to save footer settings.';
      }
    } catch (e) {
      footerErrorNotice = 'Failed to connect to backend server.';
    } finally {
      isSavingFooter = false;
    }
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Menu size={22} class="text-orange-500" />
        Navigation Menus & Page Selector
      </h2>
      <p class="text-xs text-slate-400 mt-1">
        Configure separate menus for the Header navigation bar and the Footer links.
      </p>
    </div>

    <button
      on:click={() => openCreateModal()}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md shadow-orange-600/20 flex items-center gap-2 self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>Add Link to {activeTab === 'header' ? 'Header' : 'Footer'}</span>
    </button>
  </div>

  <!-- Menu Location Tabs -->
  <div class="flex gap-2 border-b border-slate-800 pb-3">
    <button
      type="button"
      on:click={() => activeTab = 'header'}
      class="px-5 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'header' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/30' : 'bg-slate-900 text-slate-400 hover:text-white'}"
    >
      <Compass size={15} />
      <span>Header Navigation Menu ({menuItems.filter(i => (i.location || 'header') === 'header').length})</span>
    </button>

    <button
      type="button"
      on:click={() => activeTab = 'footer'}
      class="px-5 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'footer' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/30' : 'bg-slate-900 text-slate-400 hover:text-white'}"
    >
      <Link size={15} />
      <span>Footer Links Menu ({menuItems.filter(i => i.location === 'footer').length})</span>
    </button>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
    <!-- Items Table (8 cols on lg) -->
    <div class="lg:col-span-8 p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-sm font-bold text-white uppercase tracking-wider">
          {activeTab === 'header' ? 'Storefront Header Links' : 'Storefront Footer Links'}
        </h3>
        <span class="text-xs text-slate-400 font-mono">{currentTabItems.length} active links ({topLevelItems.length} top-level)</span>
      </div>

      {#if topLevelItems.length === 0}
        <div class="py-16 text-center text-xs text-slate-400">
          No navigation items configured for the {activeTab} menu. Click "Add Link" above.
        </div>
      {:else}
        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs">
            <thead>
              <tr class="border-b border-slate-800 text-[11px] text-slate-400 uppercase tracking-wider">
                <th class="pb-3 font-semibold">Order</th>
                <th class="pb-3 font-semibold">Label & Structure</th>
                <th class="pb-3 font-semibold">Target URL</th>
                <th class="pb-3 font-semibold text-center">Status</th>
                <th class="pb-3 font-semibold text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-800/60">
              {#each topLevelItems as item, idx}
                {@const children = getChildren(item.id)}
                <!-- Top-Level Item Row -->
                <tr class="text-slate-300 hover:bg-slate-800/20 transition-colors">
                  <td class="py-3.5 font-mono text-slate-400 font-bold align-middle">
                    <div class="flex items-center gap-1.5">
                      <span>{item.sort_order}</span>
                      <div class="flex flex-col gap-0.5">
                        <button
                          type="button"
                          disabled={idx === 0}
                          on:click={() => moveItem(topLevelItems, idx, -1)}
                          class="p-1 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-30 disabled:hover:bg-slate-800 text-slate-300 hover:text-white transition-colors"
                          title="Move Up"
                        >
                          <ArrowUp size={11} />
                        </button>
                        <button
                          type="button"
                          disabled={idx === topLevelItems.length - 1}
                          on:click={() => moveItem(topLevelItems, idx, 1)}
                          class="p-1 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-30 disabled:hover:bg-slate-800 text-slate-300 hover:text-white transition-colors"
                          title="Move Down"
                        >
                          <ArrowDown size={11} />
                        </button>
                      </div>
                    </div>
                  </td>
                  <td class="py-3.5 align-middle">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-white text-sm">{item.label}</span>
                      {#if children.length > 0}
                        <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20">
                          Dropdown ({children.length})
                        </span>
                      {/if}
                    </div>
                  </td>
                  <td class="py-3.5 font-mono text-orange-400 text-xs align-middle">{item.url}</td>
                  <td class="py-3.5 text-center align-middle">
                    <button
                      type="button"
                      on:click={() => handleToggleActive(item)}
                      class="px-2.5 py-1 rounded-md text-[11px] font-bold transition-colors {item.is_active ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-slate-800 text-slate-500'}"
                    >
                      {item.is_active ? 'Active' : 'Hidden'}
                    </button>
                  </td>
                  <td class="py-3.5 text-right align-middle">
                    <div class="inline-flex items-center gap-1.5">
                      <button
                        type="button"
                        on:click={() => openCreateModal(item.id)}
                        class="p-2 rounded-lg bg-orange-600/15 hover:bg-orange-600/25 text-orange-400 transition-colors border border-orange-500/20 flex items-center gap-1"
                        title="Add dropdown sub-link"
                      >
                        <FolderPlus size={13} />
                        <span class="text-[10px] font-bold hidden sm:inline">+ Sub-link</span>
                      </button>
                      <button
                        type="button"
                        on:click={() => openEditModal(item)}
                        class="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors border border-slate-700"
                        title="Edit Link & Label"
                      >
                        <Edit2 size={13} class="text-orange-400" />
                      </button>
                      <button
                        type="button"
                        on:click={() => handleDeleteItem(item.id)}
                        class="p-2 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors border border-rose-500/20"
                        title="Remove Link"
                      >
                        <Trash2 size={13} />
                      </button>
                    </div>
                  </td>
                </tr>

                <!-- Sub-items Rows (Nested Children - Layer 2) -->
                {#each children as child, cIdx}
                  {@const grandchildren = getChildren(child.id)}
                  <tr class="bg-slate-950/40 text-slate-300 hover:bg-slate-950/80 transition-colors">
                    <td class="py-2.5 pl-6 font-mono text-slate-400 text-xs align-middle">
                      <div class="flex items-center gap-1.5">
                        <CornerDownRight size={12} class="text-amber-500" />
                        <span>{child.sort_order}</span>
                        <div class="flex flex-col gap-0.5">
                          <button
                            type="button"
                            disabled={cIdx === 0}
                            on:click={() => moveItem(children, cIdx, -1)}
                            class="p-0.5 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-30 disabled:hover:bg-slate-800 text-slate-300 hover:text-white transition-colors"
                            title="Move Sub-item Up"
                          >
                            <ArrowUp size={10} />
                          </button>
                          <button
                            type="button"
                            disabled={cIdx === children.length - 1}
                            on:click={() => moveItem(children, cIdx, 1)}
                            class="p-0.5 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-30 disabled:hover:bg-slate-800 text-slate-300 hover:text-white transition-colors"
                            title="Move Sub-item Down"
                          >
                            <ArrowDown size={10} />
                          </button>
                        </div>
                      </div>
                    </td>
                    <td class="py-2.5 pl-4 align-middle">
                      <div class="flex items-center gap-2">
                        <span class="text-xs font-semibold text-slate-200">{child.label}</span>
                        <span class="px-1.5 py-0.5 rounded text-[9px] font-mono bg-slate-800 text-slate-400 border border-slate-700">
                          Layer 2
                        </span>
                        {#if grandchildren.length > 0}
                          <span class="px-1.5 py-0.5 rounded text-[9px] font-bold bg-orange-500/10 text-orange-400 border border-orange-500/20">
                            Flyout ({grandchildren.length})
                          </span>
                        {/if}
                      </div>
                    </td>
                    <td class="py-2.5 font-mono text-amber-400/90 text-xs align-middle">{child.url}</td>
                    <td class="py-2.5 text-center align-middle">
                      <button
                        type="button"
                        on:click={() => handleToggleActive(child)}
                        class="px-2 py-0.5 rounded text-[10px] font-bold transition-colors {child.is_active ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-slate-800 text-slate-500'}"
                      >
                        {child.is_active ? 'Active' : 'Hidden'}
                      </button>
                    </td>
                    <td class="py-2.5 text-right align-middle">
                      <div class="inline-flex items-center gap-1.5">
                        <button
                          type="button"
                          on:click={() => openCreateModal(child.id)}
                          class="p-1.5 rounded-lg bg-orange-600/15 hover:bg-orange-600/25 text-orange-400 transition-colors border border-orange-500/20 flex items-center gap-1"
                          title="Add 3rd-layer sub-link (Grandchild)"
                        >
                          <FolderPlus size={12} />
                          <span class="text-[10px] font-bold hidden sm:inline">+ 3rd Layer</span>
                        </button>
                        <button
                          type="button"
                          on:click={() => openEditModal(child)}
                          class="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors border border-slate-700"
                          title="Edit Sub-link"
                        >
                          <Edit2 size={12} class="text-orange-400" />
                        </button>
                        <button
                          type="button"
                          on:click={() => handleDeleteItem(child.id)}
                          class="p-1.5 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors border border-rose-500/20"
                          title="Remove Sub-link"
                        >
                          <Trash2 size={12} />
                        </button>
                      </div>
                    </td>
                  </tr>

                  <!-- Grandchildren Rows (Layer 3 Flyout Items) -->
                  {#each grandchildren as grand, gIdx}
                    <tr class="bg-slate-950/70 text-slate-300 hover:bg-slate-950 transition-colors">
                      <td class="py-2 pl-12 font-mono text-slate-400 text-xs align-middle">
                        <div class="flex items-center gap-1.5">
                          <CornerDownRight size={11} class="text-orange-400" />
                          <span>{grand.sort_order}</span>
                          <div class="flex flex-col gap-0.5">
                            <button
                              type="button"
                              disabled={gIdx === 0}
                              on:click={() => moveItem(grandchildren, gIdx, -1)}
                              class="p-0.5 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-30 disabled:hover:bg-slate-800 text-slate-300 hover:text-white transition-colors"
                              title="Move Grandchild Up"
                            >
                              <ArrowUp size={9} />
                            </button>
                            <button
                              type="button"
                              disabled={gIdx === grandchildren.length - 1}
                              on:click={() => moveItem(grandchildren, gIdx, 1)}
                              class="p-0.5 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-30 disabled:hover:bg-slate-800 text-slate-300 hover:text-white transition-colors"
                              title="Move Grandchild Down"
                            >
                              <ArrowDown size={9} />
                            </button>
                          </div>
                        </div>
                      </td>
                      <td class="py-2 pl-8 align-middle">
                        <div class="flex items-center gap-2">
                          <span class="text-xs font-semibold text-slate-200">{grand.label}</span>
                          <span class="px-1.5 py-0.5 rounded text-[9px] font-mono bg-orange-500/10 text-orange-400 border border-orange-500/20">
                            Layer 3 (Flyout)
                          </span>
                        </div>
                      </td>
                      <td class="py-2 font-mono text-orange-400/80 text-xs align-middle">{grand.url}</td>
                      <td class="py-2 text-center align-middle">
                        <button
                          type="button"
                          on:click={() => handleToggleActive(grand)}
                          class="px-2 py-0.5 rounded text-[10px] font-bold transition-colors {grand.is_active ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-slate-800 text-slate-500'}"
                        >
                          {grand.is_active ? 'Active' : 'Hidden'}
                        </button>
                      </td>
                      <td class="py-2 text-right align-middle">
                        <div class="inline-flex items-center gap-1.5">
                          <button
                            type="button"
                            on:click={() => openEditModal(grand)}
                            class="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors border border-slate-700"
                            title="Edit 3rd Layer Link"
                          >
                            <Edit2 size={12} class="text-orange-400" />
                          </button>
                          <button
                            type="button"
                            on:click={() => handleDeleteItem(grand.id)}
                            class="p-1.5 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors border border-rose-500/20"
                            title="Remove 3rd Layer Link"
                          >
                            <Trash2 size={12} />
                          </button>
                        </div>
                      </td>
                    </tr>
                  {/each}
                {/each}
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

    <!-- Available Pages Overview Panel (4 cols on lg) -->
    <div class="lg:col-span-4 p-5 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-4">
      <div class="flex items-center gap-2 text-white font-bold text-xs uppercase tracking-wider pb-3 border-b border-slate-800">
        <HelpCircle size={15} class="text-orange-400" />
        <span>Available Target Pages Overview</span>
      </div>

      <div class="space-y-4 max-h-[500px] overflow-y-auto text-xs pr-1">
        <!-- Standard Routes -->
        <div class="space-y-1.5">
          <p class="text-[11px] font-bold text-slate-400 uppercase tracking-wider">Core Store Routes</p>
          {#each standardRoutes as r}
            <div class="p-2 rounded-xl bg-slate-950/80 border border-slate-800/80 flex items-center justify-between">
              <span class="text-slate-200">{r.label}</span>
              <code class="text-[10px] text-orange-400 font-mono">{r.url}</code>
            </div>
          {/each}
        </div>

        <!-- Categories -->
        {#if categories.length > 0}
          <div class="space-y-1.5 pt-2 border-t border-slate-800/80">
            <p class="text-[11px] font-bold text-slate-400 uppercase tracking-wider">Product Categories ({categories.length})</p>
            {#each categories as c}
              <div class="p-2 rounded-xl bg-slate-950/80 border border-slate-800/80 flex items-center justify-between">
                <span class="text-slate-200 truncate pr-2">{c.name}</span>
                <code class="text-[10px] text-sky-400 font-mono truncate">/?category={c.name}</code>
              </div>
            {/each}
          </div>
        {/if}

        <!-- CMS Pages -->
        {#if pages.length > 0}
          <div class="space-y-1.5 pt-2 border-t border-slate-800/80">
            <p class="text-[11px] font-bold text-slate-400 uppercase tracking-wider">CMS Policy Pages ({pages.length})</p>
            {#each pages as p}
              <div class="p-2 rounded-xl bg-slate-950/80 border border-slate-800/80 flex items-center justify-between">
                <span class="text-slate-200 truncate pr-2">{p.title}</span>
                <code class="text-[10px] text-emerald-400 font-mono truncate">/policies/{p.slug}</code>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- When in Footer Tab: Footer Architecture & Social Media Presentation Panel -->
  {#if activeTab === 'footer'}
    <div class="mt-8 space-y-6 pt-6 border-t border-slate-800">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h3 class="text-lg font-bold text-white flex items-center gap-2">
            <Layout size={20} class="text-orange-400" />
            <span>Footer Brand Presentation, Social Media & Trust Badges</span>
          </h3>
          <p class="text-xs text-slate-400 mt-0.5">
            Configure how your brand info, social networks, and verified payment badges render across the storefront footer.
          </p>
        </div>

        <button
          type="button"
          on:click={handleSaveFooter}
          disabled={isSavingFooter}
          class="px-6 py-2.5 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all flex items-center gap-2 disabled:opacity-50 self-start sm:self-auto"
        >
          <Save size={15} />
          <span>{isSavingFooter ? 'Saving Footer Settings...' : 'Save Footer Settings'}</span>
        </button>
      </div>

      {#if footerSuccessNotice}
        <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
          <CheckCircle2 size={16} />
          <span>{footerSuccessNotice}</span>
        </div>
      {/if}

      {#if footerErrorNotice}
        <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={16} />
          <span>{footerErrorNotice}</span>
        </div>
      {/if}

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Section 1: Storefront Branding Presentation -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <h4 class="text-sm font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
            <Building size={16} class="text-orange-400" />
            <span>Left Section: Storefront Branding</span>
          </h4>
          <p class="text-xs text-slate-400">Choose how your brand identity appears on the left side of the footer.</p>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
            <label class="p-3.5 rounded-xl border cursor-pointer transition-all {footerConfig.branding_mode === 'full' ? 'bg-orange-600/10 border-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'}">
              <div class="flex items-center gap-2.5">
                <input
                  type="radio"
                  name="branding_mode"
                  value="full"
                  bind:group={footerConfig.branding_mode}
                  class="accent-orange-600"
                />
                <div>
                  <div class="font-bold text-white text-xs">Full Store Identity</div>
                  <div class="text-[11px] text-slate-400">Logo, address, email & phone</div>
                </div>
              </div>
            </label>

            <label class="p-3.5 rounded-xl border cursor-pointer transition-all {footerConfig.branding_mode === 'logo_only' ? 'bg-orange-600/10 border-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'}">
              <div class="flex items-center gap-2.5">
                <input
                  type="radio"
                  name="branding_mode"
                  value="logo_only"
                  bind:group={footerConfig.branding_mode}
                  class="accent-orange-600"
                />
                <div>
                  <div class="font-bold text-white text-xs">Logo Only</div>
                  <div class="text-[11px] text-slate-400">Minimalist brand logo only</div>
                </div>
              </div>
            </label>
          </div>
        </div>

        <!-- Section 2: Accepted Payments Badges -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <div class="flex items-center justify-between border-b border-slate-800 pb-3">
            <h4 class="text-sm font-bold text-white flex items-center gap-2">
              <ShieldCheck size={16} class="text-orange-400" />
              <span>Accepted Payment Badges</span>
            </h4>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" bind:checked={footerConfig.show_payments} class="sr-only peer" />
              <div class="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-orange-600"></div>
              <span class="ml-2 text-xs text-slate-300 font-semibold">{footerConfig.show_payments ? 'Visible' : 'Hidden'}</span>
            </label>
          </div>
          <p class="text-xs text-slate-400">
            Displays verified official logos for Stripe, PayPal, Apple Pay, Google Pay, and Amazon Pay in the storefront footer.
          </p>
          <div class="flex flex-wrap gap-2 pt-2">
            <span class="px-2.5 py-1 rounded-lg bg-slate-950 border border-slate-800 text-[11px] text-slate-300">Stripe</span>
            <span class="px-2.5 py-1 rounded-lg bg-slate-950 border border-slate-800 text-[11px] text-slate-300">PayPal</span>
            <span class="px-2.5 py-1 rounded-lg bg-slate-950 border border-slate-800 text-[11px] text-slate-300">Apple Pay</span>
            <span class="px-2.5 py-1 rounded-lg bg-slate-950 border border-slate-800 text-[11px] text-slate-300">Google Pay</span>
            <span class="px-2.5 py-1 rounded-lg bg-slate-950 border border-slate-800 text-[11px] text-slate-300">Amazon Pay</span>
          </div>
        </div>

        <!-- Section 3: Social Media Channels (Full Width) -->
        <div class="md:col-span-2 p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <div class="flex items-center justify-between border-b border-slate-800 pb-3">
            <h4 class="text-sm font-bold text-white flex items-center gap-2">
              <Share2 size={16} class="text-orange-400" />
              <span>Social Media Profiles & Follow Channels</span>
            </h4>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" bind:checked={footerConfig.show_socials} class="sr-only peer" />
              <div class="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-orange-600"></div>
              <span class="ml-2 text-xs text-slate-300 font-semibold">{footerConfig.show_socials ? 'Visible' : 'Hidden'}</span>
            </label>
          </div>

          {#if footerConfig.show_socials}
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3.5">
              {#each socialPlatforms as platform}
                <div class="p-3.5 rounded-xl border transition-all space-y-2 {footerConfig.enabled_socials.includes(platform.id) ? 'bg-slate-950 border-orange-500/50 ring-1 ring-orange-500/20' : 'bg-slate-950/60 border-slate-800/80 opacity-60'}">
                  <div class="flex items-center justify-between">
                    <label class="flex items-center gap-2 font-bold text-white text-xs cursor-pointer">
                      <Globe size={13} class="text-orange-400" />
                      <span>{platform.name}</span>
                    </label>
                    <input
                      type="checkbox"
                      checked={footerConfig.enabled_socials.includes(platform.id)}
                      on:change={() => toggleSocial(platform.id)}
                      class="accent-orange-600 rounded"
                    />
                  </div>

                  {#if footerConfig.enabled_socials.includes(platform.id)}
                    <input
                      type="text"
                      bind:value={footerConfig.social_links[platform.id]}
                      placeholder={platform.placeholder}
                      class="w-full px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-200 font-mono text-xs focus:outline-none focus:border-orange-500"
                    />
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Section 4: Copyright Bar (Full Width) -->
        <div class="md:col-span-2 p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <h4 class="text-sm font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
            <ShieldCheck size={16} class="text-orange-400" />
            <span>Full-Width Bottom Copyright Bar</span>
          </h4>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
            <label class="p-3.5 rounded-xl border cursor-pointer transition-all {footerConfig.copyright_format === 'standard' ? 'bg-orange-600/10 border-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400'}">
              <div class="flex items-center gap-2.5">
                <input
                  type="radio"
                  name="copyright_format"
                  value="standard"
                  bind:group={footerConfig.copyright_format}
                  class="accent-orange-600"
                />
                <div>
                  <div class="font-bold text-white text-xs">Dynamic Storefront Copyright</div>
                  <div class="text-[11px] text-slate-400">© 2026 {settings.store_name || 'RustCraft'}. All rights reserved.</div>
                </div>
              </div>
            </label>

            <label class="p-3.5 rounded-xl border cursor-pointer transition-all {footerConfig.copyright_format === 'custom' ? 'bg-orange-600/10 border-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400'}">
              <div class="flex items-center gap-2.5">
                <input
                  type="radio"
                  name="copyright_format"
                  value="custom"
                  bind:group={footerConfig.copyright_format}
                  class="accent-orange-600"
                />
                <div>
                  <div class="font-bold text-white text-xs">Custom Copyright Line</div>
                  <div class="text-[11px] text-slate-400">Explicit custom entity text</div>
                </div>
              </div>
            </label>

            {#if footerConfig.copyright_format === 'custom'}
              <div class="sm:col-span-2">
                <label class="block text-slate-300 font-semibold mb-1">Custom Copyright Text</label>
                <input
                  type="text"
                  bind:value={footerConfig.custom_copyright}
                  placeholder="e.g. Copyright © 2026 Example Store. All rights reserved."
                  class="w-full px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
                />
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Add Navigation Item Modal -->
{#if isModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-md rounded-3xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-slate-800">
        <h3 class="text-sm font-bold text-white">Add Link to {activeTab === 'header' ? 'Header' : 'Footer'} Menu</h3>
        <button
          type="button"
          on:click={() => isModalOpen = false}
          class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <form on:submit|preventDefault={handleCreateItem} class="space-y-4 text-xs">
        <!-- Quick Select Preset Page Dropdown -->
        <div>
          <label class="block text-slate-300 font-semibold mb-1">
            Choose from Existing Shop Pages (Auto-Fills Link & URL)
          </label>
          <select
            on:change={handlePresetSelect}
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 font-mono text-xs"
          >
            <option value="">-- Or select an existing page below --</option>
            <optgroup label="Core Store Pages">
              {#each standardRoutes as r}
                <option value={r.url}>{r.label} ({r.url})</option>
              {/each}
            </optgroup>
            {#if categories.length > 0}
              <optgroup label="Categories">
                {#each categoryRoutes as c}
                  <option value={c.url}>{c.label}</option>
                {/each}
              </optgroup>
            {/if}
            {#if pages.length > 0}
              <optgroup label="Policy & CMS Pages">
                {#each cmsPageRoutes as p}
                  <option value={p.url}>{p.label}</option>
                {/each}
              </optgroup>
            {/if}
          </select>
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Menu Label / Link Title</label>
          <input
            type="text"
            bind:value={newLabel}
            required
            placeholder="e.g. Mechanical Hardware, Shipping Policy"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Target URL</label>
          <input
            type="text"
            bind:value={newUrl}
            required
            placeholder="e.g. /policies/shipment-policy or /?category=Hardware"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono placeholder-slate-500 focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Parent Item (Up to 3 Navigation Layers)</label>
          <select
            bind:value={newParentId}
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 text-xs"
          >
            <option value="">[ None - Top Level Menu Link (Layer 1) ]</option>
            {#each parentOptions as p}
              <option value={p.id}>{p.label}</option>
            {/each}
          </select>
          <p class="text-[10px] text-slate-500 mt-1">
            Nesting under Layer 1 creates a Dropdown (Layer 2). Nesting under Layer 2 creates a Nested Flyout Submenu (Layer 3).
          </p>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Menu Placement</label>
            <select
              bind:value={activeTab}
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            >
              <option value="header">Header Menu</option>
              <option value="footer">Footer Menu</option>
            </select>
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Sort Order</label>
            <input
              type="number"
              bind:value={newSortOrder}
              min="1"
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div class="pt-2 flex justify-end gap-2 border-t border-slate-800">
          <button
            type="button"
            on:click={() => isModalOpen = false}
            class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-semibold"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSaving}
            class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold transition-all shadow-md shadow-orange-600/30 disabled:opacity-50"
          >
            {isSaving ? 'Adding...' : 'Add Menu Item'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Edit Navigation Item Modal -->
{#if isEditModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-md rounded-3xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-slate-800">
        <h3 class="text-sm font-bold text-white flex items-center gap-2">
          <Edit2 size={16} class="text-orange-400" />
          <span>Edit Navigation Link</span>
        </h3>
        <button
          type="button"
          on:click={() => isEditModalOpen = false}
          class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <form on:submit|preventDefault={handleEditItem} class="space-y-4 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Menu Label / Link Title</label>
          <input
            type="text"
            bind:value={editLabel}
            required
            placeholder="e.g. Keyboards, Shipping Policy"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Target URL</label>
          <input
            type="text"
            bind:value={editUrl}
            required
            placeholder="e.g. /policies/shipment-policy or /?category=Hardware"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono placeholder-slate-500 focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Parent Item (Up to 3 Navigation Layers)</label>
          <select
            bind:value={editParentId}
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 text-xs"
          >
            <option value="">[ None - Top Level Menu Link (Layer 1) ]</option>
            {#each parentOptions.filter(p => p.id !== editingItemId && !getDescendantIds(editingItemId).includes(p.id)) as p}
              <option value={p.id}>{p.label}</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Sort Order Position</label>
          <input
            type="number"
            bind:value={editSortOrder}
            min="1"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="pt-2 flex justify-end gap-2 border-t border-slate-800">
          <button
            type="button"
            on:click={() => isEditModalOpen = false}
            class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-semibold"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSaving}
            class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold transition-all shadow-md shadow-orange-600/30 disabled:opacity-50"
          >
            {isSaving ? 'Saving Changes...' : 'Save Changes'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
