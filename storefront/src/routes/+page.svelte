<script>
  import { onMount, onDestroy } from "svelte";
  import ProductCard from "$lib/components/ProductCard.svelte";
  import Seo from "$lib/components/Seo.svelte";
  import { page } from "$app/stores";
  import { absoluteUrl } from "$lib/seo.js";
  import {
    ArrowRight,
    ChevronLeft,
    ChevronRight,
    Sparkles
  } from "lucide-svelte";

  export let data;
  $: products = data.products || [];
  $: carousels = data.carousels || [];
  $: categories = data.categories || [];
  $: currentCategory = data.currentCategory || "";
  $: currentSubcategory = data.currentSubcategory || "";
  $: currentSearch = data.currentSearch || "";

  // --- SEO ---
  $: store = data.store || {};
  $: storeName = store.store_name || "Shop";
  $: origin = $page.url.origin;
  $: seoTitle = currentCategory
    ? `${currentSubcategory || currentCategory} | ${storeName}`
    : currentSearch
      ? `Search: ${currentSearch} | ${storeName}`
      : store.store_subtitle ? `${storeName} | ${store.store_subtitle}` : storeName;
  $: seoDescription = currentCategory
    ? (matchedCategoryNode?.description || `Shop ${currentSubcategory || currentCategory} at ${storeName}.`)
    : (store.store_subtitle || `Welcome to ${storeName} — browse our products and order online.`);
  $: seoPath = currentCategory
    ? `/?category=${encodeURIComponent(currentCategory)}${currentSubcategory ? `&subcategory=${encodeURIComponent(currentSubcategory)}` : ""}`
    : "/";
  $: organizationJsonLd = {
    "@context": "https://schema.org",
    "@type": "Organization",
    name: store.legal_name || storeName,
    url: `${origin}/`,
    ...(store.logo_url ? { logo: absoluteUrl(origin, store.logo_url) } : {}),
    ...(store.support_email ? { email: store.support_email } : {}),
    ...(store.phone ? { telephone: store.phone } : {})
  };
  $: websiteJsonLd = {
    "@context": "https://schema.org",
    "@type": "WebSite",
    name: storeName,
    url: `${origin}/`,
    potentialAction: {
      "@type": "SearchAction",
      target: `${origin}/?search={search_term_string}`,
      "query-input": "required name=search_term_string"
    }
  };
  $: matchedCategoryNode = categories.find(
    (c) => c.name.toLowerCase() === currentCategory.toLowerCase() || c.slug.toLowerCase() === currentCategory.toLowerCase()
  );
  $: subcategories = matchedCategoryNode ? (matchedCategoryNode.children || []) : [];
  $: heroConfig = data.heroConfig || {
    layout: "split",
    carousel_items: [
      {
        id: "c1",
        title: "RustCraft Pro 75% Mechanical Keyboard",
        subtitle: "Hot-swappable CNC anodized aluminum chassis with QMK/VIA firmware",
        image_url: "https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=1400&q=80",
        link_url: "/products/rust-mechanical-keyboard",
        button_text: "Discover Precision"
      },
      {
        id: "c2",
        title: "Rustacean Heavyweight Hoodie",
        subtitle: "480 GSM French Terry organic cotton with embroidered Ferris insignia",
        image_url: "https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=1400&q=80",
        link_url: "/products/rustacean-heavyweight-hoodie",
        button_text: "Gear Up"
      }
    ],
    featured_buttons: [
      {
        id: "b1",
        title: "Mechanical Keyboards",
        subtitle: "Tactile Switches & Aluminum",
        price: "189.00 €",
        show_price: true,
        image_url: "https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/rust-mechanical-keyboard",
        bg_color: "#ea580c"
      },
      {
        id: "b2",
        title: "Heavyweight Hoodies",
        subtitle: "480 GSM Organic Cotton",
        price: "79.00 €",
        show_price: true,
        image_url: "https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/rustacean-heavyweight-hoodie",
        bg_color: "#0284c7"
      },
      {
        id: "b3",
        title: "Architecture Guide",
        subtitle: "Zero-Cost Concurrency eBook",
        price: "29.00 €",
        show_price: true,
        image_url: "https://images.unsplash.com/photo-1532012164546-f432f2e37b73?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/zero-cost-abstractions-guide",
        bg_color: "#16a34a"
      },
      {
        id: "b4",
        title: "Aviator Coiled Cables",
        subtitle: "Double-Sleeved USB-C",
        price: "34.00 €",
        show_price: true,
        image_url: "https://images.unsplash.com/photo-1544716278-ca5e3f4abd8c?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/rust-mechanical-keyboard",
        bg_color: "#7c3aed"
      }
    ]
  };

  let activeSlide = 0;
  let autoRotateTimer = null;
  let productCarouselTimer = null;
  let isCarouselHovered = false;
  let hoveredSectionId = null;

  $: carouselItems = (heroConfig && heroConfig.carousel_items && heroConfig.carousel_items.length > 0)
    ? heroConfig.carousel_items
    : [];

  onMount(() => {
    startAutoRotate();
  });

  onDestroy(() => {
    stopAutoRotate();
  });

  function startAutoRotate() {
    stopAutoRotate();
    autoRotateTimer = setInterval(() => {
      if (!isCarouselHovered && carouselItems.length > 1) {
        nextSlide();
      }
    }, 6500); // Hero slides auto-rotate every 6.5s

    // Auto-scroll product carousels every 7 seconds to the next product
    productCarouselTimer = setInterval(() => {
      if (carousels && carousels.length > 0) {
        for (const section of carousels) {
          if (section.items && section.items.length > 5 && hoveredSectionId !== section.id) {
            scrollCarousel(section.id, 1);
          }
        }
      }
    }, 7000);
  }

  function stopAutoRotate() {
    if (autoRotateTimer) {
      clearInterval(autoRotateTimer);
      autoRotateTimer = null;
    }
    if (productCarouselTimer) {
      clearInterval(productCarouselTimer);
      productCarouselTimer = null;
    }
  }

  function nextSlide() {
    if (carouselItems.length === 0) return;
    activeSlide = (activeSlide + 1) % carouselItems.length;
  }

  function prevSlide() {
    if (carouselItems.length === 0) return;
    activeSlide = (activeSlide - 1 + carouselItems.length) % carouselItems.length;
  }

  function scrollCarousel(sectionId, direction) {
    const el = document.getElementById(`carousel-${sectionId}`);
    if (el) {
      const scrollAmount = 300;
      const maxScroll = el.scrollWidth - el.clientWidth;
      if (direction > 0) {
        if (el.scrollLeft >= maxScroll - 20) {
          // Endlessly loop back to the start
          el.scrollTo({ left: 0, behavior: 'smooth' });
        } else {
          el.scrollBy({ left: scrollAmount, behavior: 'smooth' });
        }
      } else {
        if (el.scrollLeft <= 20) {
          // Loop to the end
          el.scrollTo({ left: maxScroll, behavior: 'smooth' });
        } else {
          el.scrollBy({ left: -scrollAmount, behavior: 'smooth' });
        }
      }
    }
  }
