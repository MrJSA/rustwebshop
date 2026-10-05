<script>
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { 
    LayoutDashboard, Package, Warehouse, ShoppingCart, 
    CreditCard, Truck, Sliders, ExternalLink, ShieldCheck, Zap,
    FileText, Menu, FolderTree, Image, Mail, AlertTriangle, KeyRound,
    LogOut, User, X, CheckCircle2, TrendingUp, ChevronDown, ChevronRight
  } from 'lucide-svelte';

  export let data;
  $: settings = data.settings || {};

  const navGroups = [
    {
      id: 'overview',
      label: 'Overview & Analytics',
      icon: LayoutDashboard,
      href: '/',
      match: (p) => p === '/' || p === '/analytics',
      subItems: [
        { href: '/', label: 'Executive Overview', tab: 'overview', match: (p, t) => (p === '/' && (!t || t === 'overview')) },
        { href: '/?tab=analytics', label: 'Purchase Analysis', tab: 'analytics', match: (p, t) => p === '/analytics' || (p === '/' && t === 'analytics') }
      ]
    },
    {
      id: 'products',
      label: 'Products',
      icon: Package,
      href: '/products',
      match: (p) => p.startsWith('/products') || p.startsWith('/categories') || p.startsWith('/logistics'),
      subItems: [
        { href: '/products?tab=catalog', label: 'Products & BOM', tab: 'catalog', match: (p, t) => (p === '/products' && (!t || t === 'catalog')) },
        { href: '/products?tab=categories', label: 'Categories Tree', tab: 'categories', match: (p, t) => p === '/categories' || (p === '/products' && t === 'categories') },
        { href: '/products?tab=stock', label: 'Logistics & Stock', tab: 'stock', match: (p, t) => p === '/logistics' || (p === '/products' && t === 'stock') },
        { href: '/products?tab=coupons', label: 'Promo & Discount Codes', tab: 'coupons', match: (p, t) => (p === '/products' && t === 'coupons') }
      ]
    },
    {
      id: 'orders',
      label: 'Orders & Slips',
      icon: ShoppingCart,
      href: '/orders',
      match: (p) => p.startsWith('/orders'),
      subItems: []
    },
    {
      id: 'storefront',
      label: 'Storefront & Design',
      icon: Sliders,
      href: '/settings/system',
      match: (p) => p === '/settings/system' || p === '/settings/menu' || p === '/settings/pages',
      subItems: [
        { href: '/settings/system?tab=hero', label: 'Hero & Carousels', tab: 'hero', match: (p, t) => (p === '/settings/system' && (!t || t === 'hero')) },
        { href: '/settings/system?tab=menu', label: 'Navigation Menus', tab: 'menu', match: (p, t) => p === '/settings/menu' || (p === '/settings/system' && t === 'menu') },
        { href: '/settings/system?tab=policies', label: 'Policy CMS (Markdown)', tab: 'policies', match: (p, t) => p === '/settings/pages' || (p === '/settings/system' && t === 'policies') },
        { href: '/settings/system?tab=cookie', label: 'Cookie Consent', tab: 'cookie', match: (p, t) => (p === '/settings/system' && t === 'cookie') }
      ]
    },
    {
      id: 'settings',
      label: 'Settings',
      icon: ShieldCheck,
      href: '/settings',
      match: (p) => p === '/settings' || (p.startsWith('/settings') && !p.startsWith('/settings/system')),
      subItems: [
        { href: '/settings?tab=identity', label: 'Store Identity & Legal', tab: 'identity', match: (p, t) => (p === '/settings' && (!t || t === 'identity')) },
        { href: '/settings?tab=payments', label: 'Payment Providers', tab: 'payments', match: (p, t) => p === '/settings/payments' || (p === '/settings' && t === 'payments') },
        { href: '/settings?tab=email', label: 'Email & Auth Policies', tab: 'email', match: (p, t) => p === '/settings/email' || (p === '/settings' && t === 'email') },
        { href: '/settings?tab=shipping', label: 'Shipping & Delivery', tab: 'shipping', match: (p, t) => p === '/settings/shipping' || (p === '/settings' && t === 'shipping') },
        { href: '/settings?tab=media', label: 'Media Library', tab: 'media', match: (p, t) => p === '/settings/media' || (p === '/settings' && t === 'media') },
        { href: '/settings?tab=users', label: 'Admin Users & Access', tab: 'users', match: (p, t) => (p === '/settings' && t === 'users') },
        { href: '/settings?tab=export', label: 'Export & Backups', tab: 'export', match: (p, t) => (p === '/settings' && t === 'export') },
        { href: '/settings?tab=system', label: 'System & Updates', tab: 'system', match: (p, t) => (p === '/settings' && t === 'system') }
      ]
    }
  ];

  let adminUsername = 'admin';
  let isDefaultCredentials = false;
  let isChangePasswordModalOpen = false;
  let currentPassword = '';
  let newUsername = 'admin';
  let newPassword = '';
  let confirmPassword = '';
  let changePasswordError = '';
  let changePasswordSuccess = '';
  let isSubmittingChange = false;

  // Collapsible Submenu State
  let expandedGroups = {};
  $: {
    for (const group of navGroups) {
      if (group.match($page.url.pathname)) {
        if (expandedGroups[group.id] === undefined) {
          expandedGroups[group.id] = true;
        }
      }
    }
  }

  function toggleGroup(groupId) {
    expandedGroups[groupId] = !expandedGroups[groupId];
    expandedGroups = { ...expandedGroups };
  }

  $: isLoginPage = $page.url.pathname === '/login';

  // Only sections this admin may use are shown (enforced again server-side)
  $: visibleNavGroups = navGroups.filter((g) => !data.admin?.permissions || data.admin.permissions[g.id] !== false);

  // The session is validated server-side (hooks.server.js); the token itself lives in an httpOnly cookie.
  $: if (data.admin) {
    isDefaultCredentials = Boolean(data.admin.is_default);
    if (data.admin.username) adminUsername = data.admin.username;
  }
  $: if (isDefaultCredentials && !isLoginPage) isChangePasswordModalOpen = true;

  onMount(() => {
    // Remove tokens stored by older versions of the admin app
    ['admin_token', 'admin_username', 'admin_is_default'].forEach((k) => localStorage.removeItem(k));
    document.cookie = 'admin_token=; path=/; max-age=0;';
    newUsername = adminUsername;
  });

  async function handleLogout() {
    await fetch('/api/v1/admin/auth/logout', { method: 'POST' }).catch(() => {});
    window.location.href = '/login';
  }

  async function handleChangeCredentials() {
    changePasswordError = '';
    changePasswordSuccess = '';

    if (!currentPassword) {
      changePasswordError = 'Please enter your current password.';
      return;
    }

    if (!newPassword || newPassword.length < 12) {
      changePasswordError = 'New password must be at least 12 characters long.';
      return;
    }

    if (newPassword !== confirmPassword) {
      changePasswordError = 'New password and confirmation do not match.';
      return;
    }

    isSubmittingChange = true;

    try {
      // The proxy stores the renewed session token in the httpOnly cookie
      const res = await fetch('/api/v1/admin/auth/change-credentials', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          current_password: currentPassword,
          new_username: newUsername,
          new_password: newPassword
        })
      });

      const resData = await res.json().catch(() => ({}));
      if (!res.ok) {
        changePasswordError = resData.error || resData.message || 'Failed to change credentials.';
        isSubmittingChange = false;
        return;
      }

      const wasDefault = isDefaultCredentials;
      adminUsername = newUsername;
      isDefaultCredentials = false;
      changePasswordSuccess = 'Credentials updated successfully!';
      setTimeout(() => {
        isChangePasswordModalOpen = false;
        changePasswordSuccess = '';
        currentPassword = '';
        newPassword = '';
        confirmPassword = '';
        // Pages were locked until the initial password was replaced: reload them with full access
        if (wasDefault) window.location.reload();
      }, 1500);
    } catch (e) {
      changePasswordError = 'Error updating credentials.';
    } finally {
      isSubmittingChange = false;
    }
  }
