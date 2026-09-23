<script>
  import { cart, cartSubtotal, cartCount } from '$lib/stores/cart.js';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { ShieldCheck, Lock, CreditCard, Truck, AlertCircle, CheckCircle2, ArrowRight } from 'lucide-svelte';

  export let data;
  $: store = data.store || {};
  $: paymentProviders = data.paymentProviders || [];

  // Form State
  let customerName = 'Joshua Rust';
  let customerEmail = 'joshua@rustwebshop.local';
  let streetAddress = 'Rustacean Strasse 10';
  let apartmentSuite = 'Apt 4B';
  let city = 'Berlin';
  let stateProvince = 'Berlin';
  let postalCode = '10115';
  let countryCode = 'DE';

  let selectedShippingRateId = '';
  let availableRates = [];
  let isFetchingRates = false;

  let selectedProvider = 'stripe';
  let isSubmitting = false;
  let errorMessage = '';

  const countries = [
    { code: 'DE', name: 'Germany (Domestic)' },
    { code: 'FR', name: 'France' },
    { code: 'NL', name: 'Netherlands' },
    { code: 'AT', name: 'Austria' },
    { code: 'IT', name: 'Italy' },
    { code: 'ES', name: 'Spain' },
    { code: 'GB', name: 'United Kingdom' },
    { code: 'US', name: 'United States' },
    { code: 'CA', name: 'Canada' }
  ];

  async function fetchShippingRates() {
    isFetchingRates = true;
    try {
      const res = await fetch(`/api/v1/shipping/rates?country_code=${countryCode}`);
      if (res.ok) {
        const data = await res.json();
        availableRates = data.rates || [];
        if (availableRates.length > 0 && !selectedShippingRateId) {
          selectedShippingRateId = availableRates[0].id;
        }
      }
    } catch (e) {
      console.warn('Could not fetch rates:', e);
    } finally {
      isFetchingRates = false;
    }
  }

  onMount(() => {
    fetchShippingRates();
  });

  $: selectedRate = availableRates.find((r) => r.id === selectedShippingRateId);
  $: shippingCostCents = selectedRate ? selectedRate.price_cents : (hasPhysicalItems ? 499 : 0);
  $: hasPhysicalItems = $cart.some((i) => !i.is_digital);

  $: taxRatePercent = store.tax_rate_percent || 19.0;
  $: taxCents = Math.round(($cartSubtotal * (taxRatePercent / 100)));
  $: grandTotalCents = $cartSubtotal + shippingCostCents + taxCents;

  async function handleSubmitOrder() {
    if ($cart.length === 0) {
      errorMessage = 'Your cart is empty.';
      return;
    }

    isSubmitting = true;
    errorMessage = '';

    const payload = {
      customer_name: customerName,
      customer_email: customerEmail,
      shipping_address: {
        full_name: customerName,
        street_address: streetAddress,
        apartment_suite: apartmentSuite,
        city,
        state_province: stateProvince,
        postal_code: postalCode,
        country_code: countryCode
      },
      shipping_rate_id: selectedShippingRateId || null,
      payment_provider: selectedProvider,
      payment_token: `tok_mock_${Date.now()}`,
      items: $cart.map((i) => ({
        variant_id: i.variant_id,
        quantity: i.quantity
      }))
    };

    try {
      const res = await fetch('/api/v1/checkout', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });

      if (!res.ok) {
        const errText = await res.text();
        throw new Error(errText || 'Checkout transaction failed');
      }

      const orderResult = await res.json();
      cart.clear();
      goto(`/order-success/${orderResult.order_number}`);
    } catch (e) {
      errorMessage = e.message || 'An error occurred during order processing.';
    } finally {
      isSubmitting = false;
    }
  }
</script>

