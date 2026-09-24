<script>
  import '../app.css';
  import { cart, cartCount, cartSubtotal, isCartOpen } from '$lib/stores/cart.js';
  import { customer } from '$lib/stores/customer.js';
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
    Heart,
    MapPin,
    Settings,
    LogOut,
    KeyRound,
    LogIn
  } from 'lucide-svelte';

  export let data;
  $: store = data.store || {};
  $: currencySymbol = store.currency_symbol || '€';
  $: headerMenu = data.headerMenu || data.menuItems || [];
  $: footerMenu = data.footerMenu || [];

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

<svelte:window on:click={(e) => {
  if (!e.target.closest('#account-dropdown-container')) {
    isAccountMenuOpen = false;
  }
}} />

<div class="min-h-screen flex flex-col bg-slate-950 text-slate-100 selection:bg-orange-500 selection:text-white">
  <!-- Top Announcement / Debug Bar (No admin button) -->
  {#if store.debug_mode}
    <div class="bg-gradient-to-r from-orange-600 to-amber-600 px-4 py-1 text-center text-xs font-semibold tracking-wide text-white flex items-center justify-center gap-2 shadow-sm">
      <span class="bg-white/20 px-1.5 py-0.5 rounded text-[10px] uppercase font-bold tracking-wider">Sandbox Active</span>
      <span>Simulated payment test mode enabled for Stripe, PayPal, Apple Pay, Google Pay & Amazon Pay.</span>
    </div>
  {/if}

  <!-- Main Sticky Header with Glassmorphism -->
  <header class="sticky top-0 z-40 bg-slate-950/85 backdrop-blur-md border-b border-slate-900 shadow-xl transition-all">
    <!-- Top Row: Logo, Search Bar, Account & Cart -->
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-3 flex items-center justify-between gap-4">
      <!-- Brand Logo & Optional Title -->
      <a href="/" class="flex items-center gap-3 group flex-shrink-0">
        {#if store.logo_url}
          <img
            src={store.logo_url}
            alt={store.store_name || 'Logo'}
            class="{store.show_store_title === false ? 'h-14 max-w-[240px] -my-1' : 'h-10 max-w-[160px]'} object-contain rounded-xl transition-all duration-300"
          />
        {:else}
          <div class="{store.show_store_title === false ? 'w-12 h-12' : 'w-10 h-10'} rounded-xl bg-gradient-to-tr from-orange-600 to-amber-500 flex items-center justify-center shadow-lg shadow-orange-600/30 group-hover:scale-105 transition-transform duration-200">
            <span class="{store.show_store_title === false ? 'text-3xl' : 'text-2xl'} select-none">🦀</span>
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

      <!-- Searchbar in Header (Placed Above Menu) -->
      <div class="flex-1 max-w-xl mx-2 hidden sm:block">
        <form action="/" method="GET" class="relative flex items-center">
          <input
            type="text"
            name="search"
            bind:value={headerSearch}
            placeholder="Search products by title, SKU, or category..."
            class="w-full pl-10 pr-20 py-2 rounded-xl bg-slate-900/90 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500/30 text-xs shadow-inner"
          />
          <Search size={15} class="absolute left-3.5 text-slate-500 pointer-events-none" />
          <button
            type="submit"
            class="absolute right-1 px-3 py-1 rounded-lg bg-orange-600 hover:bg-orange-500 text-white text-[11px] font-bold transition-all shadow-sm"
          >
            Search
          </button>
        </form>
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

    <!-- Mobile Search Bar (under top row for smaller devices) -->
    <div class="px-4 pb-2.5 sm:hidden">
      <form action="/" method="GET" class="relative flex items-center">
        <input
          type="text"
          name="search"
          placeholder="Search products..."
          class="w-full pl-9 pr-16 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white text-xs"
        />
        <Search size={14} class="absolute left-3 text-slate-500" />
        <button type="submit" class="absolute right-1 px-2.5 py-0.5 rounded bg-orange-600 text-white text-[11px] font-bold">
          Search
        </button>
      </form>
    </div>

    <!-- Lower Row: Dynamic Customizable Navigation Menu -->
    <nav class="border-t border-slate-900/80 bg-slate-950/60 backdrop-blur-md">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex items-center {store.show_store_title === false ? 'justify-center' : 'justify-start'} gap-1 sm:gap-6 overflow-x-auto py-2 text-xs font-medium scrollbar-none">
        {#each headerMenu as item}
          <a
            href={item.url}
            class="px-3 py-1.5 rounded-lg text-slate-300 hover:text-white hover:bg-slate-900 transition-all flex items-center gap-1.5 whitespace-nowrap"
          >
            <span>{item.label}</span>
          </a>
        {/each}
      </div>
    </nav>
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

  <!-- Footer with Policy Links & No Port Numbers -->
  <footer class="border-t border-slate-900 bg-slate-950 mt-auto">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <div class="grid grid-cols-1 md:grid-cols-4 gap-8 mb-10">
        <!-- Col 1: Brand Info -->
        <div class="md:col-span-1 space-y-3">
          <div class="flex items-center gap-3">
            <span class="text-2xl select-none">🦀</span>
            <span class="font-bold text-white text-sm">{store.store_name || 'RustCraft Gear'}</span>
          </div>
          <p class="text-xs text-slate-400 leading-relaxed">
            High-performance hardware, accessories, and architecture eBooks backed by memory safety and row-locking concurrency.
          </p>
          <div class="text-[11px] text-slate-500">
            &copy; 2026 {store.store_name || 'RustCraft'}. All rights reserved.
          </div>
        </div>

        <!-- Col 2: Navigation & Policies -->
        <div class="space-y-2">
          <h4 class="text-xs font-bold text-white uppercase tracking-wider mb-3">Shipping & Links</h4>
          <ul class="space-y-2 text-xs text-slate-400">
            {#if footerMenu.length > 0}
              {#each footerMenu as fItem}
                <li><a href={fItem.url} class="hover:text-orange-400 transition-colors">{fItem.label}</a></li>
              {/each}
            {:else}
              <li><a href="/policies/shipment-policy" class="hover:text-orange-400 transition-colors">Shipment Policy & Rates Table</a></li>
              <li><a href="/policies/return-policy" class="hover:text-orange-400 transition-colors">Return & Cancellation Policy</a></li>
              <li><a href="/track" class="hover:text-orange-400 transition-colors">Track Order Status</a></li>
              <li><a href="/policies/contact" class="hover:text-orange-400 transition-colors">Contact Information</a></li>
            {/if}
          </ul>
        </div>

        <!-- Col 3: Legal & Regulatory Compliance -->
        <div class="space-y-2">
          <h4 class="text-xs font-bold text-white uppercase tracking-wider mb-3">Legal & Privacy</h4>
          <ul class="space-y-2 text-xs text-slate-400">
            <li><a href="/policies/legal-notice" class="hover:text-orange-400 transition-colors">Legal Notice (Impressum)</a></li>
            <li><a href="/policies/terms-conditions" class="hover:text-orange-400 transition-colors">Terms and Conditions (AGB)</a></li>
            <li><a href="/policies/privacy-policy" class="hover:text-orange-400 transition-colors">Privacy Policy (GDPR)</a></li>
            <li><a href="/policies/cookie-policy" class="hover:text-orange-400 transition-colors">Cookie Policy</a></li>
          </ul>
        </div>

        <!-- Col 4: Payment Methods Accepted -->
        <div class="space-y-3">
          <h4 class="text-xs font-bold text-white uppercase tracking-wider mb-3">Payment Methods</h4>
          <p class="text-xs text-slate-400">Direct instant checkout via certified gateways:</p>
          <div class="flex flex-wrap gap-2 text-xs">
            <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-slate-300 font-semibold">Stripe</span>
            <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-slate-300 font-semibold">PayPal</span>
            <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-slate-300 font-semibold">Apple Pay</span>
            <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-slate-300 font-semibold">Google Pay</span>
            <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-slate-300 font-semibold">Amazon Pay</span>
          </div>
        </div>
      </div>
    </div>
  </footer>
</div>
