<script>
  import '../app.css';
  import { cart, cartCount, cartSubtotal, isCartOpen } from '$lib/stores/cart.js';
  import { customer } from '$lib/stores/customer.js';
  import CookieBanner from '$lib/components/CookieBanner.svelte';
  import NewsletterSignup from '$lib/components/NewsletterSignup.svelte';
  import { page } from '$app/stores';
  import { PRIVATE_PATH } from '$lib/seo.js';
  import {
    ShoppingBag,
    X,
    Plus,
    Minus,
    ArrowRight,
    ShieldCheck,
    Box,
    Search,
    User,
    ChevronDown,
    ChevronRight,
    Heart,
    MapPin,
    Settings,
    LogOut,
    KeyRound,
    LogIn,
    Mail,
    Phone,
    Download
  } from 'lucide-svelte';

  export let data;
  $: store = data.store || {};
  $: isPrivatePage = PRIVATE_PATH.test($page.url.pathname);
  $: currencySymbol = store.currency_symbol || '€';
  $: headerMenu = data.headerMenu || data.menuItems || [];
  $: footerMenu = data.footerMenu || [];

  $: footerCfg = store.footer_config || {
    branding_mode: 'full',
    menu_layout: 'columns',
    columns: [
      {
        title: 'Customer Service',
        links: [
          { label: 'Shipping Policy & Rates', url: '/policies/shipment-policy' },
          { label: 'Return Policy', url: '/policies/return-policy' },
          { label: 'Revocation Policy & Form', url: '/policies/revocation-policy' },
          { label: 'Track Order', url: '/track' }
        ]
      },
      {
        title: 'Legal & Privacy',
        links: [
          { label: 'Legal Notice (Impressum)', url: '/policies/legal-notice' },
          { label: 'Terms and Conditions (AGB)', url: '/policies/terms-conditions' },
          { label: 'Privacy Policy (GDPR)', url: '/policies/privacy-policy' },
          { label: 'Cookie Policy', url: '/policies/cookie-policy' }
        ]
      },
      {
        title: 'Store & Support',
        links: [
          { label: 'Contact Information', url: '/policies/contact' }
        ]
      }
    ],
    social_links: {
      github: 'https://github.com',
      twitter: 'https://x.com',
      instagram: '',
      youtube: '',
      facebook: '',
      discord: 'https://discord.gg',
      whatsapp: ''
    },
    enabled_socials: ['github', 'twitter', 'discord'],
    show_socials: true,
    show_payments: true,
    copyright_format: 'standard',
    custom_copyright: ''
  };

  let isAccountMenuOpen = false;
  let headerSearch = '';

  function toggleAccountMenu() {
    isAccountMenuOpen = !isAccountMenuOpen;
  }

  function handleLogout() {
    customer.logout();
    isAccountMenuOpen = false;
  }
</script>

