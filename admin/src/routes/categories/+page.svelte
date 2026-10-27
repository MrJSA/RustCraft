<script>
  import CategoryTreeNode from '$lib/components/CategoryTreeNode.svelte';
  import MediaPickerModal from '$lib/components/MediaPickerModal.svelte';
  import { FolderTree, Plus, FolderPlus, CheckCircle2, X, Image as ImageIcon, Trash2 } from 'lucide-svelte';

  export let data;
  let categories = data.categories || [];
  let products = data.products || [];
  let isSaving = false;
  let successNotice = '';

  // Modal State
  let isModalOpen = false;
  let isMediaPickerOpen = false;
  let modalMode = 'create'; // 'create' or 'edit'
  let editId = null;
  let parentId = null;
  let name = '';
  let slug = '';
  let description = '';
  let imageUrl = '';
  let displayOrder = 1;

  $: rootCategories = categories
    .filter((c) => !c.parent_id)
    .sort((a, b) => a.display_order - b.display_order);

  function buildHierarchicalOptions(pid = null, prefix = '') {
    let result = [];
    const directChildren = categories
      .filter(c => c.parent_id === pid)
      .sort((a, b) => a.display_order - b.display_order);

    for (const child of directChildren) {
      if (modalMode === 'edit' && child.id === editId) continue;
      result.push({ id: child.id, label: `${prefix}${prefix ? '↳ ' : ''}${child.name}` });
      result = result.concat(buildHierarchicalOptions(child.id, prefix + '  '));
    }
    return result;
  }

  function openCreateRoot() {
    modalMode = 'create';
    editId = null;
    parentId = null;
    name = '';
    slug = '';
    description = '';
    imageUrl = '';
    displayOrder = (rootCategories.length + 1) * 10;
    isModalOpen = true;
  }

  function openCreateSub(parent) {
    modalMode = 'create';
    editId = null;
    parentId = parent.id;
    name = '';
    slug = '';
    description = '';
    imageUrl = '';
    const siblings = categories.filter(c => c.parent_id === parent.id);
    displayOrder = (siblings.length + 1) * 10;
    isModalOpen = true;
  }

  function openEdit(category) {
    modalMode = 'edit';
    editId = category.id;
    parentId = category.parent_id;
    name = category.name;
    slug = category.slug;
    description = category.description || '';
    imageUrl = category.image_url || '';
    displayOrder = category.display_order;
    isModalOpen = true;
  }

  async function reloadCategories() {
    try {
      const token = localStorage.getItem('admin_token');
      const res = await fetch('/api/v1/admin/categories', {
        headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) }
      });
      if (res.ok) {
        categories = await res.json();
      }
    } catch (e) {
      console.error('Failed to reload categories:', e);
    }
  }

  async function handleSave() {
    isSaving = true;
    const finalSlug = slug.trim() || name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
    const token = localStorage.getItem('admin_token');

    try {
      if (modalMode === 'create') {
        const res = await fetch('/api/v1/admin/categories', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            ...(token ? { Authorization: `Bearer ${token}` } : {})
          },
          body: JSON.stringify({
            parent_id: parentId || null,
            name,
            slug: finalSlug,
            description,
            image_url: imageUrl.trim() || null,
            display_order: parseInt(displayOrder) || 0
          })
        });
        if (res.ok) {
          successNotice = `Category "${name}" created successfully!`;
          isModalOpen = false;
          await reloadCategories();
          setTimeout(() => successNotice = '', 3500);
        }
      } else {
        const res = await fetch(`/api/v1/admin/categories/${editId}`, {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
            ...(token ? { Authorization: `Bearer ${token}` } : {})
          },
          body: JSON.stringify({
            parent_id: parentId || null,
            name,
            slug: finalSlug,
            description,
            image_url: imageUrl.trim() || null,
            display_order: parseInt(displayOrder) || 0
          })
        });
        if (res.ok) {
          successNotice = `Category "${name}" updated successfully!`;
          isModalOpen = false;
          await reloadCategories();
          setTimeout(() => successNotice = '', 3500);
        }
      }
    } catch (e) {
      console.error('Failed to save category:', e);
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete(category) {
    if (!confirm(`Are you sure you want to delete category "${category.name}" and all its child categories?`)) return;
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch(`/api/v1/admin/categories/${category.id}`, {
        method: 'DELETE',
        headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) }
      });
      if (res.ok) {
        await reloadCategories();
      }
    } catch (e) {
      console.error('Failed to delete category:', e);
    }
  }
