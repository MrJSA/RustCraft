<script>
  import { cart, isCartOpen } from '$lib/stores/cart.js';
  import { customer } from '$lib/stores/customer.js';
  import ProductCard from '$lib/components/ProductCard.svelte';
  import PriceNote from '$lib/components/PriceNote.svelte';
  import ProductReviews from '$lib/components/ProductReviews.svelte';
  import { variantPricing, formatEuro, reductionPercent, deliveryTime, deliveryForCountry, formatWeight } from '$lib/pricing.js';
  import { browser } from '$app/environment';
  import Seo from '$lib/components/Seo.svelte';
  import { page } from '$app/stores';
  import { absoluteUrl, toDescription } from '$lib/seo.js';
  import {
    Download,
    ArrowLeft,
    Plus,
    Minus,
    Box,
    Heart,
    Bell,
    Maximize2,
    X,
    ChevronLeft,
    ChevronRight,
    Sparkles,
    ShieldAlert,
    Truck
  } from 'lucide-svelte';

  export let data;
  $: product = data.product || {};
  $: variants = data.variants || [];
  $: relatedProducts = data.relatedProducts || [];
  $: store = data.store || {};

  let selectedVariantIndex = 0;
  let activeImageIndex = 0;
  let isLightboxOpen = false;
  let quantity = 1;
  let isInWishlist = false;
  let stockNotificationEmail = '';
  let subscribingStock = false;
  let stockNotificationSuccess = false;

  $: currentVariant = variants[selectedVariantIndex] || {};
  $: pricing = variantPricing(product, currentVariant);
  $: currentPriceCents = pricing.price_cents;
  $: weightGrams = currentVariant.weight_grams ?? product.weight_grams;
  $: manufacturer = (product.gpsr_manufacturer || store.gpsr_default_manufacturer || '').trim();

  // Country-specific delivery time only for logged-in customers with a saved address (no guessing
  // the location; everyone else sees the delivery time for their country at checkout)
  let shippingInfo = null;
  let shippingInfoKey = '';
  $: if (browser && $customer?.token && currentVariant.id && !isDigital) loadShippingInfo(currentVariant.id);

  async function loadShippingInfo(key) {
    if (key === shippingInfoKey) return;
    shippingInfoKey = key;
    try {
      const res = await fetch('/api/v1/customer/addresses', { headers: { Authorization: `Bearer ${$customer.token}` } });
      if (!res.ok) return;
      const list = await res.json();
      const address = (Array.isArray(list) && (list.find((a) => a.is_default) || list[0])) || null;
      if (!address?.country_code) return;
      const info = await deliveryForCountry(address.country_code, key);
      if (info && key === shippingInfoKey) {
        let countryName = address.country_code;
        try {
          countryName = new Intl.DisplayNames(['en'], { type: 'region' }).of(address.country_code) || address.country_code;
        } catch (_) {}
        shippingInfo = { ...info, countryName };
      }
    } catch (_) {
      // optional information; the checkout shows it in any case
    }
  }
  $: isDigital = product.product_type === 'digital';
  $: inStock = isDigital || (currentVariant.stock_quantity && currentVariant.stock_quantity > 0);
  $: isLowStock = !isDigital && currentVariant.stock_quantity > 0 && currentVariant.stock_quantity <= currentVariant.low_stock_threshold;

  // Resolve gallery images: check variant images, then product images, then single image_url
  $: currentImages = (() => {
    if (currentVariant.images && Array.isArray(currentVariant.images) && currentVariant.images.length > 0) {
      return currentVariant.images;
    }
    if (product.images && Array.isArray(product.images) && product.images.length > 0) {
      return product.images;
    }
    const single = currentVariant.image_url || product.image_url;
    return single ? [single] : [];
  })();

  $: activeImageUrl = currentImages[activeImageIndex] || currentImages[0] || currentVariant.image_url || product.image_url || '';

  // --- SEO structured data (schema.org Product + BreadcrumbList) ---
  $: storeName = store.store_name || 'Shop';
  $: origin = $page.url.origin;
  $: productUrl = `${origin}/products/${product.slug}`;
  $: variantPrice = (v) => variantPricing(product, v).price_cents || 0;
  $: variantInStock = (v) => isDigital || (v.stock_quantity || 0) > 0;
  $: offerVariants = variants.length > 0 ? variants : [{}];
  $: productJsonLd = {
    '@context': 'https://schema.org',
    '@type': 'Product',
    name: product.title,
    description: toDescription(product.short_description || product.description, 5000),
    image: currentImages.map((src) => absoluteUrl(origin, src)),
    sku: currentVariant.sku || product.slug,
    url: productUrl,
    brand: { '@type': 'Brand', name: storeName },
    ...(product.category ? { category: product.category } : {}),
    offers: offerVariants.map((v) => ({
      '@type': 'Offer',
      url: productUrl,
      ...(v.sku ? { sku: v.sku } : {}),
      priceCurrency: 'EUR',
      price: (variantPrice(v) / 100).toFixed(2),
      availability: variantInStock(v) ? 'https://schema.org/InStock' : 'https://schema.org/OutOfStock',
      itemCondition: 'https://schema.org/NewCondition',
      seller: { '@type': 'Organization', name: storeName }
    }))
  };
  $: breadcrumbJsonLd = {
    '@context': 'https://schema.org',
    '@type': 'BreadcrumbList',
    itemListElement: [
      { '@type': 'ListItem', position: 1, name: storeName, item: `${origin}/` },
      ...(product.category
        ? [{ '@type': 'ListItem', position: 2, name: product.category, item: `${origin}/?category=${encodeURIComponent(product.category)}` }]
        : []),
      { '@type': 'ListItem', position: product.category ? 3 : 2, name: product.title, item: productUrl }
    ]
  };

  // When variant changes, ensure activeImageIndex is valid
  $: if (activeImageIndex >= currentImages.length) {
    activeImageIndex = 0;
  }

  function nextImage() {
    if (currentImages.length > 0) {
      activeImageIndex = (activeImageIndex + 1) % currentImages.length;
    }
  }

  function prevImage() {
    if (currentImages.length > 0) {
      activeImageIndex = (activeImageIndex - 1 + currentImages.length) % currentImages.length;
    }
  }

  function handleKeydown(e) {
    if (!isLightboxOpen) return;
    if (e.key === 'Escape') isLightboxOpen = false;
    if (e.key === 'ArrowRight') nextImage();
    if (e.key === 'ArrowLeft') prevImage();
  }

  // Simple Markdown parser for long description (supports headers, bold, italics, lists, blank lines)
  function renderMarkdown(md) {
    if (!md) return '';
    const lines = md.split('\n');
    let html = '';
    let inList = false;

    for (let i = 0; i < lines.length; i++) {
      let line = lines[i].trim();

      if (line.startsWith('- ') || line.startsWith('* ')) {
        if (!inList) {
          html += '<ul class="list-disc list-inside space-y-1 my-3 text-slate-300">';
          inList = true;
        }
        let itemText = line.substring(2);
        itemText = formatInline(itemText);
        html += `<li>${itemText}</li>`;
        continue;
      } else if (inList) {
        html += '</ul>';
        inList = false;
      }

      if (!line) {
        // Blank line creates paragraph spacing
        html += '<div class="h-4"></div>';
        continue;
      }

      if (line.startsWith('### ')) {
        html += `<h3 class="text-base font-bold text-white mt-5 mb-2">${formatInline(line.substring(4))}</h3>`;
      } else if (line.startsWith('## ')) {
        html += `<h2 class="text-lg font-extrabold text-white mt-6 mb-2.5">${formatInline(line.substring(3))}</h2>`;
      } else if (line.startsWith('# ')) {
        html += `<h1 class="text-xl font-black text-white mt-8 mb-3">${formatInline(line.substring(2))}</h1>`;
      } else {
        html += `<p class="my-2 text-slate-300 leading-relaxed">${formatInline(line)}</p>`;
      }
    }

    if (inList) {
      html += '</ul>';
    }

    return html;
  }

  function formatInline(text) {
    return text
      .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener noreferrer" class="text-orange-400 underline hover:text-orange-300 font-semibold transition-colors">$1</a>')
      .replace(/\*\*(.*?)\*\*/g, '<strong class="font-bold text-white">$1</strong>')
      .replace(/\*(.*?)\*/g, '<em class="italic text-slate-200">$1</em>')
      .replace(/`([^`]+)`/g, '<code class="px-1.5 py-0.5 rounded bg-slate-800 text-orange-400 font-mono text-xs">$1</code>');
  }

  async function toggleWishlist() {
    try {
      const headers = {
        'Content-Type': 'application/json',
        ...($customer ? { Authorization: `Bearer ${$customer.token}` } : {})
      };
      const res = await fetch('/api/v1/customer/wishlist/toggle', {
        method: 'POST',
        headers,
        body: JSON.stringify({ product_id: product.id })
      });
      if (res.ok) {
        const d = await res.json();
        isInWishlist = d.in_wishlist;
      }
    } catch (e) {
      console.error('Failed to toggle wishlist:', e);
    }
  }

  function addToCart() {
    if (!inStock) return;
    cart.addItem({
      variant_id: currentVariant.id,
      sku: currentVariant.sku,
      product_title: product.title,
      variant_title: currentVariant.title,
      price_cents: currentPriceCents,
      quantity,
      is_digital: isDigital,
      image_url: activeImageUrl,
      slug: product.slug
    });
    isCartOpen.set(true);
  }

  async function subscribeStockNotification() {
    if (!stockNotificationEmail || !stockNotificationEmail.includes('@')) return;
    subscribingStock = true;
    try {
      const res = await fetch(`/api/v1/products/${product.id}/notify-stock`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: stockNotificationEmail,
          variant_id: currentVariant.id || null
        })
      });
      if (res.ok) {
        stockNotificationSuccess = true;
      }
    } catch (e) {
      console.error('Failed to subscribe to stock notification:', e);
    } finally {
      subscribingStock = false;
    }
  }

  // Related products carousel scroll container
  let relatedCarouselEl;
  function scrollRelated(direction) {
    if (relatedCarouselEl) {
      const scrollAmount = relatedCarouselEl.clientWidth * 0.8;
      relatedCarouselEl.scrollBy({ left: direction * scrollAmount, behavior: 'smooth' });
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- Full SEO Tags: Title, Description, OpenGraph, Twitter, Canonical, Product JSON-LD -->
<Seo
  title={`${product.title} | ${storeName}`}
  description={product.short_description || product.description || `Buy ${product.title} at ${storeName}.`}
  path={`/products/${product.slug}`}
  image={currentImages[0] || product.image_url}
  type="product"
  jsonLd={[productJsonLd, breadcrumbJsonLd]}
/>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
  <!-- Breadcrumb -->
  <nav class="flex items-center gap-2 text-xs text-slate-400 mb-8">
    <a href="/" class="hover:text-orange-400 flex items-center gap-1 transition-colors">
      <ArrowLeft size={14} /> Back to Catalog
    </a>
    <span>/</span>
    <span class="text-slate-400">{product.category}</span>
    {#if product.subcategory}
      <span>/</span>
      <span class="text-slate-400">{product.subcategory}</span>
    {/if}
    <span>/</span>
    <span class="text-white font-medium truncate">{product.title}</span>
  </nav>

  <!-- Top Section: Media Gallery (Left) & Details / Purchase (Right) -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-10 lg:gap-14">
    <!-- Media Column with Multi-Image Gallery & Lightbox Trigger -->
    <div class="space-y-4">
      <!-- Main Featured Image Container with Lightbox Click -->
      <div class="relative aspect-square rounded-3xl overflow-hidden bg-slate-900 border border-slate-800 shadow-2xl group">
        {#if activeImageUrl}
          <button
            type="button"
            on:click={() => isLightboxOpen = true}
            class="w-full h-full block focus:outline-none cursor-zoom-in"
            aria-label="Enlarge image in fullscreen lightbox"
          >
            <img
              src={activeImageUrl}
              alt={product.title}
              class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
            />
            <!-- Maximize Button Overlay -->
            <div class="absolute bottom-4 right-4 p-2.5 rounded-xl bg-slate-950/70 backdrop-blur-md text-white border border-slate-700/60 opacity-0 group-hover:opacity-100 transition-opacity shadow-lg flex items-center gap-1.5 text-xs font-semibold">
              <Maximize2 size={15} class="text-orange-400" />
              <span>Fullscreen</span>
            </div>
          </button>
        {:else}
          <div class="w-full h-full flex items-center justify-center text-7xl text-slate-700">
            📦
          </div>
        {/if}

        {#if isDigital}
          <div class="absolute top-4 left-4 px-3 py-1 rounded-lg bg-sky-500 text-white text-xs font-bold flex items-center gap-1.5 shadow-md pointer-events-none">
            <Download size={13} /> Digital Masterclass & Asset
          </div>
        {/if}
      </div>

      <!-- Thumbnail Carousel / Switcher (if multiple images exist) -->
      {#if currentImages.length > 1}
        <div class="flex items-center gap-3 overflow-x-auto pb-2 scrollbar-none">
          {#each currentImages as imgUrl, idx}
            <button
              type="button"
              on:click={() => activeImageIndex = idx}
              class="relative w-20 h-20 rounded-xl overflow-hidden p-0.5 border-2 transition-all flex-shrink-0 bg-slate-900 {activeImageIndex === idx ? 'border-orange-500 shadow-lg shadow-orange-500/25' : 'border-slate-800 hover:border-slate-600 opacity-60 hover:opacity-100'}"
            >
              <img src={imgUrl} alt={`Thumbnail ${idx + 1}`} class="w-full h-full object-cover rounded-[8px]" />
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Product Details Column -->
    <div class="flex flex-col justify-between">
      <div>
        <div class="flex items-center gap-2 mb-2">
          <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-orange-500/10 text-orange-400 border border-orange-500/20">
            {product.category} {#if product.subcategory}&bull; {product.subcategory}{/if}
          </span>
          {#if isLowStock}
            <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-500/10 text-amber-400 border border-amber-500/20">
              Low Stock Alert
            </span>
          {/if}
        </div>

        <h1 class="text-3xl sm:text-4xl font-black text-white tracking-tight">
          {product.title}
        </h1>

        <!-- Product Subtitle Underneath Title -->
        {#if product.subtitle}
          <p class="text-sm font-medium text-slate-400 mt-1.5 leading-snug">
            {product.subtitle}
          </p>
        {/if}

        <div class="mt-4 space-y-1">
          <div class="flex items-baseline gap-3 flex-wrap">
            <span class="text-3xl font-black font-mono {pricing.on_sale ? 'text-rose-400' : 'text-white'}">
              {formatEuro(currentPriceCents)}
            </span>
            {#if pricing.on_sale}
              <span class="text-lg font-mono text-slate-500 line-through" aria-label="Previous lowest price">{formatEuro(pricing.reference_cents)}</span>
              <span class="px-2 py-0.5 rounded-md bg-rose-600 text-white text-xs font-black">−{reductionPercent(pricing)} %</span>
            {/if}
          </div>
          {#if pricing.on_sale}
            <!-- § 11 PAngV: the reduction refers to the lowest price of the 30 days before it -->
            <p class="text-[11px] text-slate-400">
              Lowest price in the 30 days before the reduction: {formatEuro(pricing.reference_cents)}{#if pricing.sale_ends_at} · Offer ends {new Date(pricing.sale_ends_at).toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' })}{/if}
            </p>
          {/if}
          {#if pricing.unit_price}
            <p class="text-xs text-slate-400 font-mono">({pricing.unit_price})</p>
          {/if}
          <PriceNote {store} digital={isDigital} />
        </div>

        <!-- Short Description Next to Image -->
        {#if product.short_description || product.description}
          <div class="mt-6 prose prose-invert text-sm text-slate-300 leading-relaxed border-t border-b border-slate-800/80 py-4">
            {product.short_description || product.description}
          </div>
        {/if}

        <!-- Variants Selection with Customizable Selector Label (Only when product has multiple versions) -->
        {#if product.has_multiple_variants && variants.length > 1}
          <div class="mt-6 space-y-3">
            <div class="flex justify-between items-center text-xs">
              <span class="font-bold text-slate-300 uppercase tracking-wider">
                {product.variant_selector_label || 'Choose Variant / Model:'}
              </span>
              <span class="font-mono text-orange-400 font-semibold">SKU: {currentVariant.sku}</span>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
              {#each variants as variant, i}
                <button
                  type="button"
                  on:click={() => {
                    selectedVariantIndex = i;
                    activeImageIndex = 0;
                  }}
                  class="p-3.5 rounded-xl border text-left transition-all duration-200 flex items-center justify-between {selectedVariantIndex === i ? 'bg-orange-600/15 border-orange-500 ring-1 ring-orange-500 text-white' : 'bg-slate-900/60 border-slate-800 hover:border-slate-700 text-slate-300'}"
                >
                  <div class="min-w-0 pr-2">
                    <div class="text-xs font-bold truncate">{variant.title}</div>
                    <div class="text-[11px] font-mono text-slate-400 mt-0.5">{variant.sku}</div>
                  </div>
                  <div class="text-right flex-shrink-0">
                    <div class="text-xs font-mono font-bold {variantPricing(product, variant).on_sale ? 'text-rose-400' : ''}">
                      {formatEuro(variantPricing(product, variant).price_cents)}
                    </div>
                    {#if !isDigital}
                      <div class="text-[10px] {variant.stock_quantity <= 0 ? 'text-rose-400' : variant.stock_quantity <= variant.low_stock_threshold ? 'text-amber-400 font-bold' : 'text-slate-400'}">
                        {variant.stock_quantity <= 0 ? 'Out of stock' : `${variant.stock_quantity} in stock`}
                      </div>
                    {/if}
                  </div>
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Stock Availability Indicator -->
        <div class="mt-6 p-3.5 rounded-xl bg-slate-900/80 border border-slate-800 flex items-center justify-between text-xs">
          <div class="flex items-center gap-2">
            {#if isDigital}
              <div class="w-2.5 h-2.5 rounded-full bg-sky-400 animate-pulse"></div>
              <span class="font-semibold text-slate-200">Instant Digital Delivery link generated upon payment.</span>
            {:else if inStock}
              <div class="w-2.5 h-2.5 rounded-full bg-emerald-400"></div>
              <span class="font-semibold text-slate-200">
                {(store?.stock_display_template || 'In Stock ({stock} units available in central warehouse)').replace('{stock}', String(currentVariant.stock_quantity))}
              </span>
            {:else}
              <div class="w-2.5 h-2.5 rounded-full bg-rose-500"></div>
              <span class="font-semibold text-rose-400">Currently out of stock</span>
            {/if}
          </div>
        </div>

        <!-- Delivery time must be stated before the order (Art. 246a § 1 Nr. 7 EGBGB) -->
        {#if deliveryTime(product) || shippingInfo || formatWeight(weightGrams)}
          <div class="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-slate-400">
            {#if deliveryTime(product)}
              <span class="flex items-center gap-1.5"><Truck size={14} class="text-slate-500" /> {deliveryTime(product)}</span>
            {/if}
            {#if shippingInfo && !deliveryTime(product)}
              <span class="flex items-center gap-1.5 {shippingInfo.ships ? '' : 'text-amber-300'}">
                <Truck size={14} class="text-slate-500" />
                {#if !shippingInfo.ships}
                  We currently don't ship to {shippingInfo.countryName}.
                {:else if !shippingInfo.fits}
                  Too large for our standard parcels to {shippingInfo.countryName} — please contact us for shipping.
                {:else if shippingInfo.estimate}
                  Delivery to {shippingInfo.countryName}: {shippingInfo.estimate}
                {:else}
                  Ships to {shippingInfo.countryName} — delivery time shown at checkout
                {/if}
              </span>
            {/if}
            {#if !isDigital && formatWeight(weightGrams)}
              <span>Weight: {formatWeight(weightGrams)}</span>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Add to Cart Action -->
      <div class="mt-8 pt-6 border-t border-slate-800 flex flex-col sm:flex-row items-center gap-4">
        <!-- Quantity counter -->
        {#if !isDigital}
          <div class="flex items-center border border-slate-700 rounded-xl bg-slate-900 px-2 py-1 w-full sm:w-auto justify-between sm:justify-start">
            <button
              on:click={() => quantity = Math.max(1, quantity - 1)}
              class="p-2 text-slate-400 hover:text-white transition-colors"
              aria-label="Decrease quantity"
            >
              <Minus size={14} />
            </button>
            <span class="px-4 text-sm font-bold text-white font-mono">{quantity}</span>
            <button
              on:click={() => quantity = Math.min(currentVariant.stock_quantity || 10, quantity + 1)}
              class="p-2 text-slate-400 hover:text-white transition-colors"
              aria-label="Increase quantity"
            >
              <Plus size={14} />
            </button>
          </div>
        {/if}

        <button
          id="add-to-cart-btn"
          on:click={addToCart}
          disabled={!inStock}
          class="flex-1 w-full py-4 px-6 rounded-xl font-bold text-sm flex items-center justify-center gap-2 shadow-xl transition-all duration-200 {!inStock ? 'bg-slate-800 text-slate-500 cursor-not-allowed' : 'bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white shadow-orange-600/30 hover:scale-[1.01]'}"
        >
          {#if isDigital}
            <Download size={18} />
            <span>Add Digital License &bull; {(currentPriceCents / 100).toFixed(2)} €</span>
          {:else if inStock}
            <Box size={18} />
            <span>Add to Cart &bull; {((currentPriceCents * quantity) / 100).toFixed(2)} €</span>
          {:else}
            <span>Out of Stock</span>
          {/if}
        </button>

        <button
          type="button"
          on:click={toggleWishlist}
          class="p-4 rounded-xl border border-slate-800 bg-slate-900 hover:bg-slate-800 text-slate-400 hover:text-rose-400 transition-colors"
          title="Toggle Wishlist"
        >
          <Heart size={18} class={isInWishlist ? 'fill-rose-500 text-rose-500' : ''} />
        </button>
      </div>

      {#if !inStock && !isDigital}
        <div class="mt-4 p-4 rounded-2xl bg-orange-950/20 border border-orange-500/30 space-y-2">
          <div class="flex items-center gap-2 text-xs font-bold text-orange-400">
            <Bell size={14} />
            <span>Notify me when back in stock</span>
          </div>
          <p class="text-[11px] text-slate-400">Enter your email and our system will notify you the moment this item is restocked.</p>
          {#if stockNotificationSuccess}
            <div class="p-2.5 rounded-lg bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-xs font-semibold">
              ✓ You're on the waitlist! We will notify you once restocked.
            </div>
          {:else}
            <form on:submit|preventDefault={subscribeStockNotification} class="flex gap-2">
              <input
                type="email"
                bind:value={stockNotificationEmail}
                placeholder="your.email@example.com"
                required
                class="flex-1 px-3 py-2 rounded-xl bg-slate-900 border border-slate-700 text-white text-xs placeholder-slate-500 focus:outline-none focus:border-orange-500"
              />
              <button
                type="submit"
                disabled={subscribingStock}
                class="px-4 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs transition-colors shadow-sm disabled:opacity-50"
              >
                {subscribingStock ? 'Joining...' : 'Notify Me'}
              </button>
            </form>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <!-- Full-Width Long Description Section (Markdown with blank lines / paragraphs) -->
  {#if product.long_description}
    <div class="mt-16 pt-10 border-t border-slate-800">
      <div class="max-w-4xl mx-auto">
        <div class="flex items-center gap-2 mb-6">
          <Sparkles size={18} class="text-orange-400" />
          <h2 class="text-xl sm:text-2xl font-black text-white tracking-tight">Product Overview & Detailed Specifications</h2>
        </div>
        <div class="bg-slate-900/40 rounded-3xl p-6 sm:p-10 border border-slate-800/80 shadow-xl">
          <div class="text-sm sm:text-base leading-relaxed text-slate-300">
            {@html renderMarkdown(product.long_description)}
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Product safety information (EU General Product Safety Regulation, Art. 19 GPSR) -->
  {#if !isDigital && (manufacturer || product.gpsr_responsible_person || product.gpsr_safety_info)}
    <div class="mt-16 pt-10 border-t border-slate-800">
      <div class="max-w-4xl mx-auto">
        <div class="flex items-center gap-2 mb-4">
          <ShieldAlert size={18} class="text-orange-400" />
          <h2 class="text-xl font-black text-white tracking-tight">Product safety</h2>
        </div>
        <dl class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
          {#if manufacturer}
            <div class="p-4 rounded-2xl bg-slate-900/50 border border-slate-800">
              <dt class="text-[11px] font-bold uppercase tracking-wider text-slate-500 mb-1">Manufacturer</dt>
              <dd class="text-slate-300 whitespace-pre-line">{manufacturer}</dd>
            </div>
          {/if}
          {#if product.gpsr_responsible_person}
            <div class="p-4 rounded-2xl bg-slate-900/50 border border-slate-800">
              <dt class="text-[11px] font-bold uppercase tracking-wider text-slate-500 mb-1">Responsible person in the EU</dt>
              <dd class="text-slate-300 whitespace-pre-line">{product.gpsr_responsible_person}</dd>
            </div>
          {/if}
          <div class="p-4 rounded-2xl bg-slate-900/50 border border-slate-800">
            <dt class="text-[11px] font-bold uppercase tracking-wider text-slate-500 mb-1">Product identification</dt>
            <dd class="text-slate-300">{product.title}{#if currentVariant.sku} · SKU {currentVariant.sku}{/if}</dd>
          </div>
          {#if product.gpsr_safety_info}
            <div class="p-4 rounded-2xl bg-slate-900/50 border border-slate-800 md:col-span-2">
              <dt class="text-[11px] font-bold uppercase tracking-wider text-slate-500 mb-1">Warnings and safety information</dt>
              <dd class="text-slate-300 whitespace-pre-line">{product.gpsr_safety_info}</dd>
            </div>
          {/if}
        </dl>
      </div>
    </div>
  {/if}

  {#if product.id}
    <ProductReviews productId={product.id} />
  {/if}

  <!-- 5-Products Wide Carousel Underneath Description: "Related Products" -->
  {#if relatedProducts.length > 0}
    <div class="mt-20 pt-10 border-t border-slate-800">
      <div class="flex items-center justify-between mb-8">
        <div>
          <h2 class="text-xl sm:text-2xl font-black text-white tracking-tight">Related Products</h2>
          <p class="text-xs text-slate-400 mt-1">Customers who viewed this item also explored</p>
        </div>
        {#if relatedProducts.length > 5}
          <div class="flex items-center gap-2">
            <button
              on:click={() => scrollRelated(-1)}
              class="p-2.5 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 text-slate-300 hover:text-white transition-colors"
              aria-label="Previous related products"
            >
              <ChevronLeft size={18} />
            </button>
            <button
              on:click={() => scrollRelated(1)}
              class="p-2.5 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 text-slate-300 hover:text-white transition-colors"
              aria-label="Next related products"
            >
              <ChevronRight size={18} />
            </button>
          </div>
        {/if}
      </div>

      <!-- 5 items wide grid/carousel on desktop -->
      <div
        bind:this={relatedCarouselEl}
        class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 overflow-x-auto scrollbar-none pb-4"
      >
        {#each relatedProducts as relProduct}
          <div class="min-w-0">
            <ProductCard item={relProduct} variants={[]} />
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<!-- Fullscreen Lightbox Modal -->
{#if isLightboxOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-8 animate-in fade-in duration-200">
    <!-- Clickable Backdrop -->
    <button
      type="button"
      on:click={() => isLightboxOpen = false}
      class="absolute inset-0 bg-slate-950/95 backdrop-blur-md cursor-zoom-out w-full h-full border-0"
      aria-label="Close fullscreen image viewer"
    ></button>

    <!-- Close Button -->
    <button
      type="button"
      on:click={() => isLightboxOpen = false}
      class="absolute top-6 right-6 z-10 p-3 rounded-2xl bg-slate-900/80 hover:bg-slate-800 text-white border border-slate-700 transition-colors shadow-2xl"
      aria-label="Close lightbox"
    >
      <X size={22} />
    </button>

    <!-- Prev Button -->
    {#if currentImages.length > 1}
      <button
        type="button"
        on:click|stopPropagation={prevImage}
        class="absolute left-4 sm:left-8 z-10 p-3.5 rounded-2xl bg-slate-900/80 hover:bg-slate-800 text-white border border-slate-700 transition-colors shadow-2xl"
        aria-label="Previous image"
      >
        <ChevronLeft size={24} />
      </button>
    {/if}

    <!-- Next Button -->
    {#if currentImages.length > 1}
      <button
        type="button"
        on:click|stopPropagation={nextImage}
        class="absolute right-4 sm:right-8 z-10 p-3.5 rounded-2xl bg-slate-900/80 hover:bg-slate-800 text-white border border-slate-700 transition-colors shadow-2xl"
        aria-label="Next image"
      >
        <ChevronRight size={24} />
      </button>
    {/if}

    <!-- Maximized Image Container -->
    <div class="relative z-10 max-w-5xl max-h-[85vh] flex flex-col items-center justify-center">
      <img
        src={activeImageUrl}
        alt={product.title}
        class="max-w-full max-h-[80vh] object-contain rounded-2xl shadow-2xl border border-slate-800"
      />
      {#if currentImages.length > 1}
        <div class="mt-4 px-4 py-1.5 rounded-full bg-slate-900/90 border border-slate-800 text-xs font-mono text-slate-300">
          {activeImageIndex + 1} / {currentImages.length}
        </div>
      {/if}
    </div>
  </div>
{/if}