<svelte:head>
  {#if isPrivatePage}
    <meta name="robots" content="noindex, nofollow" />
  {/if}
  <meta name="theme-color" content="#020617" />
  {#if store.logo_url}
    <link rel="icon" href={store.logo_url} />
    <link rel="apple-touch-icon" href={store.logo_url} />
  {:else}
    <link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>🦀</text></svg>" />
  {/if}
</svelte:head>

<svelte:window on:click={(e) => {
  if (!e.target.closest('#account-dropdown-container')) {
    isAccountMenuOpen = false;
  }
}} />

<div class="min-h-screen flex flex-col bg-slate-950 text-slate-100 selection:bg-orange-500 selection:text-white">
  <!-- Top Announcement / Debug Bar (No admin button) -->
  {#if store.debug_mode}
    {@const liveStripe = (data.paymentProviders || []).some(p => p.provider === 'stripe' && !p.is_sandbox && p.is_enabled)}
    <div class="bg-gradient-to-r {liveStripe ? 'from-emerald-700 to-teal-800' : 'from-orange-600 to-amber-600'} px-4 py-1 text-center text-xs font-semibold tracking-wide text-white flex items-center justify-center gap-2 shadow-sm">
      <span class="bg-white/20 px-1.5 py-0.5 rounded text-[10px] uppercase font-bold tracking-wider">{liveStripe ? 'Stripe Live Active' : 'Sandbox Active'}</span>
      <span>{liveStripe ? 'Live Stripe payment gateway connected with live bank network authentication.' : 'Simulated payment test mode enabled for sandbox providers.'}</span>
    </div>
  {/if}

  <!-- Unified Sticky Header with Left Vertically Centered Logo, Middle Stacked Search & Nav, Right Actions -->
  <header class="sticky top-0 z-40 bg-slate-950/90 backdrop-blur-md border-b border-slate-900 shadow-xl transition-all">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-3 sm:py-3.5 flex items-center justify-between gap-4 lg:gap-8">
      <!-- Left: Brand Logo & Optional Title (Vertically Centered) -->
      <a href="/" class="flex items-center gap-3.5 group flex-shrink-0">
        {#if store.logo_url}
          <img
            src={store.logo_url}
            alt={store.store_name || 'Logo'}
            class="{store.show_store_title === false ? 'h-14 sm:h-16 max-w-[260px]' : 'h-10 max-w-[160px]'} object-contain rounded-xl transition-all duration-300"
          />
        {:else}
          <div class="{store.show_store_title === false ? 'w-14 h-14 rounded-2xl shadow-orange-600/30 text-3xl' : 'w-10 h-10 rounded-xl shadow-orange-600/20 text-2xl'} bg-gradient-to-tr from-orange-600 to-amber-500 flex items-center justify-center shadow-lg group-hover:scale-105 transition-transform duration-200">
            <span class="select-none">🦀</span>
          </div>
        {/if}

        {#if store.show_store_title !== false}
          <div>
            <span class="text-lg font-extrabold tracking-tight bg-gradient-to-r from-white via-slate-100 to-slate-400 bg-clip-text text-transparent">
              {store.store_name || 'RustCraft'}
            </span>
            {#if store.show_store_subtitle !== false}
              <div class="text-[9px] text-orange-400/90 font-mono font-semibold tracking-widest uppercase">
                {store.store_subtitle || 'Rust Powered • ACID Fast'}
              </div>
            {/if}
          </div>
        {/if}
      </a>

      <!-- Middle: Stacked Search Bar (Top) & Nav Links (Bottom) aligned to the same left line -->
      <div class="flex-1 flex flex-col justify-center min-w-0 max-w-2xl mx-2 sm:mx-6">
        <!-- Top: Search Bar -->
        <form action="/" method="GET" class="relative flex items-center w-full mb-1">
          <input
            type="text"
            name="search"
            bind:value={headerSearch}
            placeholder="Search products by title, SKU, or category..."
            class="w-full pl-9 pr-20 py-1.5 sm:py-2 rounded-xl bg-slate-900/90 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500/30 text-xs shadow-inner"
          />
          <Search size={14} class="absolute left-3 text-slate-500 pointer-events-none" />
          <button
            type="submit"
            class="absolute right-1 px-3 py-1 rounded-lg bg-orange-600 hover:bg-orange-500 text-white text-[11px] font-bold transition-all shadow-sm"
          >
            Search
          </button>
        </form>

        <!-- Bottom: Dynamic Customizable Navigation Menu with Dropdown Support -->
        <nav class="flex items-center gap-1 sm:gap-2 text-xs font-medium pt-0.5 pl-3 sm:pl-4 overflow-visible">
          {#each headerMenu as item}
            {#if item.children && item.children.length > 0}
              <!-- Dropdown Menu Item -->
              <div class="relative group/nav py-0.5">
                <a
                  href={item.url || '#'}
                  class="px-2.5 py-1.5 rounded-lg text-slate-300 hover:text-white hover:bg-slate-900/80 transition-all flex items-center gap-1 whitespace-nowrap"
                >
                  <span>{item.label}</span>
                  <ChevronDown size={12} class="text-slate-500 group-hover/nav:text-orange-400 group-hover/nav:rotate-180 transition-transform duration-200" />
                </a>

                <!-- Dropdown Sub-menu Floating Card (Layer 2) -->
                <div class="absolute left-0 top-full pt-1 hidden group-hover/nav:block z-50 animate-in fade-in slide-in-from-top-1 duration-150">
                  <div class="w-60 rounded-xl bg-slate-900/95 backdrop-blur-md border border-slate-800 shadow-2xl p-1.5 space-y-0.5 ring-1 ring-white/5">
                    {#each item.children as sub}
                      {#if sub.children && sub.children.length > 0}
                        <!-- 2nd Layer Item with 3rd Layer Flyout -->
                        <div class="relative group/sublayer">
                          <a
                            href={sub.url}
                            class="flex items-center justify-between px-3 py-2 rounded-lg text-xs font-medium text-slate-300 hover:text-white hover:bg-orange-600/15 hover:text-orange-300 transition-all group/sub"
                          >
                            <span>{sub.label}</span>
                            <ChevronRight size={12} class="text-slate-400 group-hover/sublayer:text-orange-400 group-hover/sublayer:translate-x-0.5 transition-all" />
                          </a>

                          <!-- 3rd Layer Floating Card (Flyout) -->
                          <div class="absolute left-full top-0 pl-1.5 hidden group-hover/sublayer:block z-50 animate-in fade-in slide-in-from-left-1 duration-150">
                            <div class="w-56 rounded-xl bg-slate-900/95 backdrop-blur-md border border-slate-800 shadow-2xl p-1.5 space-y-0.5 ring-1 ring-white/5">
                              {#each sub.children as grand}
                                <a
                                  href={grand.url}
                                  class="flex items-center justify-between px-3 py-2 rounded-lg text-xs font-medium text-slate-300 hover:text-white hover:bg-orange-600/15 hover:text-orange-300 transition-all group/grand"
                                >
                                  <span>{grand.label}</span>
                                  <ArrowRight size={11} class="text-orange-400 opacity-60 group-hover/grand:opacity-100 group-hover/grand:translate-x-0.5 transition-all" />
                                </a>
                              {/each}
                            </div>
                          </div>
                        </div>
                      {:else}
                        <a
                          href={sub.url}
                          class="flex items-center justify-between px-3 py-2 rounded-lg text-xs font-medium text-slate-300 hover:text-white hover:bg-orange-600/15 hover:text-orange-300 transition-all group/sub"
                        >
                          <span>{sub.label}</span>
                          <ArrowRight size={11} class="text-orange-400 opacity-60 group-hover/sub:opacity-100 group-hover/sub:translate-x-0.5 transition-all" />
                        </a>
                      {/if}
                    {/each}
                  </div>
                </div>
              </div>
            {:else}
              <a
                href={item.url}
                class="px-2.5 py-1.5 rounded-lg text-slate-300 hover:text-white hover:bg-slate-900/80 transition-all flex items-center gap-1 whitespace-nowrap"
              >
                <span>{item.label}</span>
              </a>
            {/if}
          {/each}
        </nav>
      </div>

      <!-- Right Header Actions (Account Dropdown + Cart) -->
      <div class="flex items-center gap-3">
        <!-- Account Dropdown Container -->
        <div class="relative" id="account-dropdown-container">
          <button
            type="button"
            on:click|stopPropagation={toggleAccountMenu}
            class="p-2 sm:px-3 sm:py-2 rounded-xl bg-slate-900/80 hover:bg-slate-800 border border-slate-800 hover:border-slate-700 text-slate-200 hover:text-white transition-all flex items-center gap-2 group text-xs font-semibold"
            aria-label="Account options"
          >
            <div class="w-6 h-6 rounded-full bg-slate-800 group-hover:bg-orange-500/20 text-slate-300 group-hover:text-orange-400 flex items-center justify-center transition-colors">
              <User size={14} />
            </div>
            <span class="hidden md:inline text-xs font-medium text-slate-300 truncate max-w-[100px]">
              {#if $customer && $customer.isLoggedIn}
                {$customer.full_name || $customer.email.split('@')[0]}
              {:else}
                Account
              {/if}
            </span>
            <ChevronDown size={13} class="text-slate-500 group-hover:text-slate-300 transition-transform {isAccountMenuOpen ? 'rotate-180' : ''}" />
          </button>

          <!-- Dropdown Menu -->
          {#if isAccountMenuOpen}
            <div
              class="absolute right-0 mt-2 w-56 rounded-2xl bg-slate-900 border border-slate-800 shadow-2xl py-2 z-50 animate-in fade-in slide-in-from-top-2 duration-150"
            >
              {#if !$customer || !$customer.isLoggedIn}
                <!-- Logged Out State: Only 2 options as requested -->
                <div class="px-3 py-2 border-b border-slate-800/80 mb-1">
                  <p class="text-[11px] font-semibold text-slate-400">Welcome Customer</p>
                  <p class="text-xs text-slate-300">Sign in to view orders & wishlist</p>
                </div>
                <a
                  href="/account/login"
                  on:click={() => isAccountMenuOpen = false}
                  class="flex items-center gap-2.5 px-4 py-2.5 text-xs text-slate-200 hover:bg-slate-800/80 hover:text-orange-400 transition-colors font-semibold"
                >
                  <LogIn size={15} class="text-orange-400" />
                  <span>Login / Register</span>
                </a>
                <a
                  href="/account/reset-password"
                  on:click={() => isAccountMenuOpen = false}
                  class="flex items-center gap-2.5 px-4 py-2.5 text-xs text-slate-300 hover:bg-slate-800/80 hover:text-white transition-colors"
                >
                  <KeyRound size={15} class="text-slate-400" />
                  <span>Lost Password / Reset Password</span>
                </a>
              {:else}
                <!-- Logged In State: Orders, Wishlist, Addresses, Account details, Logout -->
                <div class="px-3 py-2 border-b border-slate-800/80 mb-1">
                  <p class="text-xs font-bold text-white truncate">{$customer.full_name || 'Customer'}</p>
                  <p class="text-[10px] text-slate-400 font-mono truncate">{$customer.email}</p>
                </div>
                <a
                  href="/account/orders"
                  on:click={() => isAccountMenuOpen = false}
                  class="flex items-center gap-2.5 px-4 py-2 text-xs text-slate-300 hover:bg-slate-800 hover:text-orange-400 transition-colors"
                >
                  <Box size={15} class="text-slate-400" />
                  <span>Orders</span>
                </a>
                <a
                  href="/account/downloads"
                  on:click={() => isAccountMenuOpen = false}
                  class="flex items-center gap-2.5 px-4 py-2 text-xs text-slate-300 hover:bg-slate-800 hover:text-sky-400 transition-colors"
                >
                  <Download size={15} class="text-slate-400" />
                  <span>Digital Downloads</span>
                </a>
                <a
                  href="/account/wishlist"
                  on:click={() => isAccountMenuOpen = false}
                  class="flex items-center gap-2.5 px-4 py-2 text-xs text-slate-300 hover:bg-slate-800 hover:text-rose-400 transition-colors"
                >
                  <Heart size={15} class="text-slate-400" />
                  <span>Wishlist</span>
                </a>
                <a
                  href="/account/addresses"
                  on:click={() => isAccountMenuOpen = false}
                  class="flex items-center gap-2.5 px-4 py-2 text-xs text-slate-300 hover:bg-slate-800 hover:text-emerald-400 transition-colors"
                >
                  <MapPin size={15} class="text-slate-400" />
                  <span>Addresses</span>
                </a>
                <a
                  href="/account/details"
                  on:click={() => isAccountMenuOpen = false}
                  class="flex items-center gap-2.5 px-4 py-2 text-xs text-slate-300 hover:bg-slate-800 hover:text-sky-400 transition-colors"
                >
                  <Settings size={15} class="text-slate-400" />
                  <span>Account details</span>
                </a>
                <div class="border-t border-slate-800/80 my-1"></div>
                <button
                  type="button"
                  on:click={handleLogout}
                  class="w-full text-left flex items-center gap-2.5 px-4 py-2 text-xs text-rose-400 hover:bg-rose-500/10 transition-colors font-semibold"
                >
                  <LogOut size={15} />
                  <span>Logout</span>
                </button>
              {/if}
            </div>
          {/if}
        </div>

        <!-- Cart Button Trigger -->
        <button
          id="cart-trigger-btn"
          on:click={() => isCartOpen.set(true)}
          class="relative p-2.5 sm:px-3.5 sm:py-2 rounded-xl bg-slate-900/80 hover:bg-slate-800 border border-slate-800 hover:border-orange-500/50 text-slate-200 hover:text-white transition-all shadow-md flex items-center gap-2 group"
          aria-label="View Shopping Cart"
        >
          <ShoppingBag size={18} class="group-hover:text-orange-400 transition-colors" />
          <span class="hidden sm:inline text-xs font-bold font-mono">
            {($cartSubtotal / 100).toFixed(2)} {currencySymbol}
          </span>
          {#if $cartCount > 0}
            <span class="absolute -top-1.5 -right-1.5 min-w-[20px] h-5 px-1 rounded-full bg-orange-600 text-white text-[11px] font-bold flex items-center justify-center shadow-lg shadow-orange-600/50 animate-pulse">
              {$cartCount}
            </span>
          {/if}
        </button>
      </div>
    </div>
  </header>

  <!-- Main Viewport -->
  <main class="flex-1">
    <slot />
  </main>

  <!-- Slide-Over Cart Sheet -->
  {#if $isCartOpen}
    <div class="fixed inset-0 z-50 overflow-hidden">
      <!-- Backdrop -->
      <button 
        type="button"
        class="absolute inset-0 bg-slate-950/80 backdrop-blur-sm transition-opacity w-full h-full border-0 p-0 m-0 cursor-default"
        on:click={() => isCartOpen.set(false)}
        aria-label="Close cart backdrop"
      ></button>

      <div class="fixed inset-y-0 right-0 max-w-full flex pl-10">
        <div class="w-screen max-w-md bg-slate-900 border-l border-slate-800 shadow-2xl flex flex-col">
          <!-- Cart Header -->
          <div class="px-6 py-5 border-b border-slate-800 flex items-center justify-between">
            <div class="flex items-center gap-2.5">
              <ShoppingBag size={20} class="text-orange-500" />
              <h2 class="text-lg font-bold text-white tracking-tight">Your Cart ({$cartCount})</h2>
            </div>
            <button
              on:click={() => isCartOpen.set(false)}
              class="p-2 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
              aria-label="Close cart"
            >
              <X size={18} />
            </button>
          </div>

          <!-- Cart Items List -->
          <div class="flex-1 overflow-y-auto px-6 py-4 divide-y divide-slate-800/80">
            {#if $cart.length === 0}
              <div class="h-full flex flex-col items-center justify-center text-center py-16 text-slate-400">
                <div class="w-16 h-16 rounded-full bg-slate-800/50 flex items-center justify-center mb-4 text-slate-500">
                  <ShoppingBag size={32} />
                </div>
                <p class="text-sm font-semibold text-slate-300">Your cart is currently empty</p>
                <p class="text-xs text-slate-500 mt-1 max-w-xs">Explore our mechanical keyboards, apparel, and software books.</p>
              </div>
            {:else}
              {#each $cart as item}
                <div class="py-4 flex gap-4">
                  {#if item.image_url}
                    <img src={item.image_url} alt={item.product_title} class="w-16 h-16 rounded-xl object-cover bg-slate-950 border border-slate-800 flex-shrink-0" />
                  {:else}
                    <div class="w-16 h-16 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center text-2xl flex-shrink-0">
                      📦
                    </div>
                  {/if}

                  <div class="flex-1 min-w-0">
                    <h3 class="text-xs font-bold text-white truncate">{item.product_title}</h3>
                    <div class="text-[11px] text-slate-400 truncate mt-0.5">{item.variant_title}</div>
                    <div class="text-[10px] font-mono text-orange-400/90 mt-0.5">SKU: {item.sku}</div>

                    <div class="mt-2 flex items-center justify-between">
                      <div class="flex items-center border border-slate-700 rounded-lg bg-slate-950 px-1 py-0.5">
                        <button
                          on:click={() => cart.updateQuantity(item.variant_id, item.quantity - 1)}
                          class="p-1 text-slate-400 hover:text-white transition-colors"
                          aria-label="Decrease quantity"
                        >
                          <Minus size={12} />
                        </button>
                        <span class="px-2 text-xs font-semibold font-mono">{item.quantity}</span>
                        <button
                          on:click={() => cart.updateQuantity(item.variant_id, item.quantity + 1)}
                          class="p-1 text-slate-400 hover:text-white transition-colors"
                          aria-label="Increase quantity"
                        >
                          <Plus size={12} />
                        </button>
                      </div>

                      <div class="text-right">
                        <span class="text-xs font-mono font-bold text-white">
                          {((item.price_cents * item.quantity) / 100).toFixed(2)} {currencySymbol}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              {/each}
            {/if}
          </div>

          <!-- Cart Footer / Checkout -->
          {#if $cart.length > 0}
            <div class="p-6 border-t border-slate-800 bg-slate-950/60 space-y-4">
              <div class="flex items-center justify-between text-sm">
                <span class="text-slate-400">Subtotal:</span>
                <span class="font-mono font-bold text-lg text-white">
                  {($cartSubtotal / 100).toFixed(2)} {currencySymbol}
                </span>
              </div>
              <p class="text-[11px] text-slate-500">Taxes and shipping calculated at checkout.</p>

              <a
                href="/checkout"
                on:click={() => isCartOpen.set(false)}
                class="w-full py-3.5 px-4 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-xs flex items-center justify-center gap-2 shadow-lg shadow-orange-600/30 transition-all hover:scale-[1.01]"
              >
                <span>Proceed to Secure Checkout</span>
                <ArrowRight size={16} />
              </a>

              <div class="flex items-center justify-center gap-2 text-xs text-slate-400">
                <ShieldCheck size={14} class="text-emerald-400" />
                <span>ACID Safe Inventory Lock &bull; SSL Secured</span>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- Footer with 3 Structured Sections & Full-Width Copyright Bar -->
  <footer class="border-t border-slate-900 bg-slate-950 mt-auto">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-14 pb-8">
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-10 items-start mb-12">
        <!-- SECTION 1 (Left): Store Branding (Logo Only vs Logo + Information) -->
        <div class="lg:col-span-4 space-y-4">
          <div class="flex items-center gap-3">
            {#if store.logo_url}
              <img src={store.logo_url} alt={store.store_name || 'Store Logo'} class="h-10 max-w-[210px] object-contain" />
            {:else}
              <span class="text-2xl select-none">🦀</span>
              <span class="font-black text-white text-base tracking-tight">{store.store_name || 'RustCraft Store'}</span>
            {/if}
          </div>

          {#if footerCfg.branding_mode !== 'logo_only'}
            <div class="space-y-2.5 text-xs text-slate-400 leading-relaxed">
              {#if store.store_subtitle}
                <p class="text-slate-300 font-medium">{store.store_subtitle}</p>
              {/if}
              {#if store.company_address}
                <div class="flex items-start gap-2 pt-1 text-slate-400">
                  <MapPin size={14} class="text-orange-400 flex-shrink-0 mt-0.5" />
                  <span>{store.company_address}</span>
                </div>
              {/if}
              {#if store.support_email}
                <div class="flex items-center gap-2 text-slate-400">
                  <Mail size={14} class="text-orange-400 flex-shrink-0" />
                  <a href="mailto:{store.support_email}" class="hover:text-white transition-colors">{store.support_email}</a>
                </div>
              {/if}
              {#if store.phone}
                <div class="flex items-center gap-2 text-slate-400">
                  <Phone size={14} class="text-orange-400 flex-shrink-0" />
                  <a href="tel:{store.phone}" class="hover:text-white transition-colors">{store.phone}</a>
                </div>
              {/if}
              {#if store.vat_id}
                <p class="text-[11px] text-slate-500 font-mono pt-1">VAT ID: {store.vat_id}</p>
              {/if}
            </div>
          {/if}
        </div>

        <!-- SECTION 2 (Middle): Footer Menu (Single Line vs Up to 3 Columns, Top-Aligned) -->
        <div class="lg:col-span-5">
          {#if footerCfg.menu_layout === 'single_line'}
            <!-- Single Line Layout -->
            <div class="space-y-3">
              <h4 class="text-xs font-bold text-white uppercase tracking-wider">Quick Navigation</h4>
              <nav class="flex flex-wrap gap-x-6 gap-y-2.5 text-xs text-slate-400">
                {#if footerCfg.columns && footerCfg.columns[0] && footerCfg.columns[0].links}
                  {#each footerCfg.columns[0].links as link}
                    <a href={link.url} class="hover:text-orange-400 transition-colors whitespace-nowrap">{link.label}</a>
                  {/each}
                {:else if footerMenu.length > 0}
                  {#each footerMenu as item}
                    <a href={item.url} class="hover:text-orange-400 transition-colors whitespace-nowrap">{item.label}</a>
                  {/each}
                {/if}
              </nav>
            </div>
          {:else}
            <!-- Up to 3 Columns Layout, Top Aligned -->
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-6 items-start">
              {#each (footerCfg.columns || []) as col}
                <div class="space-y-3">
                  <h4 class="text-xs font-bold text-white uppercase tracking-wider border-b border-slate-900 pb-2">
                    {col.title}
                  </h4>
                  <ul class="space-y-2 text-xs text-slate-400">
                    {#each (col.links || []) as link}
                      <li>
                        {#if link.url === '#cookie-preferences'}
                          <button
                            type="button"
                            on:click={() => {
                              if (typeof window !== 'undefined') {
                                window.dispatchEvent(new CustomEvent('open-cookie-preferences'));
                              }
                            }}
                            class="hover:text-orange-400 transition-colors text-left"
                          >
                            {link.label}
                          </button>
                        {:else}
                          <a href={link.url} class="hover:text-orange-400 transition-colors">{link.label}</a>
                        {/if}
                      </li>
                    {/each}
                  </ul>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- SECTION 3 (Right): Follow Us + Supported Payments Underneath -->
        <div class="lg:col-span-3 space-y-6">
          <!-- Follow Us Platforms -->
          {#if footerCfg.show_socials !== false && footerCfg.enabled_socials && footerCfg.enabled_socials.length > 0}
            <div class="space-y-3">
              <h4 class="text-xs font-bold text-white uppercase tracking-wider">Follow Us</h4>
              <p class="text-[11px] text-slate-400">Connect with our community across verified channels:</p>
              <div class="flex flex-wrap items-center gap-2">
                <!-- GitHub -->
                {#if footerCfg.enabled_socials.includes('github') && footerCfg.social_links?.github}
                  <a
                    href={footerCfg.social_links.github}
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="GitHub"
                    class="w-8 h-8 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-slate-700 text-slate-300 hover:text-white flex items-center justify-center transition-all hover:scale-105"
                  >
                    <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>
                  </a>
                {/if}

                <!-- Twitter / X -->
                {#if footerCfg.enabled_socials.includes('twitter') && footerCfg.social_links?.twitter}
                  <a
                    href={footerCfg.social_links.twitter}
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="Twitter / X"
                    class="w-8 h-8 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-slate-700 text-slate-300 hover:text-white flex items-center justify-center transition-all hover:scale-105"
                  >
                    <svg class="w-3.5 h-3.5 fill-current" viewBox="0 0 24 24"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>
                  </a>
                {/if}

                <!-- Instagram -->
                {#if footerCfg.enabled_socials.includes('instagram') && footerCfg.social_links?.instagram}
                  <a
                    href={footerCfg.social_links.instagram}
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="Instagram"
                    class="w-8 h-8 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-slate-700 text-slate-300 hover:text-pink-400 flex items-center justify-center transition-all hover:scale-105"
                  >
                    <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24"><path d="M12 2.163c3.204 0 3.584.012 4.85.07 3.252.148 4.771 1.691 4.919 4.919.058 1.265.069 1.645.069 4.849 0 3.205-.012 3.584-.069 4.849-.149 3.225-1.664 4.771-4.919 4.919-1.266.058-1.644.07-4.85.07-3.204 0-3.584-.012-4.849-.07-3.26-.149-4.771-1.699-4.919-4.92-.058-1.265-.07-1.644-.07-4.849 0-3.204.013-3.583.07-4.849.149-3.227 1.664-4.771 4.919-4.919 1.266-.057 1.645-.069 4.849-.069zm0-2.163c-3.259 0-3.667.014-4.947.072-4.358.2-6.78 2.618-6.98 6.98-.059 1.281-.073 1.689-.073 4.948 0 3.259.014 3.668.072 4.948.2 4.358 2.618 6.78 6.98 6.98 1.281.058 1.689.072 4.948.072 3.259 0 3.668-.014 4.948-.072 4.354-.2 6.782-2.618 6.979-6.98.059-1.28.073-1.689.073-4.948 0-3.259-.014-3.667-.072-4.947-.196-4.354-2.617-6.78-6.979-6.98-1.281-.059-1.69-.073-4.949-.073zm0 5.838c-3.403 0-6.162 2.759-6.162 6.162s2.759 6.163 6.162 6.163 6.162-2.759 6.162-6.163c0-3.403-2.759-6.162-6.162-6.162zm0 10.162c-2.209 0-4-1.79-4-4 0-2.209 1.791-4 4-4s4 1.791 4 4c0 2.21-1.791 4-4 4zm6.406-11.845c-.796 0-1.441.645-1.441 1.44s.645 1.44 1.441 1.44c.795 0 1.439-.645 1.439-1.44s-.644-1.44-1.439-1.44z"/></svg>
                  </a>
                {/if}

                <!-- YouTube -->
                {#if footerCfg.enabled_socials.includes('youtube') && footerCfg.social_links?.youtube}
                  <a
                    href={footerCfg.social_links.youtube}
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="YouTube"
                    class="w-8 h-8 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-slate-700 text-slate-300 hover:text-red-500 flex items-center justify-center transition-all hover:scale-105"
                  >
                    <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>
                  </a>
                {/if}

                <!-- Facebook -->
                {#if footerCfg.enabled_socials.includes('facebook') && footerCfg.social_links?.facebook}
                  <a
                    href={footerCfg.social_links.facebook}
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="Facebook"
                    class="w-8 h-8 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-slate-700 text-slate-300 hover:text-blue-500 flex items-center justify-center transition-all hover:scale-105"
                  >
                    <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24"><path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z"/></svg>
                  </a>
                {/if}

                <!-- Discord -->
                {#if footerCfg.enabled_socials.includes('discord') && footerCfg.social_links?.discord}
                  <a
                    href={footerCfg.social_links.discord}
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="Discord"
                    class="w-8 h-8 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-slate-700 text-slate-300 hover:text-indigo-400 flex items-center justify-center transition-all hover:scale-105"
                  >
                    <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24"><path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.929 1.793 8.18 1.793 12.061 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.894.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.028zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/></svg>
                  </a>
                {/if}

                <!-- WhatsApp -->
                {#if footerCfg.enabled_socials.includes('whatsapp') && footerCfg.social_links?.whatsapp}
                  <a
                    href={footerCfg.social_links.whatsapp}
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="WhatsApp"
                    class="w-8 h-8 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-slate-700 text-slate-300 hover:text-emerald-400 flex items-center justify-center transition-all hover:scale-105"
                  >
                    <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24"><path d="M12.031 0C5.396 0 .029 5.367.029 12.002c0 2.118.552 4.185 1.602 6.008L0 24l6.183-1.621a11.968 11.968 0 0 0 5.848 1.51h.005c6.634 0 12.001-5.367 12.001-12.002A11.939 11.939 0 0 0 12.031 0zm-.005 21.884h-.004a9.948 9.948 0 0 1-5.074-1.388l-.364-.216-3.771.989 1.006-3.676-.237-.377a9.927 9.927 0 0 1-1.52-5.204c0-5.498 4.474-9.972 9.974-9.972 2.664 0 5.168 1.038 7.051 2.923a9.922 9.922 0 0 1 2.918 7.053c0 5.498-4.474 9.972-9.974 9.972zm5.464-7.464c-.3-.15-1.774-.875-2.049-.975-.275-.1-.475-.15-.675.15-.2.3-.775.975-.95 1.175-.175.2-.35.225-.65.075-.3-.15-1.267-.467-2.414-1.49-1.002-.894-1.678-2-1.875-2.34-.197-.34-.021-.524.129-.674.135-.134.3-.35.45-.525.15-.175.2-.3.3-.5.1-.2.05-.375-.025-.525-.075-.15-.675-1.625-.925-2.225-.244-.585-.492-.505-.675-.515-.175-.01-.375-.01-.575-.01-.2 0-.525.075-.8.375s-1.05 1.025-1.05 2.5 1.075 2.898 1.225 3.1c.15.2 2.115 3.23 5.124 4.53 3.01 1.3 3.01.867 3.56.817.55-.05 1.774-.725 2.024-1.425.25-.7.25-1.3.175-1.425-.075-.125-.275-.2-.575-.35z"/></svg>
                  </a>
                {/if}
              </div>
            </div>
          {/if}

          <!-- Underneath: Supported Payments with Official Logos -->
          {#if footerCfg.show_payments !== false}
            <div class="space-y-3 pt-2">
              <h4 class="text-xs font-bold text-white uppercase tracking-wider">Accepted Payment Methods</h4>
              <div class="flex flex-wrap items-center gap-2">
                <!-- Stripe -->
                <span class="inline-flex items-center px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-300 shadow-sm" title="Stripe Payments">
                  <svg class="h-4 w-auto fill-indigo-400" viewBox="0 0 60 25"><path d="M59.64 14.28c0-4.49-3.21-7.8-7.55-7.8-4.41 0-7.39 3.32-7.39 7.78 0 5.28 3.56 7.74 7.91 7.74 2.12 0 3.72-.48 4.93-1.18v-3.08c-1.21.64-2.61.98-4.32.98-2.02 0-3.69-.84-4.14-2.82h10.51c.03-.5.05-1.09.05-1.62zm-10.59-1.25c.18-1.78 1.44-2.69 3.06-2.69 1.57 0 2.85.91 3.04 2.69h-6.1zm-8.89-6.22c-1.32-.47-2.73-.72-4.16-.72-4.38 0-7.07 2.27-7.07 6.13 0 5.99 8.24 5.04 8.24 7.64 0 .91-.8 1.4-1.92 1.4-1.67 0-3.83-.69-5.52-1.63v3.74c1.86.8 3.79 1.18 5.61 1.18 4.49 0 7.42-2.22 7.42-6.15 0-6.44-8.27-5.32-8.27-7.66 0-.81.69-1.29 1.76-1.29 1.44 0 3.31.52 4.71 1.34v-3.98h-.8zm-16.71 7.47c0-4.49-3.21-7.8-7.55-7.8-4.41 0-7.39 3.32-7.39 7.78 0 5.28 3.56 7.74 7.91 7.74 2.12 0 3.72-.48 4.93-1.18v-3.08c-1.21.64-2.61.98-4.32.98-2.02 0-3.69-.84-4.14-2.82h10.51c.03-.5.05-1.09.05-1.62zm-10.59-1.25c.18-1.78 1.44-2.69 3.06-2.69 1.57 0 2.85.91 3.04 2.69h-6.1zm-5.18-6.22h-4.38v15.22h4.38v-15.22zm-2.19-5.81c-1.46 0-2.61 1.09-2.61 2.49 0 1.37 1.15 2.46 2.61 2.46s2.61-1.09 2.61-2.46c0-1.4-1.15-2.49-2.61-2.49z"/></svg>
                </span>

                <!-- PayPal -->
                <span class="inline-flex items-center px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-300 shadow-sm" title="PayPal">
                  <svg class="h-4 w-auto" viewBox="0 0 80 25"><path fill="#003087" d="M12.63 2.15H4.88c-.52 0-.96.38-1.04.89L.86 22.04c-.06.39.24.74.63.74h3.76c.46 0 .85-.33.92-.78l.84-5.32c.08-.51.52-.89 1.04-.89h2.38c4.95 0 7.82-2.4 8.57-7.14.33-2.09.02-3.73-.93-4.88-1.06-1.29-2.95-1.62-5.44-1.62z"/><path fill="#0079C1" d="M14.61 7.87c-.75 4.74-3.62 7.14-8.57 7.14h-2.38c-.52 0-.96.38-1.04.89l-.97 6.13c-.06.39.24.74.63.74h3.5c.46 0 .85-.33.92-.78l.74-4.69c.08-.51.52-.89 1.04-.89h1.65c4.32 0 7.7-1.75 8.68-6.83.41-2.11.2-3.89-.78-5.07-.63-.76-1.59-1.25-2.88-1.44.41.97.47 2.1.26 3.42z"/><text x="24" y="17" fill="#ffffff" font-family="sans-serif" font-weight="bold" font-size="14">PayPal</text></svg>
                </span>

                <!-- Apple Pay -->
                <span class="inline-flex items-center px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white shadow-sm font-semibold text-xs" title="Apple Pay">
                  <svg class="h-4 w-auto fill-current mr-1" viewBox="0 0 170 170"><path d="M150.37 130.25c-2.45 5.66-5.35 10.87-8.71 15.66-4.58 6.53-8.33 11.05-11.22 13.56-4.48 4.12-9.28 6.23-14.42 6.35-3.69 0-8.14-1.05-13.32-3.18-5.19-2.12-9.97-3.17-14.34-3.17-4.58 0-9.49 1.05-14.75 3.17-5.26 2.13-9.5 3.24-12.74 3.35-4.35.13-9.16-1.9-14.42-6.08-3.69-3.04-7.67-7.81-11.96-14.34-5.87-8.91-10.46-19.11-13.77-30.61-3.31-11.5-4.97-22.38-4.97-32.65 0-14.35 3.69-26.06 11.07-35.13 7.39-9.08 16.64-13.73 27.76-13.97 4.8 0 10.12 1.25 15.98 3.76 5.86 2.5 9.77 3.82 11.71 3.94 1.52 0 5.66-1.42 12.44-4.25 6.78-2.83 12.39-4.04 16.85-3.64 12.87.97 22.84 5.76 29.9 14.37-11.45 6.9-17.07 16.38-16.85 28.45.22 9.5 3.91 17.5 11.07 24 7.17 6.5 15.66 10.15 25.48 10.95-2.07 6.09-4.47 12.3-7.2 18.63zM119.22 33.15c0-7.39 2.61-14.46 7.83-21.21 5.22-6.74 11.74-11.19 19.57-13.34.87 7.07-.98 13.97-5.55 20.7-4.57 6.73-10.76 11.23-18.57 13.5-.76-.78-1.28-1.57-1.57-2.36-.44-1.3-.71-2.73-.71-4.29z"/></svg>
                  <span>Pay</span>
                </span>

                <!-- Google Pay -->
                <span class="inline-flex items-center px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-200 shadow-sm font-semibold text-xs" title="Google Pay">
                  <svg class="h-4 w-auto mr-1" viewBox="0 0 24 24"><path fill="#4285F4" d="M23.745 12.27c0-.7-.06-1.4-.19-2.07H12v4.51h6.6c-.29 1.52-1.14 2.82-2.4 3.68v3.05h3.88c2.27-2.09 3.665-5.17 3.665-9.17z"/><path fill="#34A853" d="M12 24c3.24 0 5.95-1.08 7.93-2.91l-3.88-3.05c-1.08.72-2.45 1.16-4.05 1.16-3.12 0-5.77-2.1-6.72-4.93H1.25v3.15C3.26 21.36 7.33 24 12 24z"/><path fill="#FBBC05" d="M5.28 14.27c-.25-.72-.38-1.49-.38-2.27s.13-1.55.38-2.27V6.58H1.25C.45 8.18 0 10.03 0 12s.45 3.82 1.25 5.42l4.03-3.15z"/><path fill="#EA4335" d="M12 4.75c1.77 0 3.35.61 4.6 1.8l3.42-3.42C17.95 1.19 15.24 0 12 0 7.33 0 3.26 2.64 1.25 6.58l4.03 3.15c.95-2.83 3.6-4.98 6.72-4.98z"/></svg>
                  <span>Pay</span>
                </span>

                <!-- Amazon Pay -->
                <span class="inline-flex items-center px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-200 shadow-sm font-semibold text-xs" title="Amazon Pay">
                  <span class="text-amber-400 font-black mr-0.5">a</span><span>pay</span>
                </span>
              </div>
            </div>
          {/if}
        </div>
      </div>

      {#if store.newsletter_enabled}
        <div class="max-w-xl mb-8">
          <NewsletterSignup />
        </div>
      {/if}

      <!-- FULL WIDTH BOTTOM COPYRIGHT BAR -->
      <div class="pt-8 border-t border-slate-900 flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-slate-500">
        <div>
          {#if footerCfg.copyright_format === 'custom' && footerCfg.custom_copyright}
            {footerCfg.custom_copyright}
          {:else}
            Copyright &copy; 2026 {store.store_name || 'RustCraft Store'}. All rights reserved.
          {/if}
        </div>
        <div class="flex flex-wrap items-center gap-4 text-[11px] text-slate-400">
          <!-- Withdrawal function: always visible (Art. 11a Consumer Rights Directive) -->
          <a href="/withdraw" class="font-bold text-orange-400 hover:text-orange-300 underline underline-offset-2">Withdraw from contract here</a>
          <span>&bull;</span>
          <a href="/policies/privacy-policy" class="hover:text-white transition-colors">Privacy Policy</a>
          <span>&bull;</span>
          <a href="/policies/terms-conditions" class="hover:text-white transition-colors">Terms & Conditions</a>
          <span>&bull;</span>
          <a href="/policies/legal-notice" class="hover:text-white transition-colors">Legal Notice (Impressum)</a>
          <span>&bull;</span>
          <button
            type="button"
            on:click={() => {
              if (typeof window !== 'undefined') {
                window.dispatchEvent(new CustomEvent('open-cookie-preferences'));
              }
            }}
            class="hover:text-white transition-colors"
          >
            Cookie Preferences
          </button>
        </div>
      </div>
    </div>
  </footer>

  <!-- Cookie Consent Banner & Preferences Modal -->
  <CookieBanner {store} />
</div>
