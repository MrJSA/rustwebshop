<script>
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { 
    LayoutDashboard, Package, Warehouse, ShoppingCart, 
    CreditCard, Truck, Sliders, ExternalLink, ShieldCheck, Zap,
    FileText, Menu, FolderTree, Image, Mail, AlertTriangle, KeyRound,
    LogOut, User, X, CheckCircle2
  } from 'lucide-svelte';

  export let data;
  $: settings = data.settings || {};

  const navLinks = [
    { href: '/', label: 'Overview & Analytics', icon: LayoutDashboard },
    { href: '/products', label: 'Products & BOM', icon: Package },
    { href: '/categories', label: 'Categories Tree', icon: FolderTree },
    { href: '/logistics', label: 'Logistics & Stock', icon: Warehouse },
    { href: '/orders', label: 'Orders & Slips', icon: ShoppingCart },
    { href: '/settings/media', label: 'Media Library', icon: Image },
    { href: '/settings/email', label: 'Email & Auth Policies', icon: Mail },
    { href: '/settings/shipping', label: 'Shipping Providers & Zones', icon: Truck },
    { href: '/settings/pages', label: 'Policy CMS (Markdown)', icon: FileText },
    { href: '/settings/menu', label: 'Navigation Menu', icon: Menu },
    { href: '/settings/payments', label: 'Payment Providers', icon: CreditCard },
    { href: '/settings/system', label: 'System & Shop Identity', icon: Sliders }
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

  $: isLoginPage = $page.url.pathname === '/login';

  onMount(async () => {
    if (isLoginPage) return;

    const token = localStorage.getItem('admin_token');
    if (!token) {
      goto('/login');
      return;
    }

    adminUsername = localStorage.getItem('admin_username') || 'admin';
    newUsername = adminUsername;

    // Verify token with backend & get current default credential status
    try {
      const res = await fetch('/api/v1/admin/auth/status', {
        headers: { Authorization: `Bearer ${token}` }
      });
      if (res.ok) {
        const statusData = await res.json();
        isDefaultCredentials = Boolean(statusData.is_default);
        adminUsername = statusData.username || adminUsername;
      } else {
        localStorage.removeItem('admin_token');
        goto('/login');
      }
    } catch (e) {
      console.error('Failed to verify admin status:', e);
    }
  });

  function handleLogout() {
    localStorage.removeItem('admin_token');
    localStorage.removeItem('admin_username');
    localStorage.removeItem('admin_is_default');
    document.cookie = 'admin_token=; path=/; max-age=0;';
    goto('/login');
  }

  async function handleChangeCredentials() {
    changePasswordError = '';
    changePasswordSuccess = '';

    if (!newPassword || newPassword.length < 8) {
      changePasswordError = 'New password must be at least 8 characters long.';
      return;
    }

    if (newPassword !== confirmPassword) {
      changePasswordError = 'New password and confirmation do not match.';
      return;
    }

    const token = localStorage.getItem('admin_token');
    isSubmittingChange = true;

    try {
      const res = await fetch('/api/v1/admin/auth/change-credentials', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`
        },
        body: JSON.stringify({
          current_password: currentPassword || 'RustCraftAdmin2026!',
          new_username: newUsername,
          new_password: newPassword
        })
      });

      const resData = await res.json();
      if (!res.ok) {
        changePasswordError = resData.error || resData.message || 'Failed to change credentials.';
        isSubmittingChange = false;
        return;
      }

      if (resData.token) {
        localStorage.setItem('admin_token', resData.token);
        document.cookie = `admin_token=${resData.token}; path=/; max-age=604800; SameSite=Lax`;
      }
      localStorage.setItem('admin_username', newUsername);
      localStorage.setItem('admin_is_default', 'false');

      adminUsername = newUsername;
      isDefaultCredentials = false;
      changePasswordSuccess = 'Credentials updated successfully!';
      setTimeout(() => {
        isChangePasswordModalOpen = false;
        changePasswordSuccess = '';
        currentPassword = '';
        newPassword = '';
        confirmPassword = '';
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
      <!-- Top Persistent Security Warning Banner (if default credentials are in use) -->
      {#if isDefaultCredentials}
        <div class="bg-gradient-to-r from-amber-600 via-orange-600 to-red-600 px-6 py-2 text-white flex items-center justify-between text-xs font-bold shadow-md z-40">
          <div class="flex items-center gap-2">
            <AlertTriangle size={16} class="text-amber-200 animate-pulse" />
            <span>Default admin credentials (<code class="bg-black/30 px-1 py-0.5 rounded font-mono">admin / RustCraftAdmin2026!</code>) are currently active! Please update your credentials for safety.</span>
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
              placeholder="Defaults to RustCraftAdmin2026!"
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
