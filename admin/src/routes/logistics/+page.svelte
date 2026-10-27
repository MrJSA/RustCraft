<script>
  import {
    Warehouse,
    AlertTriangle,
    Search,
    Plus,
    Minus,
    Equal,
    Layers,
    ChevronDown,
    ChevronRight,
    MapPin,
    Edit2,
    Trash2,
    X,
    Check,
    Save,
    Package
  } from 'lucide-svelte';

  export let data;
  let inventory = data.inventory || [];
  let filterLowStockOnly = false;
  let inventoryTypeFilter = 'all'; // 'all' | 'products' | 'parts'
  let searchQuery = '';
  let updatingItemId = null;
  let restockInputs = {};
  let expandedProducts = {};

  // Create Part Modal State
  let isCreatePartModalOpen = false;
  let isSavingPart = false;
  let createPartSku = '';
  let createPartName = '';
  let createPartLocation = 'Warehouse Main, Bin 01';
  let createPartStock = 20;
  let createPartThreshold = 5;
  let createPartNotes = '';

  // Edit Part Modal State
  let isEditPartModalOpen = false;
  let editingPartId = null;
  let editPartSku = '';
  let editPartName = '';
  let editPartLocation = '';
  let editPartStock = 0;
  let editPartThreshold = 5;
  let editPartNotes = '';

  function toggleProductExpand(variantId) {
    expandedProducts[variantId] = !expandedProducts[variantId];
    expandedProducts = expandedProducts;
  }

  function getRestockQty(item) {
    const key = item.part_id || item.variant_id || item.sku;
    if (restockInputs[key] === undefined) {
      restockInputs[key] = 10;
    }
    return restockInputs[key];
  }

  function setRestockQty(item, val) {
    const key = item.part_id || item.variant_id || item.sku;
    restockInputs[key] = Math.max(0, parseInt(val, 10) || 0);
  }

  $: totalProductsCount = inventory.filter(i => i.item_type === 'product').length;
  $: totalPartsCount = inventory.filter(i => i.item_type === 'part').length;

  $: filteredItems = inventory.filter((item) => {
    if (inventoryTypeFilter === 'products' && item.item_type !== 'product') return false;
    if (inventoryTypeFilter === 'parts' && item.item_type !== 'part') return false;
    if (filterLowStockOnly && !item.is_low_stock && !item.is_out_of_stock && !item.bom_has_missing_parts) {
      return false;
    }
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      return (
        (item.sku && item.sku.toLowerCase().includes(q)) ||
        (item.product_title && item.product_title.toLowerCase().includes(q)) ||
        (item.variant_title && item.variant_title.toLowerCase().includes(q)) ||
        (item.part_name && item.part_name.toLowerCase().includes(q)) ||
        (item.storage_location && item.storage_location.toLowerCase().includes(q)) ||
        (item.used_in_summary && item.used_in_summary.toLowerCase().includes(q))
      );
    }
    return true;
  });

  async function reloadInventory() {
    try {
      const invRes = await fetch('/api/v1/admin/logistics/inventory');
      if (invRes.ok) {
        inventory = await invRes.json();
      }
    } catch (e) {
      console.error('Failed to reload inventory:', e);
    }
  }

  async function quickRestockAction(item, actionType) {
    const key = item.part_id || item.variant_id || item.sku;
    const num = Math.max(0, parseInt(restockInputs[key] ?? 10, 10) || 0);
    if (num === 0 && actionType !== 'set') return;

    updatingItemId = key;
    try {
      const isPart = item.item_type === 'part' || !!item.product_part_id;
      const url = isPart
        ? `/api/v1/admin/logistics/parts/${item.part_id}/stock`
        : `/api/v1/admin/logistics/inventory/${item.variant_id}/stock`;

      let body = {};
      if (actionType === 'add') {
        body = { adjustment: num };
      } else if (actionType === 'sub') {
        body = { adjustment: -num };
      } else if (actionType === 'set') {
        body = { absolute_quantity: num };
      }

      const res = await fetch(url, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(body)
      });

      if (res.ok) {
        await reloadInventory();
      } else {
        const err = await res.json().catch(() => ({}));
        alert(err.error || 'Failed to update stock');
      }
    } catch (e) {
      console.error('Failed to update stock:', e);
    } finally {
      updatingItemId = null;
    }
  }

  function openCreatePartModal() {
    createPartSku = '';
    createPartName = '';
    createPartLocation = 'Warehouse Main, Bin 01';
    createPartStock = 20;
    createPartThreshold = 5;
    createPartNotes = '';
    isCreatePartModalOpen = true;
  }

  async function handleCreatePartSubmit() {
    if (!createPartSku.trim()) {
      alert('Please enter a Part SKU');
      return;
    }
    if (!createPartName.trim()) {
      alert('Please enter a Part Name');
      return;
    }

    isSavingPart = true;
    try {
      const res = await fetch('/api/v1/admin/bom-parts', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sku: createPartSku.trim(),
          name: createPartName.trim(),
          storage_location: createPartLocation.trim() || 'Warehouse Main, Bin 01',
          stock_quantity: parseInt(createPartStock) || 0,
          low_stock_threshold: parseInt(createPartThreshold) || 5,
          notes: createPartNotes.trim() || null
        })
      });

      if (res.ok) {
        isCreatePartModalOpen = false;
        await reloadInventory();
      } else {
        const err = await res.json().catch(() => ({}));
        alert('Failed to create BOM part: ' + (err.error || res.statusText));
      }
    } catch (e) {
      console.error('Error creating BOM part:', e);
      alert('Error creating BOM part: ' + e.message);
    } finally {
      isSavingPart = false;
    }
  }

  function openEditPartModal(part) {
    editingPartId = part.part_id || part.id;
    editPartSku = part.part_sku || part.sku || '';
    editPartName = part.part_name || part.name || part.product_title || '';
    editPartLocation = part.storage_location || 'Warehouse Main, Bin 01';
    editPartStock = part.stock_quantity ?? 0;
    editPartThreshold = part.low_stock_threshold ?? 5;
    editPartNotes = part.notes || '';
    isEditPartModalOpen = true;
  }

  async function handleSaveEditedPartSubmit() {
    if (!editPartSku.trim()) {
      alert('Please enter a Part SKU');
      return;
    }
    if (!editPartName.trim()) {
      alert('Please enter a Part Name');
      return;
    }

    isSavingPart = true;
    try {
      const res = await fetch(`/api/v1/admin/bom-parts/${editingPartId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sku: editPartSku.trim(),
          name: editPartName.trim(),
          storage_location: editPartLocation.trim() || 'Warehouse Main, Bin 01',
          stock_quantity: parseInt(editPartStock) || 0,
          low_stock_threshold: parseInt(editPartThreshold) || 5,
          notes: editPartNotes.trim() || null
        })
      });

      if (res.ok) {
        isEditPartModalOpen = false;
        await reloadInventory();
      } else {
        const err = await res.json().catch(() => ({}));
        alert('Failed to update BOM part: ' + (err.error || res.statusText));
      }
    } catch (e) {
      console.error('Error updating BOM part:', e);
      alert('Error updating BOM part: ' + e.message);
    } finally {
      isSavingPart = false;
    }
  }

  async function handleDeletePartAction(partId) {
    if (!confirm('Are you sure you want to delete this BOM part from the central inventory?')) return;
    try {
      const res = await fetch(`/api/v1/admin/bom-parts/${partId}`, {
        method: 'DELETE'
      });
      if (res.ok) {
        await reloadInventory();
      } else {
        const err = await res.json().catch(() => ({}));
        alert('Failed to delete BOM part: ' + (err.error || res.statusText));
      }
    } catch (e) {
      console.error('Error deleting BOM part:', e);
    }
  }
</script>

<svelte:head>
  <title>Logistics & Inventory | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-7xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Warehouse size={24} class="text-orange-500" />
        Logistics, Finished Goods & BOM Parts Management
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Centralized Bill of Materials (BOM) stock, warehouse storage bins, and finished product inventory. Expand any product to inspect and restock its component parts. Product buildable stock is auto-calculated.
      </p>
    </div>

    <!-- Filters, Action & Search -->
    <div class="flex flex-wrap items-center gap-3">
      <!-- Create BOM Part Button -->
      <button
        type="button"
        on:click={openCreatePartModal}
        class="px-3.5 py-2 rounded-xl bg-purple-600 hover:bg-purple-500 text-white text-xs font-bold transition-all shadow-lg shadow-purple-600/20 flex items-center gap-1.5"
      >
        <Plus size={14} />
        <span>Create BOM Part</span>
      </button>

      <!-- Type Filter Tabs -->
      <div class="inline-flex rounded-xl bg-slate-900 border border-slate-800 p-1 text-xs font-semibold">
        <button
          type="button"
          on:click={() => inventoryTypeFilter = 'all'}
          class="px-3 py-1.5 rounded-lg transition-colors {inventoryTypeFilter === 'all' ? 'bg-orange-600 text-white font-bold' : 'text-slate-400 hover:text-white'}"
        >
          All ({inventory.length})
        </button>
        <button
          type="button"
          on:click={() => inventoryTypeFilter = 'products'}
          class="px-3 py-1.5 rounded-lg transition-colors {inventoryTypeFilter === 'products' ? 'bg-orange-600 text-white font-bold' : 'text-slate-400 hover:text-white'}"
        >
          Products ({totalProductsCount})
        </button>
        <button
          type="button"
          on:click={() => inventoryTypeFilter = 'parts'}
          class="px-3 py-1.5 rounded-lg transition-colors {inventoryTypeFilter === 'parts' ? 'bg-purple-600 text-white font-bold' : 'text-slate-400 hover:text-white'}"
        >
          BOM Parts ({totalPartsCount})
        </button>
      </div>

      <div class="relative">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter SKU, location, or part..."
          class="pl-9 pr-4 py-2 rounded-xl bg-slate-900 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500 w-52 sm:w-60"
        />
        <Search size={14} class="absolute left-3 top-3 text-slate-500 pointer-events-none" />
      </div>

      <button
        type="button"
        on:click={() => filterLowStockOnly = !filterLowStockOnly}
        class="px-3.5 py-2 rounded-xl text-xs font-bold border transition-colors flex items-center gap-1.5 {filterLowStockOnly ? 'bg-amber-500/20 text-amber-300 border-amber-500/40' : 'bg-slate-900 text-slate-300 border-slate-800 hover:border-slate-700'}"
      >
        <AlertTriangle size={13} class={filterLowStockOnly ? 'text-amber-400' : 'text-slate-400'} />
        <span>Low Stock / Alerts</span>
      </button>
    </div>
  </div>

  <!-- Warehouse Inventory Table -->
  <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden">
    <div class="overflow-x-auto">
      <table class="w-full text-left text-xs">
        <thead class="bg-slate-950/60 border-b border-slate-800 text-slate-400 uppercase tracking-wider text-[11px]">
          <tr>
            <th class="py-3.5 px-4 font-bold w-12 text-center">Tree</th>
            <th class="py-3.5 px-4 font-bold">SKU Code</th>
            <th class="py-3.5 px-4 font-bold">Item Name & Storage Location</th>
            <th class="py-3.5 px-4 font-bold">Inventory Type</th>
            <th class="py-3.5 px-4 font-bold text-center">Available Stock</th>
            <th class="py-3.5 px-4 font-bold text-center">Status & Readiness</th>
            <th class="py-3.5 px-4 font-bold text-right min-w-[280px]">Restock & Operations</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          {#if filteredItems.length === 0}
            <tr>
              <td colspan="7" class="text-center py-12 text-slate-400">
                No inventory matches the active filter.
              </td>
            </tr>
          {:else}
            {#each filteredItems as item}
              <!-- Main Item Row -->
              <tr
                class="hover:bg-slate-800/30 transition-colors {item.item_type === 'part' ? 'bg-purple-950/5' : ''} {item.item_type === 'product' && item.has_bom_parts ? 'cursor-pointer' : ''}"
                on:click={() => {
                  if (item.item_type === 'product' && item.has_bom_parts) {
                    toggleProductExpand(item.variant_id);
                  }
                }}
              >
                <!-- Tree Expand Icon -->
                <td class="py-4 px-4 text-center">
                  {#if item.item_type === 'product' && item.has_bom_parts}
                    <button
                      type="button"
                      class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
                      title={expandedProducts[item.variant_id] ? 'Collapse BOM components' : 'Expand BOM components'}
                    >
                      <ChevronRight
                        size={16}
                        class="transition-transform duration-200 {expandedProducts[item.variant_id] ? 'rotate-90 text-orange-400' : 'text-slate-400'}"
                      />
                    </button>
                  {:else if item.item_type === 'part'}
                    <Layers size={14} class="text-purple-400 mx-auto" />
                  {:else}
                    <Package size={14} class="text-slate-500 mx-auto" />
                  {/if}
                </td>

                <!-- SKU Code -->
                <td class="py-4 px-4 font-mono font-bold">
                  <div class="flex items-center gap-1.5">
                    <span class="{item.item_type === 'part' ? 'text-purple-400' : 'text-orange-400'}">{item.sku}</span>
                  </div>
                  <div class="text-[10px] text-slate-500 font-sans uppercase font-bold tracking-wider mt-0.5">
                    {item.item_type === 'part' ? 'Part SKU' : 'Product SKU'}
                  </div>
                </td>

                <!-- Product / Part Title & Storage Location -->
                <td class="py-4 px-4">
                  {#if item.item_type === 'part'}
                    <div class="font-bold text-white text-sm flex items-center gap-1.5">
                      <Layers size={14} class="text-purple-400 flex-shrink-0" />
                      <span>{item.part_name || item.product_title}</span>
                    </div>
                    <div class="text-[11px] text-slate-400 mt-1 flex items-center gap-1.5">
                      <MapPin size={12} class="text-purple-400 flex-shrink-0" />
                      <span class="font-mono text-slate-300 font-semibold">{item.storage_location || 'Warehouse Main, Bin 01'}</span>
                    </div>
                    <div class="text-slate-400 text-xs mt-0.5">
                      <span class="text-orange-400 font-semibold">Required by:</span> {item.used_in_summary}
                    </div>
                    {#if item.notes}
                      <div class="text-[10px] text-slate-500 italic mt-0.5">{item.notes}</div>
                    {/if}
                  {:else}
                    <div class="font-bold text-white text-sm flex items-center gap-2">
                      <span>{item.product_title}</span>
                      <span class="text-slate-400 text-xs font-normal">({item.variant_title})</span>
                    </div>
                    <div class="text-slate-400 text-xs mt-0.5">Category: {item.category}</div>
                    {#if item.bom_parts && item.bom_parts.length > 0}
                      <div class="mt-1 flex items-center gap-1.5">
                        {#if item.bom_has_missing_parts}
                          <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-rose-500/15 text-rose-400 border border-rose-500/30 flex items-center gap-1">
                            <AlertTriangle size={10} />
                            <span>{item.bom_parts_depleted} BOM component{item.bom_parts_depleted === 1 ? '' : 's'} depleted</span>
                          </span>
                        {:else}
                          <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                            ✓ All {item.bom_parts_total} BOM components ready
                          </span>
                        {/if}
                        <span class="text-[10px] text-slate-500">
                          (Click row to {expandedProducts[item.variant_id] ? 'collapse' : 'expand'} components)
                        </span>
                      </div>
                    {/if}
                  {/if}
                </td>

                <!-- Type -->
                <td class="py-4 px-4">
                  {#if item.item_type === 'part'}
                    <span class="px-2.5 py-1 rounded text-[10px] font-bold uppercase tracking-wider bg-purple-500/15 text-purple-300 border border-purple-500/30 inline-flex items-center gap-1">
                      <Layers size={11} /> BOM Part
                    </span>
                  {:else if item.product_type === 'digital'}
                    <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-sky-500/10 text-sky-400 border border-sky-500/20">
                      Digital Asset
                    </span>
                  {:else}
                    <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-slate-800 text-slate-300 border border-slate-700">
                      Finished Good
                    </span>
                  {/if}
                </td>

                <!-- In Stock Count -->
                <td class="py-4 px-4 text-center font-mono font-bold text-base {item.is_out_of_stock ? 'text-rose-400' : item.is_low_stock ? 'text-amber-400' : 'text-emerald-400'}">
                  {#if item.product_type === 'digital'}
                    ∞
                  {:else}
                    {item.stock_quantity}
                    <span class="text-[10px] text-slate-500 block font-sans font-normal">
                      {item.item_type === 'part' ? 'units on hand' : item.has_bom_parts ? 'buildable units (BOM)' : 'units'}
                    </span>
                  {/if}
                </td>

                <!-- Status Badge -->
                <td class="py-4 px-4 text-center">
                  {#if item.product_type === 'digital'}
                    <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-sky-500/10 text-sky-400 border border-sky-500/20">
                      Unlimited
                    </span>
                  {:else if item.item_type === 'part'}
                    {#if item.is_out_of_stock}
                      <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-rose-500/20 text-rose-300 border border-rose-500/40 animate-pulse">
                        <AlertTriangle size={11} class="text-rose-400" /> Depleted (Produce Now)
                      </span>
                    {:else if item.is_low_stock}
                      <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/15 text-amber-300 border border-amber-500/30">
                        <AlertTriangle size={11} class="text-amber-400" /> Low Stock (&le; {item.low_stock_threshold})
                      </span>
                    {:else}
                      <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/15 text-emerald-400 border border-emerald-500/30">
                        ✓ Ready in Bin
                      </span>
                    {/if}
                  {:else}
                    {#if item.is_out_of_stock}
                      <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-rose-500/10 text-rose-400 border border-rose-500/20">
                        Out of Stock
                      </span>
                    {:else if item.bom_has_missing_parts}
                      <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-rose-500/10 text-rose-300 border border-rose-500/20">
                        Missing BOM Parts
                      </span>
                    {:else if item.is_low_stock}
                      <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20">
                        Low Stock (&le; {item.low_stock_threshold})
                      </span>
                    {:else}
                      <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                        Healthy Stock
                      </span>
                    {/if}
                  {/if}
                </td>

                <!-- Operations & Quick Restock -->
                <td class="py-4 px-4 text-right" on:click|stopPropagation>
                  {#if item.item_type === 'part'}
                    <div class="inline-flex items-center justify-end gap-1.5">
                      <input
                        type="number"
                        min="0"
                        value={getRestockQty(item)}
                        on:input={(e) => setRestockQty(item, e.target.value)}
                        class="w-16 px-2 py-1.5 rounded-lg bg-slate-950 border border-slate-700 focus:border-purple-500 text-white font-mono text-xs text-center focus:outline-none"
                        title="Enter quantity to add, subtract, or set"
                        disabled={updatingItemId === item.part_id}
                      />
                      <button
                        type="button"
                        on:click={() => quickRestockAction(item, 'add')}
                        disabled={updatingItemId === item.part_id}
                        class="px-2.5 py-1.5 rounded-lg bg-emerald-600/20 hover:bg-emerald-600/30 border border-emerald-500/30 disabled:opacity-30 text-emerald-400 font-mono font-bold text-xs transition-colors flex items-center gap-1"
                        title="Add this quantity to current stock (+)"
                      >
                        <Plus size={12} />
                        <span>Add</span>
                      </button>
                      <button
                        type="button"
                        on:click={() => quickRestockAction(item, 'sub')}
                        disabled={updatingItemId === item.part_id || item.stock_quantity <= 0}
                        class="px-2.5 py-1.5 rounded-lg bg-rose-600/20 hover:bg-rose-600/30 border border-rose-500/30 disabled:opacity-30 text-rose-400 font-mono font-bold text-xs transition-colors flex items-center gap-1"
                        title="Subtract this quantity from current stock (-)"
                      >
                        <Minus size={12} />
                        <span>Sub</span>
                      </button>
                      <button
                        type="button"
                        on:click={() => quickRestockAction(item, 'set')}
                        disabled={updatingItemId === item.part_id}
                        class="px-2.5 py-1.5 rounded-lg bg-purple-600/20 hover:bg-purple-600/30 border border-purple-500/30 disabled:opacity-30 text-purple-300 font-mono font-bold text-xs transition-colors flex items-center gap-1"
                        title="Set current stock exactly to this quantity (=)"
                      >
                        <Equal size={12} />
                        <span>Set</span>
                      </button>
                      <button
                        type="button"
                        on:click={() => openEditPartModal(item)}
                        class="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors ml-1"
                        title="Edit BOM part details & storage location"
                      >
                        <Edit2 size={13} class="text-purple-400" />
                      </button>
                      <button
                        type="button"
                        on:click={() => handleDeletePartAction(item.part_id)}
                        class="p-1.5 rounded-lg bg-slate-800 hover:bg-rose-900/40 text-slate-400 hover:text-rose-400 transition-colors"
                        title="Delete BOM Part"
                      >
                        <Trash2 size={13} />
                      </button>
                    </div>
                  {:else if item.has_bom_parts}
                    <!-- User explicit constraint: Product stock cannot be changed individually, only through BOM parts -->
                    <button
                      type="button"
                      on:click={() => toggleProductExpand(item.variant_id)}
                      class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-orange-600/15 hover:bg-orange-600/25 border border-orange-500/30 text-orange-300 font-medium text-xs transition-colors"
                    >
                      <Layers size={13} class="text-orange-400" />
                      <span>Derived from BOM Parts</span>
                      <ChevronDown size={13} class="transition-transform duration-200 {expandedProducts[item.variant_id] ? 'rotate-180' : ''}" />
                    </button>
                  {:else if item.product_type !== 'digital'}
                    <!-- Product with no BOM parts defined yet -->
                    <div class="inline-flex items-center justify-end gap-1.5">
                      <input
                        type="number"
                        min="0"
                        value={getRestockQty(item)}
                        on:input={(e) => setRestockQty(item, e.target.value)}
                        class="w-16 px-2 py-1.5 rounded-lg bg-slate-950 border border-slate-700 focus:border-orange-500 text-white font-mono text-xs text-center focus:outline-none"
                        disabled={updatingItemId === item.variant_id}
                      />
                      <button
                        type="button"
                        on:click={() => quickRestockAction(item, 'add')}
                        disabled={updatingItemId === item.variant_id}
                        class="px-2.5 py-1.5 rounded-lg bg-emerald-600/20 hover:bg-emerald-600/30 border border-emerald-500/30 disabled:opacity-30 text-emerald-400 font-mono font-bold text-xs transition-colors"
                      >
                        <Plus size={12} />
                      </button>
                      <button
                        type="button"
                        on:click={() => quickRestockAction(item, 'sub')}
                        disabled={updatingItemId === item.variant_id || item.stock_quantity <= 0}
                        class="px-2.5 py-1.5 rounded-lg bg-rose-600/20 hover:bg-rose-600/30 border border-rose-500/30 disabled:opacity-30 text-rose-400 font-mono font-bold text-xs transition-colors"
                      >
                        <Minus size={12} />
                      </button>
                      <button
                        type="button"
                        on:click={() => quickRestockAction(item, 'set')}
                        disabled={updatingItemId === item.variant_id}
                        class="px-2.5 py-1.5 rounded-lg bg-orange-600/20 hover:bg-orange-600/30 border border-orange-500/30 disabled:opacity-30 text-orange-400 font-mono font-bold text-xs transition-colors"
                      >
                        <Equal size={12} />
                      </button>
                    </div>
                  {:else}
                    <span class="text-[11px] text-slate-500">Asset Hosted</span>
                  {/if}
                </td>
              </tr>

              <!-- Expandable Accordion Subrow: BOM Parts Used for this Product -->
              {#if item.item_type === 'product' && item.has_bom_parts && expandedProducts[item.variant_id]}
                <tr class="bg-slate-950/80 border-b border-slate-800">
                  <td colspan="7" class="p-4 sm:p-5">
                    <div class="rounded-xl border border-slate-800 bg-slate-900/90 p-4 space-y-3 shadow-inner">
                      <!-- Subrow Header -->
                      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-800/80 pb-3">
                        <div class="flex items-center gap-2">
                          <Layers size={16} class="text-orange-400" />
                          <h4 class="font-bold text-white text-xs">
                            BOM Components Required for: <span class="text-orange-300">{item.product_title}</span> ({item.variant_title})
                          </h4>
                          <span class="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-orange-500/20 text-orange-300 border border-orange-500/30">
                            {item.stock_quantity} complete builds available
                          </span>
                        </div>
                        <p class="text-[11px] text-slate-400">
                          Adjusting any part stock immediately recalculates the finished product stock.
                        </p>
                      </div>

                      <!-- Components Table -->
                      <div class="overflow-x-auto">
                        <table class="w-full text-left text-xs">
                          <thead class="text-slate-400 text-[10px] uppercase tracking-wider border-b border-slate-800/60 pb-1">
                            <tr>
                              <th class="py-2 px-3 font-semibold">Component SKU</th>
                              <th class="py-2 px-3 font-semibold">Component Name & Specs</th>
                              <th class="py-2 px-3 font-semibold">Storage Location</th>
                              <th class="py-2 px-3 font-semibold text-center">Qty / Product</th>
                              <th class="py-2 px-3 font-semibold text-center">Part Stock</th>
                              <th class="py-2 px-3 font-semibold text-center">Status</th>
                              <th class="py-2 px-3 font-semibold text-right">Adjust Stock & Edit</th>
                            </tr>
                          </thead>
                          <tbody class="divide-y divide-slate-800/40 font-normal">
                            {#each item.bom_parts as part}
                              <tr class="hover:bg-slate-800/20 transition-colors">
                                <td class="py-2.5 px-3 font-mono font-bold text-purple-400">
                                  {part.part_sku || 'PRT-UNSET'}
                                </td>
                                <td class="py-2.5 px-3">
                                  <div class="font-semibold text-white">{part.part_name}</div>
                                  {#if part.notes}
                                    <div class="text-[10px] text-slate-400 italic">{part.notes}</div>
                                  {/if}
                                </td>
                                <td class="py-2.5 px-3 text-slate-300">
                                  <div class="inline-flex items-center gap-1 text-[11px] font-mono text-slate-300 bg-slate-950 px-2 py-0.5 rounded border border-slate-800">
                                    <MapPin size={11} class="text-purple-400" />
                                    <span>{part.storage_location || 'Warehouse Main, Bin 01'}</span>
                                  </div>
                                </td>
                                <td class="py-2.5 px-3 text-center font-mono font-bold text-orange-400">
                                  {part.quantity_required}x
                                </td>
                                <td class="py-2.5 px-3 text-center font-mono font-bold {part.is_depleted ? 'text-rose-400' : 'text-emerald-400'}">
                                  {part.stock_quantity}
                                  <span class="text-[9px] text-slate-500 block font-sans font-normal">
                                    yields {part.buildable_units} builds
                                  </span>
                                </td>
                                <td class="py-2.5 px-3 text-center">
                                  {#if part.is_depleted}
                                    <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-rose-500/20 text-rose-300 border border-rose-500/40 inline-flex items-center gap-1">
                                      <AlertTriangle size={10} /> Depleted (Bottleneck)
                                    </span>
                                  {:else}
                                    <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-500/15 text-emerald-400 border border-emerald-500/30">
                                      ✓ Ready
                                    </span>
                                  {/if}
                                </td>
                                <td class="py-2.5 px-3 text-right">
                                  <div class="inline-flex items-center justify-end gap-1.5">
                                    <input
                                      type="number"
                                      min="0"
                                      value={getRestockQty(part)}
                                      on:input={(e) => setRestockQty(part, e.target.value)}
                                      class="w-14 px-1.5 py-1 rounded bg-slate-950 border border-slate-700 focus:border-purple-500 text-white font-mono text-xs text-center focus:outline-none"
                                      disabled={updatingItemId === part.part_id}
                                    />
                                    <button
                                      type="button"
                                      on:click={() => quickRestockAction(part, 'add')}
                                      disabled={updatingItemId === part.part_id}
                                      class="px-2 py-1 rounded bg-emerald-600/20 hover:bg-emerald-600/30 border border-emerald-500/30 text-emerald-400 font-mono font-bold text-xs transition-colors"
                                      title="Add to part stock (+)"
                                    >
                                      <Plus size={11} />
                                    </button>
                                    <button
                                      type="button"
                                      on:click={() => quickRestockAction(part, 'sub')}
                                      disabled={updatingItemId === part.part_id || part.stock_quantity <= 0}
                                      class="px-2 py-1 rounded bg-rose-600/20 hover:bg-rose-600/30 border border-rose-500/30 text-rose-400 font-mono font-bold text-xs transition-colors"
                                      title="Subtract from part stock (-)"
                                    >
                                      <Minus size={11} />
                                    </button>
                                    <button
                                      type="button"
                                      on:click={() => quickRestockAction(part, 'set')}
                                      disabled={updatingItemId === part.part_id}
                                      class="px-2 py-1 rounded bg-purple-600/20 hover:bg-purple-600/30 border border-purple-500/30 text-purple-300 font-mono font-bold text-xs transition-colors"
                                      title="Set part stock (=)"
                                    >
                                      <Equal size={11} />
                                    </button>
                                    <button
                                      type="button"
                                      on:click={() => openEditPartModal(part)}
                                      class="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors ml-1"
                                      title="Edit part specs & storage location"
                                    >
                                      <Edit2 size={12} class="text-purple-400" />
                                    </button>
                                  </div>
                                </td>
                              </tr>
                            {/each}
                          </tbody>
                        </table>
                      </div>
                    </div>
                  </td>
                </tr>
              {/if}
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<!-- Modal 1: Create Centralized BOM Part -->
{#if isCreatePartModalOpen}
  <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="w-full max-w-lg bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-base font-bold text-white flex items-center gap-2">
          <Layers size={18} class="text-purple-400" />
          <span>Create Centralized BOM Part</span>
        </h3>
        <button
          type="button"
          on:click={() => isCreatePartModalOpen = false}
          class="text-xs text-slate-400 hover:text-white p-1"
        >
          <X size={16} />
        </button>
      </div>

      <p class="text-xs text-slate-400">
        Register a hardware part with centralized stock and storage location. Once created, this part can be added to any number of products in "Products & BOM".
      </p>

      <div class="space-y-3 text-xs">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-400 mb-1 font-semibold">Part SKU <span class="text-rose-400">*</span></label>
            <input
              type="text"
              bind:value={createPartSku}
              placeholder="e.g. PRT-SW-YELLOW-84"
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-xs focus:outline-none focus:border-purple-500 uppercase"
            />
          </div>
          <div>
            <label class="block text-slate-400 mb-1 font-semibold">Part Name <span class="text-rose-400">*</span></label>
            <input
              type="text"
              bind:value={createPartName}
              placeholder="e.g. Gateron Yellow Linear Switches (84x)"
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-purple-500"
            />
          </div>
        </div>

        <div>
          <label class="block text-slate-400 mb-1 font-semibold flex items-center gap-1">
            <MapPin size={12} class="text-purple-400" />
            <span>Storage Location / Warehouse Bin</span>
          </label>
          <input
            type="text"
            bind:value={createPartLocation}
            placeholder="e.g. Aisle 4, Bin 23 (Switches)"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-purple-500"
          />
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-400 mb-1 font-semibold">Initial Stock Quantity</label>
            <input
              type="number"
              min="0"
              bind:value={createPartStock}
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-xs focus:outline-none focus:border-purple-500"
            />
          </div>
          <div>
            <label class="block text-slate-400 mb-1 font-semibold">Low Stock Threshold</label>
            <input
              type="number"
              min="0"
              bind:value={createPartThreshold}
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-xs focus:outline-none focus:border-purple-500"
            />
          </div>
        </div>

        <div>
          <label class="block text-slate-400 mb-1 font-semibold">Technical Notes / Specs (Optional)</label>
          <textarea
            bind:value={createPartNotes}
            rows="2"
            placeholder="e.g. 50g actuation, factory lubed, 5-pin PCB mount"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-purple-500"
          ></textarea>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2.5 pt-2 border-t border-slate-800">
        <button
          type="button"
          on:click={() => isCreatePartModalOpen = false}
          class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold transition-colors"
        >
          Cancel
        </button>
        <button
          type="button"
          on:click={handleCreatePartSubmit}
          disabled={isSavingPart}
          class="px-5 py-2 rounded-xl bg-purple-600 hover:bg-purple-500 disabled:opacity-50 text-white text-xs font-bold transition-colors flex items-center gap-1.5 shadow-lg shadow-purple-600/25"
        >
          <Save size={14} />
          <span>{isSavingPart ? 'Saving...' : 'Create Part'}</span>
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal 2: Edit Centralized BOM Part -->
{#if isEditPartModalOpen}
  <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="w-full max-w-lg bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-base font-bold text-white flex items-center gap-2">
          <Edit2 size={18} class="text-purple-400" />
          <span>Edit BOM Component</span>
        </h3>
        <button
          type="button"
          on:click={() => isEditPartModalOpen = false}
          class="text-xs text-slate-400 hover:text-white p-1"
        >
          <X size={16} />
        </button>
      </div>

      <p class="text-xs text-slate-400">
        Update this component's SKU, name, storage location, or inventory threshold. Changes automatically sync to all linked products.
      </p>

      <div class="space-y-3 text-xs">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-400 mb-1 font-semibold">Part SKU <span class="text-rose-400">*</span></label>
            <input
              type="text"
              bind:value={editPartSku}
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-xs focus:outline-none focus:border-purple-500 uppercase"
            />
          </div>
          <div>
            <label class="block text-slate-400 mb-1 font-semibold">Part Name <span class="text-rose-400">*</span></label>
            <input
              type="text"
              bind:value={editPartName}
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-purple-500"
            />
          </div>
        </div>

        <div>
          <label class="block text-slate-400 mb-1 font-semibold flex items-center gap-1">
            <MapPin size={12} class="text-purple-400" />
            <span>Storage Location / Warehouse Bin</span>
          </label>
          <input
            type="text"
            bind:value={editPartLocation}
            placeholder="e.g. Aisle 2, Bin 14 (Electronics)"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-purple-500"
          />
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-400 mb-1 font-semibold">Stock Quantity</label>
            <input
              type="number"
              min="0"
              bind:value={editPartStock}
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-xs focus:outline-none focus:border-purple-500"
            />
          </div>
          <div>
            <label class="block text-slate-400 mb-1 font-semibold">Low Stock Threshold</label>
            <input
              type="number"
              min="0"
              bind:value={editPartThreshold}
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-xs focus:outline-none focus:border-purple-500"
            />
          </div>
        </div>

        <div>
          <label class="block text-slate-400 mb-1 font-semibold">Technical Notes / Specs</label>
          <textarea
            bind:value={editPartNotes}
            rows="2"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-purple-500"
          ></textarea>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2.5 pt-2 border-t border-slate-800">
        <button
          type="button"
          on:click={() => isEditPartModalOpen = false}
          class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold transition-colors"
        >
          Cancel
        </button>
        <button
          type="button"
          on:click={handleSaveEditedPartSubmit}
          disabled={isSavingPart}
          class="px-5 py-2 rounded-xl bg-purple-600 hover:bg-purple-500 disabled:opacity-50 text-white text-xs font-bold transition-colors flex items-center gap-1.5 shadow-lg shadow-purple-600/25"
        >
          <Save size={14} />
          <span>{isSavingPart ? 'Saving...' : 'Save Changes'}</span>
        </button>
      </div>
    </div>
  </div>
{/if}
