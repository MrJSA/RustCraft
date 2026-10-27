<script>
  import { onMount } from 'svelte';
  import { customer } from '$lib/stores/customer.js';
  import { cart, isCartOpen } from '$lib/stores/cart.js';
  import { goto } from '$app/navigation';
  import { Heart, ArrowLeft, ShoppingBag, Trash2, ArrowRight } from 'lucide-svelte';

  let wishlist = [];
  let isLoading = true;

  async function loadWishlist() {
    try {
      const headers = $customer ? { Authorization: `Bearer ${$customer.token}` } : {};
      const res = await fetch('/api/v1/customer/wishlist', { headers });
      if (res.ok) {
        wishlist = await res.json();
      }
    } catch (e) {
      console.error('Failed to load wishlist:', e);
    } finally {
      isLoading = false;
    }
  }

  async function removeFromWishlist(productId) {
    try {
      const headers = {
        'Content-Type': 'application/json',
        ...($customer ? { Authorization: `Bearer ${$customer.token}` } : {})
      };
      await fetch('/api/v1/customer/wishlist/toggle', {
        method: 'POST',
        headers,
        body: JSON.stringify({ product_id: productId })
      });
      wishlist = wishlist.filter((item) => item.id !== productId);
    } catch (e) {
      console.error('Failed to toggle wishlist:', e);
    }
  }

  onMount(() => {
    loadWishlist();
  });
</script>

<svelte:head>
  <title>My Wishlist | RustCraft</title>
</svelte:head>

<div class="max-w-4xl mx-auto px-4 py-12 space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <a href="/" class="inline-flex items-center gap-1.5 text-xs text-slate-400 hover:text-white mb-2 transition-colors">
        <ArrowLeft size={14} />
        <span>Back to Store</span>
      </a>
      <h1 class="text-2xl font-bold text-white tracking-tight flex items-center gap-2">
        <Heart size={24} class="text-rose-500 fill-rose-500" />
        Customer Wishlist
      </h1>
      <p class="text-xs text-slate-400 mt-0.5">Save your favorite hardware and eBooks for later.</p>
    </div>
  </div>

  {#if isLoading}
    <div class="text-center py-16 text-xs text-slate-400">Loading your wishlist...</div>
  {:else if wishlist.length === 0}
    <div class="p-12 text-center rounded-3xl bg-slate-900 border border-slate-800 text-slate-400 space-y-3">
      <Heart size={36} class="mx-auto text-slate-600" />
      <p class="text-sm font-bold text-slate-200">Your wishlist is empty</p>
      <p class="text-xs text-slate-500">Add products you like so you can easily purchase them anytime.</p>
      <a href="/" class="inline-block mt-2 px-4 py-2 rounded-xl bg-orange-600 text-white text-xs font-bold hover:bg-orange-500 transition-colors">
        Explore Products
      </a>
    </div>
  {:else}
    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-6">
      {#each wishlist as item}
        <div class="rounded-2xl bg-slate-900 border border-slate-800 p-4 flex flex-col justify-between shadow-xl space-y-4">
          <div>
            {#if item.image_url}
              <img src={item.image_url} alt={item.title} class="w-full h-40 object-cover rounded-xl bg-slate-950 mb-3" />
            {/if}
            <span class="text-[10px] font-bold uppercase text-orange-400">{item.category}</span>
            <h3 class="text-sm font-bold text-white tracking-tight mt-1 line-clamp-1">{item.title}</h3>
            <div class="text-xs font-mono font-bold text-slate-200 mt-1">{(item.base_price_cents / 100).toFixed(2)} €</div>
          </div>

          <div class="flex items-center gap-2 pt-2 border-t border-slate-800">
            <a
              href="/products/{item.slug}"
              class="flex-1 py-2 px-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold text-center transition-colors"
            >
              View Product
            </a>
            <button
              on:click={() => removeFromWishlist(item.id)}
              class="p-2 rounded-xl bg-slate-800 hover:bg-rose-500/20 text-slate-400 hover:text-rose-400 transition-colors"
              title="Remove from wishlist"
            >
              <Trash2 size={15} />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
