<script>
  import { Folder, Plus, Edit2, Trash2, ChevronRight, ChevronDown } from 'lucide-svelte';

  export let node;
  export let allCategories = [];
  export let products = [];
  export let depth = 0;
  export let onAddChild = (parent) => {};
  export let onEdit = (cat) => {};
  export let onDelete = (cat) => {};

  let isExpanded = true;
  let isProductsExpanded = false;

  $: children = allCategories
    .filter((c) => c.parent_id === node.id)
    .sort((a, b) => a.display_order - b.display_order);

  function getAllDescendantNames(catId) {
    let names = [];
    const directChildren = allCategories.filter(c => c.parent_id === catId);
    for (const ch of directChildren) {
      names.push(ch.name);
      names = names.concat(getAllDescendantNames(ch.id));
    }
    return names;
  }

  $: descendantNames = new Set(getAllDescendantNames(node.id));
  $: directProducts = products.filter(p => p.subcategory === node.name || (p.category === node.name && (!p.subcategory || p.subcategory === node.name)));
  $: inheritedProducts = products.filter(p => !directProducts.includes(p) && (descendantNames.has(p.subcategory) || descendantNames.has(p.category)));
  $: allProductsInNode = [...directProducts, ...inheritedProducts];
</script>

<div class="border border-slate-800/80 rounded-2xl bg-slate-950/40 overflow-hidden mb-2">
  <!-- Node Header Row -->
  <div class="p-3.5 bg-slate-900/60 flex items-center justify-between gap-3 hover:bg-slate-900/90 transition-colors">
    <div class="flex items-center gap-2.5 min-w-0">
      <!-- Expand/Collapse toggle if has children -->
      {#if children.length > 0}
        <button
          type="button"
          on:click={() => isExpanded = !isExpanded}
          class="p-1 rounded text-slate-400 hover:text-white transition-colors"
          aria-label="Toggle children"
        >
          {#if isExpanded}
            <ChevronDown size={14} />
          {:else}
            <ChevronRight size={14} />
          {/if}
        </button>
      {:else}
        <div class="w-5"></div>
      {/if}

      <!-- Icon / Thumbnail & Depth indicator -->
      {#if node.image_url}
        <img
          src={node.image_url}
          alt={node.name}
          class="w-7 h-7 sm:w-8 sm:h-8 rounded-lg object-cover border border-slate-700/80 flex-shrink-0"
        />
      {:else}
        <div class="p-1.5 rounded-lg {depth === 0 ? 'bg-orange-500/10 text-orange-400 border border-orange-500/20' : depth === 1 ? 'bg-sky-500/10 text-sky-400 border border-sky-500/20' : 'bg-purple-500/10 text-purple-400 border border-purple-500/20'}">
          <Folder size={15} />
        </div>
      {/if}

      <div class="min-w-0">
        <div class="flex items-center gap-2 flex-wrap">
          <span class="text-xs sm:text-sm font-bold text-white truncate">{node.name}</span>
          <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-800 text-slate-300 border border-slate-700">
            /{node.slug}
          </span>
          <span class="text-[10px] font-mono text-slate-500">
            Order: {node.display_order}
          </span>
          {#if depth > 0}
            <span class="text-[9px] font-mono px-1 rounded bg-slate-900 text-slate-400">
              Level {depth + 1}
            </span>
          {/if}
        </div>
        {#if node.description}
          <p class="text-[11px] text-slate-400 truncate mt-0.5">{node.description}</p>
        {/if}
      </div>
    </div>

    <!-- Actions: Add Child, Edit, Delete, Product Count -->
    <div class="flex items-center gap-2 flex-shrink-0">
      {#if allProductsInNode.length > 0}
        <button
          type="button"
          on:click={() => isProductsExpanded = !isProductsExpanded}
          class="text-[11px] font-bold px-2.5 py-1 rounded-lg border transition-all flex items-center gap-1.5 {isProductsExpanded ? 'bg-orange-500/20 text-orange-300 border-orange-500/40' : directProducts.length > 0 ? 'bg-orange-500/10 text-orange-400 border-orange-500/20 hover:bg-orange-500/20' : 'bg-slate-800 text-slate-300 border-slate-700 hover:bg-slate-700'}"
          title="Toggle {allProductsInNode.length} included products"
        >
          <span>{allProductsInNode.length} {allProductsInNode.length === 1 ? 'product' : 'products'}</span>
          {#if directProducts.length > 0 && inheritedProducts.length > 0}
            <span class="text-[9px] opacity-75 font-normal">({directProducts.length} dir, {inheritedProducts.length} sub)</span>
          {/if}
        </button>
      {:else}
        <span class="text-[10px] text-slate-500 font-mono px-2 py-0.5 rounded bg-slate-800/50">0 products</span>
      {/if}

      <button
        type="button"
        on:click={() => onAddChild(node)}
        class="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] font-semibold flex items-center gap-1 transition-colors border border-slate-700"
        title="Add subcategory under {node.name}"
      >
        <Plus size={12} class="text-orange-400" />
        <span class="hidden sm:inline">Add Child</span>
      </button>

      <button
        type="button"
        on:click={() => onEdit(node)}
        class="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors"
        title="Edit"
      >
        <Edit2 size={13} />
      </button>

      <button
        type="button"
        on:click={() => onDelete(node)}
        class="p-1.5 rounded-lg bg-slate-800 hover:bg-rose-950/40 text-slate-400 hover:text-rose-400 transition-colors"
        title="Delete"
      >
        <Trash2 size={13} />
      </button>
    </div>
  </div>

  <!-- Expanded Products List -->
  {#if isProductsExpanded && allProductsInNode.length > 0}
    <div class="px-4 py-3 bg-slate-950/80 border-t border-slate-800/80 space-y-2">
      <div class="text-[11px] font-bold text-slate-300 flex items-center justify-between">
        <span class="flex items-center gap-1.5 text-orange-400">
          <span>Products in "{node.name}" & Subcategories ({allProductsInNode.length})</span>
        </span>
        <span class="text-[10px] text-slate-500">Every subchild product is automatically included here</span>
      </div>
      <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
        {#each allProductsInNode as p}
          <div class="flex items-center gap-2 p-2 rounded-xl bg-slate-900 border border-slate-800 hover:border-slate-700 transition-colors">
            {#if p.image_url}
              <img src={p.image_url} alt={p.title} class="w-8 h-8 rounded-lg object-cover border border-slate-700 flex-shrink-0" />
            {:else}
              <div class="w-8 h-8 rounded-lg bg-slate-800 flex items-center justify-center text-slate-500 text-[10px] flex-shrink-0">IMG</div>
            {/if}
            <div class="min-w-0 flex-1">
              <p class="text-xs font-semibold text-white truncate">{p.title}</p>
              <p class="text-[10px] text-slate-400 flex items-center gap-1 truncate">
                <span class="font-mono text-orange-400 font-bold">€{(p.base_price_cents / 100).toFixed(2)}</span>
                <span>&bull;</span>
                {#if p.subcategory === node.name}
                  <span class="text-emerald-400 font-medium">Direct</span>
                {:else}
                  <span class="text-sky-400 truncate">via {p.subcategory || p.category}</span>
                {/if}
              </p>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Recursive Child Branches -->
  {#if children.length > 0 && isExpanded}
    <div class="pl-4 sm:pl-8 pr-2 py-2 border-t border-slate-800/60 bg-slate-950/30">
      {#each children as child}
        <svelte:self
          node={child}
          allCategories={allCategories}
          products={products}
          depth={depth + 1}
          {onAddChild}
          {onEdit}
          {onDelete}
        />
      {/each}
    </div>
  {/if}
</div>