</script>

{#if isLoginPage}
  <slot />
{:else}
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

      <!-- Navigation links with Sub-menus -->
      <nav class="flex-1 px-3 py-6 space-y-2 overflow-y-auto">
        {#each visibleNavGroups as group}
          {@const isGroupActive = group.match($page.url.pathname)}
          {@const isExpanded = expandedGroups[group.id] ?? isGroupActive}
          <div class="space-y-0.5">
            <div class="flex items-center gap-1">
              <a
                href={group.href}
                class="flex-1 flex items-center justify-between px-3 py-2 rounded-xl text-xs font-bold transition-all {isGroupActive ? 'bg-orange-600/15 text-orange-400 border border-orange-500/20' : 'text-slate-300 hover:text-white hover:bg-slate-800/80'}"
              >
                <div class="flex items-center gap-2.5">
                  <svelte:component this={group.icon} size={16} class={isGroupActive ? 'text-orange-400' : 'text-slate-400'} />
                  <span>{group.label}</span>
                </div>
                {#if group.subItems.length > 0}
                  <span class="text-[10px] text-slate-500 font-mono font-normal">{group.subItems.length}</span>
                {/if}
              </a>

              {#if group.subItems.length > 0}
                <button
                  type="button"
                  on:click|stopPropagation={() => toggleGroup(group.id)}
                  class="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800/80 transition-colors"
                  title={isExpanded ? 'Collapse submenu' : 'Expand submenu'}
                  aria-label={isExpanded ? 'Collapse submenu' : 'Expand submenu'}
                >
                  <ChevronDown
                    size={14}
                    class="transition-transform duration-200 {isExpanded ? 'rotate-180 text-orange-400' : 'text-slate-500'}"
                  />
                </button>
              {/if}
            </div>

            <!-- Submenu Items (Collapsible) -->
            {#if group.subItems.length > 0 && isExpanded}
              <div class="pl-7 pr-1 py-1 space-y-0.5 border-l border-slate-800/80 ml-5 my-0.5 animate-in fade-in slide-in-from-top-1 duration-150">
                {#each group.subItems as sub}
                  {@const isSubActive = sub.match($page.url.pathname, $page.url.searchParams.get('tab'))}
                  <a
                    href={sub.href}
                    class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-[11px] font-semibold transition-all {isSubActive ? 'bg-orange-600 text-white font-bold shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'}"
                  >
                    <span class="w-1.5 h-1.5 rounded-full {isSubActive ? 'bg-white' : 'bg-slate-600'}"></span>
                    <span>{sub.label}</span>
                  </a>
                {/each}
              </div>
            {/if}
          </div>
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
      <!-- Top Persistent Security Warning Banner (if default credentials are in use) -->
      {#if isDefaultCredentials}
        <div class="bg-gradient-to-r from-amber-600 via-orange-600 to-red-600 px-6 py-2 text-white flex items-center justify-between text-xs font-bold shadow-md z-40">
          <div class="flex items-center gap-2">
            <AlertTriangle size={16} class="text-amber-200 animate-pulse" />
            <span>This account still uses its initial password. Choose a new password to unlock the admin area.</span>
          </div>
          <button
            on:click={() => isChangePasswordModalOpen = true}
            class="px-3 py-1 rounded-lg bg-white text-slate-950 font-bold text-[11px] hover:bg-slate-100 transition-colors shadow-sm"
          >
            Change Credentials Now
          </button>
        </div>
      {/if}

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

          <!-- User Badge & Actions -->
          <div class="flex items-center gap-2">
            <button
              on:click={() => isChangePasswordModalOpen = true}
              class="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold border border-slate-700 transition-colors"
              title="Change Admin Password"
            >
              <User size={14} class="text-orange-400" />
              <span>{adminUsername}</span>
              <KeyRound size={12} class="text-slate-400 ml-1" />
            </button>

            <button
              on:click={handleLogout}
              class="p-2 rounded-xl bg-slate-800 hover:bg-rose-950/40 hover:text-rose-400 text-slate-400 border border-slate-700 transition-colors"
              title="Logout"
            >
              <LogOut size={16} />
            </button>
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

  <!-- Change Credentials Modal -->
  {#if isChangePasswordModalOpen}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-150">
      <div class="w-full max-w-md rounded-3xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4">
        <div class="flex items-center justify-between pb-3 border-b border-slate-800">
          <div class="flex items-center gap-2 text-white font-bold text-sm">
            <KeyRound size={18} class="text-orange-500" />
            <span>Change Admin Credentials</span>
          </div>
          <button
            on:click={() => isChangePasswordModalOpen = false}
            class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
          >
            <X size={16} />
          </button>
        </div>

        {#if changePasswordSuccess}
          <div class="p-3 rounded-xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-xs font-semibold flex items-center gap-2">
            <CheckCircle2 size={16} />
            <span>{changePasswordSuccess}</span>
          </div>
        {/if}

        {#if changePasswordError}
          <div class="p-3 rounded-xl bg-rose-500/20 text-rose-400 border border-rose-500/30 text-xs font-semibold">
            {changePasswordError}
          </div>
        {/if}

        <form on:submit|preventDefault={handleChangeCredentials} class="space-y-3.5 text-xs">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Current Password</label>
            <input
              type="password"
              bind:value={currentPassword}
              placeholder="Your current password"
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500 font-mono"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Admin Username</label>
            <input
              type="text"
              bind:value={newUsername}
              required
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">New Password</label>
            <input
              type="password"
              bind:value={newPassword}
              required
              placeholder="Minimum 8 characters"
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 font-mono"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Confirm New Password</label>
            <input
              type="password"
              bind:value={confirmPassword}
              required
              placeholder="Re-enter new password"
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 font-mono"
            />
          </div>

          <div class="pt-2 flex justify-end gap-2">
            <button
              type="button"
              on:click={() => isChangePasswordModalOpen = false}
              class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-semibold"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmittingChange}
              class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold transition-all shadow-md shadow-orange-600/30 disabled:opacity-50"
            >
              {isSubmittingChange ? 'Saving...' : 'Update Credentials'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
{/if}
