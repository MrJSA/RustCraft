<script>
  // Tab bar shown at the top of every page of a section; tabs may be query tabs or separate pages.
  import {
    Package, FolderTree, Warehouse, Tag, MessageSquare,
    ShoppingCart, Undo2,
    Layout, Menu, FileText, Cookie, Mail,
    Building, CreditCard, Truck, Image, UserCheck, Download, Server, Sparkles
  } from 'lucide-svelte';

  /** @type {'products' | 'orders' | 'storefront' | 'settings'} */
  export let section;
  export let active;
  /** Optional badge numbers by tab id */
  export let counts = {};

  const sections = {
    products: [
      { id: 'catalog', href: '/products?tab=catalog', label: 'Products & BOM', icon: Package },
      { id: 'categories', href: '/products?tab=categories', label: 'Categories Tree', icon: FolderTree },
      { id: 'stock', href: '/products?tab=stock', label: 'Logistics & Stock', icon: Warehouse },
      { id: 'coupons', href: '/products?tab=coupons', label: 'Promo & Discount Codes', icon: Tag },
      { id: 'reviews', href: '/products/reviews', label: 'Product Reviews', icon: MessageSquare }
    ],
    orders: [
      { id: 'orders', href: '/orders', label: 'All Orders', icon: ShoppingCart },
      { id: 'withdrawals', href: '/orders/withdrawals', label: 'Withdrawals', icon: Undo2 }
    ],
    storefront: [
      { id: 'hero', href: '/settings/system?tab=hero', label: 'Hero & Carousels', icon: Layout },
      { id: 'menu', href: '/settings/system?tab=menu', label: 'Navigation Menus', icon: Menu },
      { id: 'policies', href: '/settings/system?tab=policies', label: 'Policy CMS (Markdown)', icon: FileText },
      { id: 'cookie', href: '/settings/system?tab=cookie', label: 'Cookie Consent', icon: Cookie },
      { id: 'newsletter', href: '/settings/system/newsletter', label: 'Newsletter', icon: Mail }
    ],
    settings: [
      { id: 'identity', href: '/settings?tab=identity', label: 'Store Identity & Legal', icon: Building },
      { id: 'payments', href: '/settings?tab=payments', label: 'Payment Providers', icon: CreditCard },
      { id: 'email', href: '/settings?tab=email', label: 'Email & Auth Policies', icon: Mail },
      { id: 'shipping', href: '/settings?tab=shipping', label: 'Shipping & Delivery', icon: Truck },
      { id: 'media', href: '/settings?tab=media', label: 'Media Library', icon: Image },
      { id: 'users', href: '/settings?tab=users', label: 'Admin Users & Access', icon: UserCheck },
      { id: 'export', href: '/settings?tab=export', label: 'Export & Backups', icon: Download },
      { id: 'system', href: '/settings?tab=system', label: 'System & Updates', icon: Server },
      { id: 'features', href: '/settings?tab=features', label: 'Shop Features & Legal', icon: Sparkles }
    ]
  };
</script>

<nav class="flex items-center gap-2 border-b border-slate-800 pb-3 overflow-x-auto">
  {#each sections[section] as tab (tab.id)}
    <a
      href={tab.href}
      data-sveltekit-replacestate
      data-sveltekit-noscroll
      data-sveltekit-keepfocus
      aria-current={active === tab.id ? 'page' : undefined}
      class="h-10 shrink-0 whitespace-nowrap px-4 rounded-xl text-xs font-bold transition-all flex items-center gap-2 border {active === tab.id ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 border-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border-slate-800'}"
    >
      <svelte:component this={tab.icon} size={15} />
      <span>{tab.label}</span>
      {#if counts[tab.id] != null}
        <span class="text-[10px] leading-none px-2 py-1 rounded-full {active === tab.id ? 'bg-white/20 text-white' : 'bg-slate-800 text-slate-400'}">{counts[tab.id]}</span>
      {/if}
    </a>
  {/each}
</nav>
