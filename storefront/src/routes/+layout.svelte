<script>
  import '../app.css';
  import { cart, cartCount, cartSubtotal, isCartOpen } from '$lib/stores/cart.js';
  import { ShoppingBag, X, Plus, Minus, ArrowRight, ShieldCheck, Box, ExternalLink, Search } from 'lucide-svelte';

  export let data;
  $: store = data.store || {};
  $: currencySymbol = store.currency_symbol || '€';
</script>

<div class="min-h-screen flex flex-col bg-slate-950 text-slate-100 selection:bg-orange-500 selection:text-white">
  <!-- Top Announcement / Debug Bar -->
  {#if store.debug_mode}
    <div class="bg-gradient-to-r from-orange-600 to-amber-600 px-4 py-1.5 text-center text-xs font-semibold tracking-wide text-white flex items-center justify-center gap-2 shadow-sm">
      <span class="bg-white/20 px-1.5 py-0.5 rounded text-[10px] uppercase font-bold tracking-wider">Debug Sandbox Active</span>
      <span>Simulated payment test mode enabled for Stripe, PayPal, Apple Pay, Google Pay & Amazon Pay.</span>
      <a href="http://localhost:4000" target="_blank" class="underline ml-2 hover:text-orange-100 flex items-center gap-1">
        Admin Portal <ExternalLink size={12} />
      </a>
    </div>
  {/if}

  <!-- Main Navigation Bar -->
  <header class="sticky top-0 z-40 glass-nav">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-20 flex items-center justify-between gap-4">
      <!-- Brand Logo -->
      <a href="/" class="flex items-center gap-3 group">
        <div class="w-11 h-11 rounded-xl bg-gradient-to-tr from-orange-600 to-amber-500 flex items-center justify-center shadow-lg shadow-orange-600/30 group-hover:scale-105 transition-transform duration-200">
          <span class="text-2xl select-none">🦀</span>
        </div>
        <div>
          <span class="text-xl font-extrabold tracking-tight bg-gradient-to-r from-white via-slate-100 to-slate-400 bg-clip-text text-transparent">
            {store.store_name || 'RustCraft'}
          </span>
          <div class="text-[10px] text-orange-400/90 font-mono font-semibold tracking-widest uppercase">
            Rust Powered &bull; ACID Fast
          </div>
        </div>
      </a>

      <!-- Navigation Links -->
      <nav class="hidden md:flex items-center gap-8 text-sm font-medium text-slate-300">
        <a href="/" class="hover:text-orange-400 transition-colors">Catalog</a>
        <a href="/?category=Hardware" class="hover:text-orange-400 transition-colors">Hardware</a>
        <a href="/?category=Apparel" class="hover:text-orange-400 transition-colors">Apparel</a>
        <a href="/?category=Software%20%26%20Books" class="hover:text-orange-400 transition-colors">Digital & Books</a>
        <a href="/track" class="hover:text-orange-400 transition-colors flex items-center gap-1.5 text-slate-400">
          <Box size={15} /> Track Order
        </a>
      </nav>

      <!-- Right Header Actions (Cart & Admin) -->
      <div class="flex items-center gap-4">
        <a 
          href="http://localhost:4000" 
          target="_blank" 
          class="hidden sm:inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-slate-700 bg-slate-900/60 hover:bg-slate-800 text-xs font-semibold text-slate-300 hover:text-white transition-colors"
        >
          <span>Admin Portal</span>
          <ExternalLink size={13} class="text-orange-400" />
        </a>

        <!-- Cart Button Trigger -->
        <button
          id="cart-trigger-btn"
          on:click={() => isCartOpen.set(true)}
          class="relative p-2.5 rounded-xl bg-slate-900/80 hover:bg-slate-800 border border-slate-800 hover:border-orange-500/50 text-slate-200 hover:text-white transition-all shadow-md flex items-center gap-2 group"
          aria-label="View Shopping Cart"
        >
          <ShoppingBag size={20} class="group-hover:text-orange-400 transition-colors" />
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
                <div class="w-16 h-16 rounded-2xl bg-slate-800/50 flex items-center justify-center mb-4">
                  <ShoppingBag size={32} class="text-slate-600" />
                </div>
                <p class="text-base font-semibold text-slate-200">Your basket is empty</p>
                <p class="text-xs text-slate-400 mt-1 max-w-xs">Discover our hot-swappable keyboards, apparel, and Rust guides in the catalog.</p>
                <button
                  on:click={() => isCartOpen.set(false)}
                  class="mt-6 px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold shadow-lg shadow-orange-600/30 transition-all"
                >
                  Explore Products
                </button>
              </div>
            {:else}
              {#each $cart as item}
                <div class="py-4 flex gap-4 items-center">
                  {#if item.image_url}
                    <img src={item.image_url} alt={item.product_title} class="w-16 h-16 object-cover rounded-xl border border-slate-800 bg-slate-950 flex-shrink-0" />
                  {:else}
                    <div class="w-16 h-16 rounded-xl border border-slate-800 bg-slate-800/50 flex items-center justify-center text-xl flex-shrink-0">
                      📦
                    </div>
                  {/if}

                  <div class="flex-1 min-w-0">
                    <h4 class="text-sm font-bold text-white truncate">{item.product_title}</h4>
                    <p class="text-xs text-slate-400 mt-0.5 truncate">{item.variant_title}</p>
                    <div class="text-[11px] font-mono text-orange-400 mt-0.5">SKU: {item.sku}</div>

                    <div class="flex items-center justify-between mt-2.5">
                      <div class="flex items-center border border-slate-700 rounded-lg overflow-hidden bg-slate-950">
                        <button
                          on:click={() => cart.updateQuantity(item.variant_id, item.quantity - 1)}
                          class="px-2 py-1 text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
                          aria-label="Decrease quantity"
                        >
                          <Minus size={12} />
                        </button>
                        <span class="px-2.5 py-1 text-xs font-semibold text-white font-mono">{item.quantity}</span>
                        <button
                          on:click={() => cart.updateQuantity(item.variant_id, item.quantity + 1)}
                          class="px-2 py-1 text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
                          aria-label="Increase quantity"
                        >
                          <Plus size={12} />
                        </button>
                      </div>

                      <div class="text-right">
                        <div class="text-sm font-bold text-white font-mono">
                          {((item.price_cents * item.quantity) / 100).toFixed(2)} {currencySymbol}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              {/each}
            {/if}
          </div>

          <!-- Cart Footer & Checkout Action -->
          {#if $cart.length > 0}
            <div class="p-6 border-t border-slate-800 bg-slate-900/90 space-y-4">
              <div class="space-y-1.5 text-sm">
                <div class="flex justify-between text-slate-400">
                  <span>Subtotal</span>
                  <span class="font-mono text-white font-semibold">{($cartSubtotal / 100).toFixed(2)} {currencySymbol}</span>
                </div>
                <div class="flex justify-between text-slate-400 text-xs">
                  <span>Taxes & Shipping</span>
                  <span>Calculated at checkout</span>
                </div>
              </div>

              <a
                href="/checkout"
                on:click={() => isCartOpen.set(false)}
                class="w-full py-3.5 px-4 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-sm flex items-center justify-center gap-2 shadow-xl shadow-orange-600/30 transition-all hover:scale-[1.01]"
              >
                Proceed to Checkout
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

  <!-- Footer -->
  <footer class="border-t border-slate-900 bg-slate-950 mt-auto">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 flex flex-col md:flex-row items-center justify-between gap-6">
      <div class="flex items-center gap-3">
        <span class="text-2xl select-none">🦀</span>
        <div>
          <span class="font-bold text-white text-sm">{store.store_name || 'RustCraft Store'}</span>
          <p class="text-xs text-slate-400">Engineered with Axum, SQLx, PostgreSQL & SvelteKit.</p>
        </div>
      </div>

      <!-- Payment Gateways Accepted -->
      <div class="flex flex-wrap items-center justify-center gap-2 text-xs text-slate-400">
        <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-slate-300 font-semibold">Stripe</span>
        <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-slate-300 font-semibold">PayPal</span>
        <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-slate-300 font-semibold">Apple Pay</span>
        <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-slate-300 font-semibold">Google Pay</span>
        <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-slate-300 font-semibold">Amazon Pay</span>
      </div>

      <div class="text-xs text-slate-400 text-center md:text-right">
        <div>Port 8080 (Storefront) &bull; Port 4000 (Admin)</div>
        <div class="mt-1">Deployment Mode: <span class="capitalize text-orange-400 font-mono font-semibold">{store.deployment_mode || 'Development'}</span></div>
      </div>
    </div>
  </footer>
</div>
