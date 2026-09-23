<script>
  import { onMount } from 'svelte';
  import { customer } from '$lib/stores/customer.js';
  import { goto } from '$app/navigation';
  import { Box, ArrowLeft, ExternalLink, Clock, CheckCircle2, Truck } from 'lucide-svelte';

  let orders = [];
  let isLoading = true;

  onMount(async () => {
    if (!$customer || !$customer.isLoggedIn) {
      goto('/account/login');
      return;
    }

    try {
      const res = await fetch('/api/v1/customer/orders', {
        headers: { Authorization: `Bearer ${$customer.token}` }
      });
      if (res.ok) {
        orders = await res.json();
      }
    } catch (e) {
      console.error('Failed to load customer orders:', e);
    } finally {
      isLoading = false;
    }
  });
</script>

<svelte:head>
  <title>My Orders | RustCraft</title>
</svelte:head>

<div class="max-w-4xl mx-auto px-4 py-12 space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <a href="/" class="inline-flex items-center gap-1.5 text-xs text-slate-400 hover:text-white mb-2 transition-colors">
        <ArrowLeft size={14} />
        <span>Back to Store</span>
      </a>
      <h1 class="text-2xl font-bold text-white tracking-tight flex items-center gap-2">
        <Box size={24} class="text-orange-500" />
        Order History
      </h1>
      <p class="text-xs text-slate-400 mt-0.5">Track your past purchases, shipment statuses, and delivery tracking codes.</p>
    </div>
  </div>

  {#if isLoading}
    <div class="text-center py-16 text-xs text-slate-400">Loading your orders...</div>
  {:else if orders.length === 0}
    <div class="p-12 text-center rounded-3xl bg-slate-900 border border-slate-800 text-slate-400 space-y-3">
      <Box size={36} class="mx-auto text-slate-600" />
      <p class="text-sm font-bold text-slate-200">No orders found</p>
      <p class="text-xs text-slate-500">You haven't placed any orders with this customer account yet.</p>
      <a href="/" class="inline-block mt-2 px-4 py-2 rounded-xl bg-orange-600 text-white text-xs font-bold hover:bg-orange-500 transition-colors">
        Start Shopping
      </a>
    </div>
  {:else}
    <div class="space-y-4">
      {#each orders as o}
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div>
            <div class="flex items-center gap-2 mb-1">
              <span class="font-mono font-bold text-white text-sm">{o.order_number}</span>
              <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase {o.order_status === 'shipped' ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'}">
                {o.order_status}
              </span>
            </div>
            <div class="text-xs text-slate-400 flex items-center gap-3">
              <span>Date: {new Date(o.created_at).toLocaleDateString()}</span>
              <span>&bull;</span>
              <span>Paid via <strong class="capitalize text-slate-200">{o.payment_provider}</strong></span>
            </div>
            {#if o.tracking_number}
              <div class="mt-2 inline-flex items-center gap-1.5 text-xs text-sky-400 font-mono">
                <Truck size={13} />
                <span>Tracking: {o.tracking_number}</span>
              </div>
            {/if}
          </div>

          <div class="flex items-center gap-4 self-end sm:self-center">
            <div class="text-right">
              <div class="text-sm font-bold font-mono text-white">{(o.total_cents / 100).toFixed(2)} €</div>
              <div class="text-[10px] text-emerald-400 font-bold uppercase">{o.payment_status}</div>
            </div>
            <a
              href="/track"
              class="px-3 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-xs font-semibold text-slate-200 hover:text-white transition-colors flex items-center gap-1"
            >
              <span>Track</span>
              <ExternalLink size={12} />
            </a>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
