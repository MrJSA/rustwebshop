<script>
  import '../app.css';
  import { page } from '$app/stores';
  import { 
    LayoutDashboard, Package, Warehouse, ShoppingCart, 
    CreditCard, Truck, Sliders, ExternalLink, ShieldCheck, Zap,
    FileText, Menu, FolderTree
  } from 'lucide-svelte';

  export let data;
  $: settings = data.settings || {};

  const navLinks = [
    { href: '/', label: 'Overview & Analytics', icon: LayoutDashboard },
    { href: '/products', label: 'Products & BOM', icon: Package },
    { href: '/categories', label: 'Categories Tree', icon: FolderTree },
    { href: '/logistics', label: 'Logistics & Stock', icon: Warehouse },
    { href: '/orders', label: 'Orders & Slips', icon: ShoppingCart },
    { href: '/settings/shipping', label: 'Shipping Providers & Zones', icon: Truck },
    { href: '/settings/pages', label: 'Policy CMS (Markdown)', icon: FileText },
    { href: '/settings/menu', label: 'Navigation Menu', icon: Menu },
    { href: '/settings/payments', label: 'Payment Providers', icon: CreditCard },
    { href: '/settings/system', label: 'System & Shop Identity', icon: Sliders }
  ];

</script>

<div class="min-h-screen flex bg-slate-950 text-slate-100 selection:bg-orange-500 selection:text-white">
  <!-- Sidebar -->
  <aside class="w-64 border-r border-slate-800 bg-slate-900/70 flex flex-col flex-shrink-0">
    <!-- Brand -->
    <div class="h-20 px-6 border-b border-slate-800 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-orange-600 flex items-center justify-center text-xl shadow-lg shadow-orange-600/30">
          🦀
        </div>
        <div>
          <span class="font-extrabold text-white text-base tracking-tight">RustCraft</span>
          <div class="text-[10px] text-orange-400 font-mono uppercase tracking-wider font-semibold">
            Admin Console (Port 4000)
          </div>
        </div>
      </div>
    </div>

    <!-- Navigation links -->
    <nav class="flex-1 px-3 py-6 space-y-1 overflow-y-auto">
      {#each navLinks as item}
        {@const active = $page.url.pathname === item.href}
        <a
          href={item.href}
          class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-bold transition-all {active ? 'bg-orange-600 text-white shadow-md shadow-orange-600/30' : 'text-slate-400 hover:text-white hover:bg-slate-800/80'}"
        >
          <svelte:component this={item.icon} size={18} class={active ? 'text-white' : 'text-slate-400'} />
          <span>{item.label}</span>
        </a>
      {/each}
    </nav>

    <!-- Bottom Environment Status -->
    <div class="p-4 border-t border-slate-800 bg-slate-900/90 space-y-3">
      <div class="p-3 rounded-xl bg-slate-950 border border-slate-800 text-[11px] space-y-1">
        <div class="flex items-center justify-between text-slate-400">
          <span>Mode:</span>
          <span class="capitalize font-mono text-orange-400 font-bold">{settings.deployment_mode || 'Development'}</span>
        </div>
        <div class="flex items-center justify-between text-slate-400">
          <span>Debug Engine:</span>
          <span class="font-mono text-emerald-400 font-bold">{settings.debug_mode ? 'ON' : 'OFF'}</span>
        </div>
      </div>

      <a
        href="http://localhost:8080"
        target="_blank"
        class="w-full py-2 px-3 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-bold flex items-center justify-center gap-2 transition-colors border border-slate-700"
      >
        <span>Open Storefront (8080)</span>
        <ExternalLink size={13} class="text-orange-400" />
      </a>
    </div>
  </aside>

  <!-- Main View Area -->
  <div class="flex-1 flex flex-col min-w-0">
    <!-- Topbar -->
    <header class="h-20 border-b border-slate-800 bg-slate-900/40 px-8 flex items-center justify-between backdrop-blur-md sticky top-0 z-30">
      <div>
        <h2 class="text-lg font-bold text-white tracking-tight">
          {settings.store_name || 'RustCraft E-Commerce Store'}
        </h2>
        <div class="text-xs text-slate-400">
          WooCommerce-Class Admin Management Back-Office
        </div>
      </div>

      <div class="flex items-center gap-4">
        <div class="hidden sm:flex items-center gap-2 px-3 py-1 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold">
          <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
          <span>PostgreSQL Active</span>
        </div>

        <a
          href="http://localhost:8080"
          target="_blank"
          class="px-4 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md flex items-center gap-1.5"
        >
          <span>Live Store</span>
          <ExternalLink size={13} />
        </a>
      </div>
    </header>

    <!-- Page Content -->
    <main class="flex-1 p-8 overflow-y-auto">
      <slot />
    </main>
  </div>
</div>