<svelte:head>
  <title>Checkout | RustCraft Gear</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
  <div class="max-w-2xl mx-auto mb-10 text-center">
    <div class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-slate-900 border border-slate-800 text-xs font-semibold text-slate-400 mb-3">
      <Lock size={12} class="text-orange-500" /> End-to-End Encrypted Checkout
    </div>
    <h1 class="text-3xl font-extrabold text-white tracking-tight">Complete Your Order</h1>
  </div>

  {#if errorMessage}
    <div class="max-w-4xl mx-auto mb-6 p-4 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-sm flex items-center gap-3">
      <AlertCircle size={20} class="text-rose-400 flex-shrink-0" />
      <span>{errorMessage}</span>
    </div>
  {/if}

  {#if $cart.length === 0}
    <div class="text-center py-20 bg-slate-900/40 rounded-3xl border border-slate-800">
      <p class="text-base text-slate-300 font-semibold">Your cart is currently empty.</p>
      <a href="/" class="mt-4 inline-block px-5 py-2.5 rounded-xl bg-orange-600 text-white text-xs font-bold">Return to Store</a>
    </div>
  {:else}
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-10 max-w-6xl mx-auto">
      <!-- Checkout Form (8 Cols) -->
      <div class="lg:col-span-7 space-y-8">
        <!-- 1. Customer Details -->
        <div class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800">
          <h2 class="text-base font-bold text-white mb-4 flex items-center gap-2">
            <span class="w-6 h-6 rounded-full bg-orange-600 text-white text-xs font-bold flex items-center justify-center">1</span>
            Customer Information
          </h2>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label for="checkout-name" class="block text-xs font-semibold text-slate-400 mb-1">Full Name</label>
              <input
                id="checkout-name"
                type="text"
                bind:value={customerName}
                required
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
              />
            </div>
            <div>
              <label for="checkout-email" class="block text-xs font-semibold text-slate-400 mb-1">Email Address</label>
              <input
                id="checkout-email"
                type="email"
                bind:value={customerEmail}
                required
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
              />
            </div>
          </div>
        </div>

        <!-- 2. Shipping Address & Country Zone -->
        <div class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800">
          <h2 class="text-base font-bold text-white mb-4 flex items-center gap-2">
            <span class="w-6 h-6 rounded-full bg-orange-600 text-white text-xs font-bold flex items-center justify-center">2</span>
            Shipping Destination & Country Zone
          </h2>

          <div class="space-y-4">
            <div>
              <label for="checkout-country" class="block text-xs font-semibold text-slate-400 mb-1">Country / Region</label>
              <select
                id="checkout-country"
                bind:value={countryCode}
                on:change={fetchShippingRates}
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
              >
                {#each countries as c}
                  <option value={c.code}>{c.name}</option>
                {/each}
              </select>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="sm:col-span-2">
                <label for="checkout-street" class="block text-xs font-semibold text-slate-400 mb-1">Street Address</label>
                <input
                  id="checkout-street"
                  type="text"
                  bind:value={streetAddress}
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
                />
              </div>

              <div>
                <label for="checkout-city" class="block text-xs font-semibold text-slate-400 mb-1">City</label>
                <input
                  id="checkout-city"
                  type="text"
                  bind:value={city}
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
                />
              </div>

              <div>
                <label for="checkout-postal" class="block text-xs font-semibold text-slate-400 mb-1">Postal Code</label>
                <input
                  id="checkout-postal"
                  type="text"
                  bind:value={postalCode}
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
                />
              </div>
            </div>
          </div>

          <!-- Shipping Provider / Tier selection -->
          {#if hasPhysicalItems}
            <div class="mt-6 pt-5 border-t border-slate-800">
              <label class="block text-xs font-bold text-slate-300 uppercase tracking-wider mb-2">Available Shipping Options:</label>
              {#if isFetchingRates}
                <div class="text-xs text-slate-400 py-2">Recalculating zone rates...</div>
              {:else if availableRates.length === 0}
                <div class="text-xs text-slate-400 py-2">Standard International Rate: 4.99 €</div>
              {:else}
                <div class="space-y-2">
                  {#each availableRates as rate}
                    <label class="flex items-center justify-between p-3.5 rounded-xl border cursor-pointer transition-all {selectedShippingRateId === rate.id ? 'bg-orange-600/10 border-orange-500 text-white' : 'bg-slate-950/60 border-slate-800 text-slate-300 hover:border-slate-700'}">
                      <div class="flex items-center gap-3">
                        <input
                          type="radio"
                          name="shipping_rate"
                          value={rate.id}
                          bind:group={selectedShippingRateId}
                          class="accent-orange-600"
                        />
                        <div>
                          <div class="text-xs font-bold">{rate.name}</div>
                          <div class="text-[11px] text-slate-400">{rate.estimated_delivery_days}</div>
                        </div>
                      </div>
                      <div class="font-mono font-bold text-xs">{(rate.price_cents / 100).toFixed(2)} €</div>
                    </label>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- 3. Payment Method Choice -->
        <div class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800">
          <h2 class="text-base font-bold text-white mb-4 flex items-center gap-2">
            <span class="w-6 h-6 rounded-full bg-orange-600 text-white text-xs font-bold flex items-center justify-center">3</span>
            Select Payment Provider
          </h2>

          <div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
            <button
              type="button"
              on:click={() => selectedProvider = 'stripe'}
              class="p-4 rounded-xl border text-center transition-all {selectedProvider === 'stripe' ? 'bg-orange-600/15 border-orange-500 text-white ring-1 ring-orange-500' : 'bg-slate-950 border-slate-800 text-slate-300 hover:border-slate-700'}"
            >
              <CreditCard size={20} class="mx-auto mb-2 text-indigo-400" />
              <div class="text-xs font-bold">Stripe Card</div>
              <div class="text-[10px] text-slate-400 mt-0.5">Visa / MC / Amex</div>
            </button>

            <button
              type="button"
              on:click={() => selectedProvider = 'paypal'}
              class="p-4 rounded-xl border text-center transition-all {selectedProvider === 'paypal' ? 'bg-orange-600/15 border-orange-500 text-white ring-1 ring-orange-500' : 'bg-slate-950 border-slate-800 text-slate-300 hover:border-slate-700'}"
            >
              <div class="text-xl mb-1 text-sky-400 font-black">P</div>
              <div class="text-xs font-bold">PayPal</div>
              <div class="text-[10px] text-slate-400 mt-0.5">Express & PayLater</div>
            </button>

            <button
              type="button"
              on:click={() => selectedProvider = 'apple_pay'}
              class="p-4 rounded-xl border text-center transition-all {selectedProvider === 'apple_pay' ? 'bg-orange-600/15 border-orange-500 text-white ring-1 ring-orange-500' : 'bg-slate-950 border-slate-800 text-slate-300 hover:border-slate-700'}"
            >
              <div class="text-lg mb-1">🍎</div>
              <div class="text-xs font-bold">Apple Pay</div>
              <div class="text-[10px] text-slate-400 mt-0.5">Touch / Face ID</div>
            </button>

            <button
              type="button"
              on:click={() => selectedProvider = 'google_pay'}
              class="p-4 rounded-xl border text-center transition-all {selectedProvider === 'google_pay' ? 'bg-orange-600/15 border-orange-500 text-white ring-1 ring-orange-500' : 'bg-slate-950 border-slate-800 text-slate-300 hover:border-slate-700'}"
            >
              <div class="text-lg mb-1 font-bold text-amber-400">G</div>
              <div class="text-xs font-bold">Google Pay</div>
              <div class="text-[10px] text-slate-400 mt-0.5">Instant Checkout</div>
            </button>

            <button
              type="button"
              on:click={() => selectedProvider = 'amazon_pay'}
              class="p-4 rounded-xl border text-center transition-all {selectedProvider === 'amazon_pay' ? 'bg-orange-600/15 border-orange-500 text-white ring-1 ring-orange-500' : 'bg-slate-950 border-slate-800 text-slate-300 hover:border-slate-700'}"
            >
              <div class="text-lg mb-1 font-bold text-amber-500">a</div>
              <div class="text-xs font-bold">Amazon Pay</div>
              <div class="text-[10px] text-slate-400 mt-0.5">Amazon Account</div>
            </button>
          </div>

          <div class="mt-4 p-3 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-400 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <ShieldCheck size={16} class="text-emerald-400" />
              <span>Sandbox Test Mode Active &bull; No real charge will occur.</span>
            </div>
            <span class="text-slate-500 uppercase font-mono text-[10px]">Test Gateway</span>
          </div>
        </div>
      </div>

      <!-- Order Summary (4 Cols) -->
      <div class="lg:col-span-5">
        <div class="sticky top-28 p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
          <h3 class="text-base font-bold text-white tracking-tight border-b border-slate-800 pb-3">
            Order Summary ({$cartCount} { $cartCount === 1 ? 'item' : 'items' })
          </h3>

          <!-- Items list -->
          <div class="space-y-3 max-h-60 overflow-y-auto pr-1 divide-y divide-slate-800/60">
            {#each $cart as item}
              <div class="pt-3 first:pt-0 flex items-center justify-between text-xs">
                <div class="min-w-0 pr-3">
                  <div class="font-bold text-white truncate">{item.product_title}</div>
                  <div class="text-slate-400 truncate">{item.variant_title} &times; {item.quantity}</div>
                </div>
                <div class="font-mono font-bold text-white flex-shrink-0">
                  {((item.price_cents * item.quantity) / 100).toFixed(2)} €
                </div>
              </div>
            {/each}
          </div>

          <!-- Price Calculations -->
          <div class="border-t border-slate-800 pt-4 space-y-2 text-xs">
            <div class="flex justify-between text-slate-400">
              <span>Subtotal</span>
              <span class="font-mono text-slate-200">{($cartSubtotal / 100).toFixed(2)} €</span>
            </div>
            <div class="flex justify-between text-slate-400">
              <span>Shipping ({countryCode})</span>
              <span class="font-mono text-slate-200">{(shippingCostCents / 100).toFixed(2)} €</span>
            </div>
            <div class="flex justify-between text-slate-400">
              <span>Estimated Tax ({taxRatePercent}%)</span>
              <span class="font-mono text-slate-200">{(taxCents / 100).toFixed(2)} €</span>
            </div>
            <div class="border-t border-slate-800 pt-3 flex justify-between text-base font-bold text-white">
              <span>Total</span>
              <span class="font-mono text-orange-400">{(grandTotalCents / 100).toFixed(2)} €</span>
            </div>
          </div>

          <button
            id="submit-order-btn"
            on:click={handleSubmitOrder}
            disabled={isSubmitting}
            class="w-full py-4 px-6 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-sm shadow-xl shadow-orange-600/30 transition-all flex items-center justify-center gap-2 hover:scale-[1.01] disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {#if isSubmitting}
              <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              <span>Locking Inventory & Processing...</span>
            {:else}
              <span>Pay {(grandTotalCents / 100).toFixed(2)} €</span>
              <ArrowRight size={16} />
            {/if}
          </button>

          <p class="text-[11px] text-slate-400 text-center leading-relaxed">
            By placing this order you authorize RustCraft to capture payment through {selectedProvider}. ACID row-level locking ensures stock is held instantly.
          </p>
        </div>
      </div>
    </div>
  {/if}
</div>