</script>

<svelte:head>
  <title>Category Hierarchy Tree | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-5xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <FolderTree size={24} class="text-orange-500" />
        Categories & Arbitrary Depth Hierarchy
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Support unlimited nesting (Category &rarr; Subcategory &rarr; Sub-Subcategory &rarr; ...).
      </p>
    </div>

    <button
      on:click={openCreateRoot}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-2 transition-all self-start sm:self-auto"
    >
      <FolderPlus size={16} />
      <span>Add Root Category</span>
    </button>
  </div>

  {#if successNotice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <CheckCircle2 size={16} />
      <span>{successNotice}</span>
    </div>
  {/if}

  <!-- Category Tree Card -->
  <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden p-6 space-y-4">
    {#if rootCategories.length === 0}
      <div class="text-center py-16 text-xs text-slate-400">
        No categories found. Click "Add Root Category" to begin building your tree.
      </div>
    {:else}
      <div class="space-y-2">
        {#each rootCategories as root}
          <CategoryTreeNode
            node={root}
            allCategories={categories}
            products={products}
            depth={0}
            onAddChild={openCreateSub}
            onEdit={openEdit}
            onDelete={handleDelete}
          />
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Create / Edit Modal -->
{#if isModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-lg rounded-3xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-slate-800">
        <h3 class="text-sm font-bold text-white">
          {modalMode === 'create' ? (parentId ? 'Create Nested Subcategory' : 'Create Root Category') : 'Edit Category'}
        </h3>
        <button
          on:click={() => isModalOpen = false}
          class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <form on:submit|preventDefault={handleSave} class="space-y-3.5 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Parent Category</label>
          <select
            bind:value={parentId}
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 font-mono text-xs"
          >
            <option value={null}>None (Root Category)</option>
            {#each buildHierarchicalOptions() as opt}
              <option value={opt.id}>{opt.label}</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Category Name</label>
          <input
            type="text"
            bind:value={name}
            required
            placeholder="e.g. Mechanical Keyboards, Switches, Lubes"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">URL Slug</label>
            <input
              type="text"
              bind:value={slug}
              placeholder="auto-generated from name"
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono placeholder-slate-500 focus:outline-none focus:border-orange-500"
            />
          </div>
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Display Order</label>
            <input
              type="number"
              bind:value={displayOrder}
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Category Image / Icon (Optional)</label>
          <div class="flex items-center gap-3">
            {#if imageUrl}
              <div class="w-12 h-12 rounded-xl bg-slate-950 border border-slate-800 overflow-hidden flex-shrink-0">
                <img src={imageUrl} alt="Preview" class="w-full h-full object-cover" />
              </div>
            {/if}
            <div class="flex-1 flex gap-2">
              <input
                type="text"
                bind:value={imageUrl}
                placeholder="https://... or choose from media"
                class="flex-1 px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500 font-mono text-xs"
              />
              <button
                type="button"
                on:click={() => isMediaPickerOpen = true}
                class="px-3 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-semibold text-xs flex items-center gap-1.5 transition-colors"
              >
                <ImageIcon size={14} class="text-orange-400" />
                <span>Media</span>
              </button>
              {#if imageUrl}
                <button
                  type="button"
                  on:click={() => imageUrl = ''}
                  class="p-2 rounded-xl bg-slate-800 hover:bg-rose-950/40 text-slate-400 hover:text-rose-400 transition-colors"
                  title="Clear Image"
                >
                  <Trash2 size={14} />
                </button>
              {/if}
            </div>
          </div>
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Description (Optional)</label>
          <textarea
            bind:value={description}
            rows="2"
            placeholder="Brief overview of products included in this hierarchy level..."
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500"
          ></textarea>
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
            {isSaving ? 'Saving...' : 'Save Category'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<MediaPickerModal
  open={isMediaPickerOpen}
  onSelect={(url) => imageUrl = url}
  onClose={() => isMediaPickerOpen = false}
/>