</script>

<Seo
  title={seoTitle}
  description={seoDescription}
  path={seoPath}
  image={store.logo_url}
  noindex={!!currentSearch}
  jsonLd={currentCategory || currentSearch ? [] : [organizationJsonLd, websiteJsonLd]}
/>

<!-- Dynamic Hero Showcase -->
{#if !currentCategory && !currentSearch && heroConfig && carouselItems.length > 0}
  <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-6 pb-6">
    {#if heroConfig.layout === 'carousel'}
      <!-- Option A: Full-Width Widescreen Carousel -->
      <div
        role="region"
        aria-label="Product Showcase Carousel"
        on:mouseenter={() => isCarouselHovered = true}
        on:mouseleave={() => isCarouselHovered = false}
        class="relative rounded-3xl overflow-hidden shadow-2xl bg-slate-950 border border-slate-800/80 aspect-[21/9] min-h-[380px] max-h-[560px]"
      >
        {#each carouselItems as item, idx}
          <div
            class="absolute inset-0 transition-opacity duration-700 ease-in-out {idx === activeSlide ? 'opacity-100 z-10' : 'opacity-0 z-0 pointer-events-none'}"
          >
            <!-- Background Image -->
            <img
              src={item.image_url}
              alt={item.title}
              class="w-full h-full object-cover object-center"
            />
            <!-- Gradient Overlay -->
            <div class="absolute inset-0 bg-gradient-to-r from-slate-950/95 via-slate-950/70 to-transparent"></div>

            <!-- Content -->
            <div class="absolute inset-0 flex flex-col justify-center px-8 sm:px-16 max-w-2xl z-20">
              <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-orange-600/20 text-orange-400 border border-orange-500/30 text-xs font-bold uppercase tracking-wider mb-4 w-fit">
                <Sparkles size={13} /> Featured Hardware
              </span>
              <h1 class="text-3xl sm:text-5xl font-black text-white tracking-tight leading-tight drop-shadow-md">
                {item.title}
              </h1>
              <p class="text-xs sm:text-sm text-slate-300 mt-3 leading-relaxed drop-shadow line-clamp-3">
                {item.subtitle}
              </p>
              <div class="mt-6">
                <a
                  href={item.link_url || '/'}
                  class="inline-flex items-center gap-2 px-6 py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs sm:text-sm shadow-xl shadow-orange-600/30 transition-all hover:scale-105"
                >
                  <span>{item.button_text || 'Explore Now'}</span>
                  <ArrowRight size={16} />
                </a>
              </div>
            </div>
          </div>
        {/each}

        <!-- Arrows -->
        {#if carouselItems.length > 1}
          <button
            on:click={prevSlide}
            aria-label="Previous Slide"
            class="absolute left-4 top-1/2 -translate-y-1/2 z-30 p-2.5 rounded-full bg-slate-950/70 hover:bg-orange-600 text-white transition-all backdrop-blur-md border border-slate-700/60"
          >
            <ChevronLeft size={20} />
          </button>
          <button
            on:click={nextSlide}
            aria-label="Next Slide"
            class="absolute right-4 top-1/2 -translate-y-1/2 z-30 p-2.5 rounded-full bg-slate-950/70 hover:bg-orange-600 text-white transition-all backdrop-blur-md border border-slate-700/60"
          >
            <ChevronRight size={20} />
          </button>

          <!-- Dot Indicators -->
          <div class="absolute bottom-5 left-1/2 -translate-x-1/2 z-30 flex items-center gap-2">
            {#each carouselItems as _, idx}
              <button
                on:click={() => activeSlide = idx}
                aria-label="Slide {idx + 1}"
                class="h-2 rounded-full transition-all {idx === activeSlide ? 'w-8 bg-orange-500' : 'w-2 bg-white/40 hover:bg-white/70'}"
              ></button>
            {/each}
          </div>
        {/if}
      </div>

    {:else}
      <!-- Option B: Split Hero (60% Carousel + 40% 4 Featured Product Buttons) -->
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-5 items-stretch">
        <!-- Left: 60% Width Carousel Slider (7 cols on lg) -->
        <div
          role="region"
          aria-label="Product Showcase Carousel"
          on:mouseenter={() => isCarouselHovered = true}
          on:mouseleave={() => isCarouselHovered = false}
          class="lg:col-span-7 relative rounded-3xl overflow-hidden shadow-2xl bg-slate-950 border border-slate-800/80 min-h-[360px] lg:min-h-[440px]"
        >
          {#each carouselItems as item, idx}
            <div
              class="absolute inset-0 transition-opacity duration-700 ease-in-out {idx === activeSlide ? 'opacity-100 z-10' : 'opacity-0 z-0 pointer-events-none'}"
            >
              <img
                src={item.image_url}
                alt={item.title}
                class="w-full h-full object-cover object-center"
              />
              <div class="absolute inset-0 bg-gradient-to-t from-slate-950 via-slate-950/60 to-transparent"></div>

              <div class="absolute inset-x-0 bottom-0 p-6 sm:p-8 z-20">
                <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-orange-600/30 text-orange-400 border border-orange-500/40 text-[10px] font-bold uppercase tracking-wider mb-2">
                  <Sparkles size={11} /> Featured Release
                </span>
                <h2 class="text-xl sm:text-3xl font-black text-white tracking-tight leading-snug drop-shadow-md">
                  {item.title}
                </h2>
                <p class="text-xs text-slate-300 mt-2 line-clamp-2 drop-shadow">
                  {item.subtitle}
                </p>
                <div class="mt-4">
                  <a
                    href={item.link_url || '/'}
                    class="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all hover:scale-105"
                  >
                    <span>{item.button_text || 'View Product'}</span>
                    <ArrowRight size={14} />
                  </a>
                </div>
              </div>
            </div>
          {/each}

          <!-- Arrows & Dots -->
          {#if carouselItems.length > 1}
            <button
              on:click={prevSlide}
              aria-label="Previous Slide"
              class="absolute left-3 top-1/2 -translate-y-1/2 z-30 p-2 rounded-full bg-slate-950/70 hover:bg-orange-600 text-white transition-all backdrop-blur-md border border-slate-700/60"
            >
              <ChevronLeft size={16} />
            </button>
            <button
              on:click={nextSlide}
              aria-label="Next Slide"
              class="absolute right-3 top-1/2 -translate-y-1/2 z-30 p-2 rounded-full bg-slate-950/70 hover:bg-orange-600 text-white transition-all backdrop-blur-md border border-slate-700/60"
            >
              <ChevronRight size={16} />
            </button>
            <div class="absolute top-4 right-4 z-30 flex items-center gap-1.5">
              {#each carouselItems as _, idx}
                <button
                  on:click={() => activeSlide = idx}
                  aria-label="Slide {idx + 1}"
                  class="h-1.5 rounded-full transition-all {idx === activeSlide ? 'w-6 bg-orange-500' : 'w-2 bg-white/40'}"
                ></button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Right: 40% Width 4 Featured Product Buttons (floating product images with depth) -->
        <div class="lg:col-span-5 grid grid-cols-1 sm:grid-cols-2 gap-4">
          {#if heroConfig.featured_buttons && heroConfig.featured_buttons.length > 0}
            {#each heroConfig.featured_buttons as btn}
              <a
                href={btn.link_url || '/'}
                class="group relative rounded-2xl overflow-hidden p-4 sm:p-5 flex flex-col justify-start shadow-xl transition-all duration-300 hover:scale-[1.02] hover:shadow-2xl border border-white/15 min-h-[185px]"
                style="background-color: {btn.bg_color || '#ea580c'};"
              >
                <!-- Depth Gradient & Highlights -->
                <div class="absolute inset-0 bg-gradient-to-br from-white/15 via-transparent to-black/35 pointer-events-none"></div>

                <!-- Floating Product Image in Bottom Right Corner (66% - 75% of button) -->
                {#if btn.image_url}
                  <div class="absolute right-[-2%] bottom-[-2%] w-[70%] h-[72%] z-10 transition-transform duration-300 group-hover:scale-105 flex items-end justify-end pointer-events-none">
                    <img
                      src={btn.image_url}
                      alt={btn.title}
                      class="max-w-full max-h-full object-contain object-right-bottom drop-shadow-[0_16px_24px_rgba(0,0,0,0.6)]"
                    />
                  </div>
                {/if}

                <!-- Text Header (Full Width at Top) -->
                <div class="relative z-20 w-full">
                  <h3 class="text-sm sm:text-base font-black text-white leading-tight drop-shadow group-hover:underline">
                    {btn.title}
                  </h3>
                  {#if btn.subtitle}
                    <p class="text-[11px] text-white/90 font-medium mt-1 leading-snug drop-shadow line-clamp-2">
                      {btn.subtitle}
                    </p>
                  {/if}

                  <!-- Price Directly Underneath Subtitle at the Top -->
                  {#if btn.show_price !== false && btn.price}
                    <div class="mt-2.5">
                      <span class="inline-block text-[11px] sm:text-xs font-mono font-black text-white bg-black/40 backdrop-blur-md px-2.5 py-0.5 rounded-lg border border-white/15 shadow-sm">
                        {btn.price}
                      </span>
                    </div>
                  {/if}
                </div>
              </a>
            {/each}
          {/if}
        </div>
      </div>
    {/if}
  </section>
{/if}

<!-- Homepage Mode: Dynamic 5-per-row Carousels Sections -->
{#if !currentCategory && !currentSearch && carousels && carousels.length > 0}
  <div class="space-y-6 pb-12">
    {#each carousels as section}
      {#if section.items && section.items.length > 0}
        <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div class="flex items-center justify-between mb-4">
            <div>
              <h2 class="text-xl sm:text-2xl font-black text-white tracking-tight flex items-center gap-2">
                <span class="w-2 h-5 rounded-full bg-orange-500"></span>
                {section.title}
              </h2>
              <span class="text-[11px] text-slate-400 ml-4 font-mono">{section.items.length} items</span>
            </div>

            <!-- Scroll arrows if carousel has more than 5 items -->
            {#if section.items.length > 5}
              <div class="flex items-center gap-1.5">
                <button
                  on:click={() => scrollCarousel(section.id, -1)}
                  class="p-2 rounded-xl bg-slate-900/90 hover:bg-slate-800 text-slate-300 hover:text-white border border-slate-800 transition-colors shadow-sm"
                  aria-label="Scroll left"
                >
                  <ChevronLeft size={16} />
                </button>
                <button
                  on:click={() => scrollCarousel(section.id, 1)}
                  class="p-2 rounded-xl bg-slate-900/90 hover:bg-slate-800 text-slate-300 hover:text-white border border-slate-800 transition-colors shadow-sm"
                  aria-label="Scroll right"
                >
                  <ChevronRight size={16} />
                </button>
              </div>
            {/if}
          </div>

          {#if section.items.length <= 5}
            <!-- 5 products per row, centered layout -->
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3.5 sm:gap-4 justify-center">
              {#each section.items as itemWithV}
                <ProductCard item={itemWithV.product || itemWithV} variants={itemWithV.variants} />
              {/each}
            </div>
          {:else}
            <!-- Smooth scrollable horizontal carousel with endless loop and hover detection -->
            <div
              id="carousel-{section.id}"
              on:mouseenter={() => hoveredSectionId = section.id}
              on:mouseleave={() => hoveredSectionId = null}
              class="flex gap-3.5 sm:gap-4 overflow-x-auto scrollbar-none snap-x snap-mandatory scroll-smooth pb-3"
            >
              {#each section.items as itemWithV}
                <div class="w-[210px] sm:w-[230px] flex-shrink-0 snap-start">
                  <ProductCard item={itemWithV.product || itemWithV} variants={itemWithV.variants} />
                </div>
              {/each}
            </div>
          {/if}
        </section>
      {/if}
    {/each}
  </div>
{:else}
  <!-- Filtered Search / Category Results Grid -->
  <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h2 class="text-xl sm:text-2xl font-black text-white tracking-tight flex items-center gap-2">
          <span class="w-2 h-5 rounded-full bg-orange-500"></span>
          {currentCategory ? currentCategory : (currentSearch ? `Search: "${currentSearch}"` : "All Products")}
        </h2>
        <p class="text-xs text-slate-400 mt-1">
          Showing {products.length} products available for immediate dispatch
        </p>
      </div>
      {#if currentCategory || currentSearch}
        <a
          href="/"
          class="text-xs text-orange-400 hover:text-orange-300 font-semibold"
        >
          &larr; Back to Overview
        </a>
      {/if}
    </div>

    <!-- Visual Subcategory Cards with Images -->
    {#if currentCategory && subcategories.length > 0}
      <div class="mb-8 p-4 rounded-2xl bg-slate-900/40 border border-slate-800/80">
        <h3 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-3">Explore Subcategories</h3>
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3">
          {#each subcategories as sub}
            <a
              href="/?category={encodeURIComponent(currentCategory)}&subcategory={encodeURIComponent(sub.name)}"
              class="group p-3 rounded-xl bg-slate-950/70 border {currentSubcategory === sub.name ? 'border-orange-500 bg-orange-500/10' : 'border-slate-800 hover:border-slate-700'} flex flex-col items-center text-center transition-all hover:scale-[1.02]"
            >
              <div class="w-16 h-16 rounded-xl overflow-hidden bg-slate-900 mb-2 flex items-center justify-center border border-slate-800">
                {#if sub.image_url}
                  <img src={sub.image_url} alt={sub.name} class="w-full h-full object-cover group-hover:scale-105 transition-transform" />
                {:else}
                  <span class="text-2xl text-slate-600">📁</span>
                {/if}
              </div>
              <span class="text-xs font-bold text-white group-hover:text-orange-400 truncate w-full">{sub.name}</span>
            </a>
          {/each}
        </div>
      </div>
    {/if}

    {#if products.length === 0}
      <div class="text-center py-20 bg-slate-900/40 rounded-3xl border border-slate-800/80">
        <div class="text-4xl mb-3">🔍</div>
        <h3 class="text-lg font-bold text-white">No products match your filter</h3>
        <p class="text-xs text-slate-400 mt-1">
          Try resetting the category filter or searching for another term.
        </p>
        <a
          href="/"
          class="inline-block mt-4 px-5 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold"
        >
          Reset Filters
        </a>
      </div>
    {:else}
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3.5 sm:gap-4">
        {#each products as itemWithV}
          <ProductCard item={itemWithV.product || itemWithV} variants={itemWithV.variants} />
        {/each}
      </div>
    {/if}
  </section>
{/if}
