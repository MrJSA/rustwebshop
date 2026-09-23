<script>
  import { ShoppingCart, FileText, Printer, Search, CheckCircle, Truck, Clock, AlertCircle } from 'lucide-svelte';

  export let data;
  let orders = data.orders || [];
  let filterStatus = '';
  let searchQuery = '';
  let updatingOrderId = null;

  $: filteredOrders = orders.filter((o) => {
    if (filterStatus && o.order_status !== filterStatus) return false;
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      return (
        o.order_number.toLowerCase().includes(q) ||
        o.customer_name.toLowerCase().includes(q) ||
        o.customer_email.toLowerCase().includes(q)
      );
    }
    return true;
  });

  async function updateStatus(orderId, newStatus) {
    updatingOrderId = orderId;
    try {
      const res = await fetch(`/api/v1/admin/orders/${orderId}/status`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true'
        },
        body: JSON.stringify({
          order_status: newStatus,
          tracking_number: newStatus === 'shipped' ? `DHL-${Date.now().toString().slice(-8)}` : null
        })
      });

      if (res.ok) {
        orders = orders.map((o) => (o.id === orderId ? { ...o, order_status: newStatus } : o));
      }
    } catch (e) {
      console.error('Failed to update order status:', e);
    } finally {
      updatingOrderId = null;
    }
  }

  function formatPrice(cents) {
    return ((cents || 0) / 100).toFixed(2) + ' €';
  }
</script>

<svelte:head>
  <title>Orders & Slips | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-7xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <ShoppingCart size={24} class="text-orange-500" />
        Orders & Packing Slips
      </h1>
      <p class="text-xs text-slate-400 mt-1">Full WooCommerce-grade order fulfillment, instant printable packing slips and tax invoices.</p>
    </div>

    <!-- Filters -->
    <div class="flex flex-wrap items-center gap-3">
      <div class="relative">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search by order # or customer..."
          class="pl-9 pr-4 py-2 rounded-xl bg-slate-900 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500 w-52 sm:w-64"
        />
        <Search size={14} class="absolute left-3 top-3 text-slate-500 pointer-events-none" />
      </div>

      <select
        bind:value={filterStatus}
        class="px-3.5 py-2 rounded-xl bg-slate-900 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
      >
        <option value="">All Statuses</option>
        <option value="pending">Pending</option>
        <option value="processing">Processing</option>
        <option value="shipped">Shipped</option>
        <option value="delivered">Delivered</option>
        <option value="cancelled">Cancelled</option>
      </select>
    </div>
  </div>

  <!-- Orders Table -->
  <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden">
    <div class="overflow-x-auto">
      <table class="w-full text-left text-xs">
        <thead class="bg-slate-950/60 border-b border-slate-800 text-slate-400 uppercase tracking-wider text-[11px]">
          <tr>
            <th class="py-3.5 px-4 font-bold">Order #</th>
            <th class="py-3.5 px-4 font-bold">Date</th>
            <th class="py-3.5 px-4 font-bold">Customer</th>
            <th class="py-3.5 px-4 font-bold">Total</th>
            <th class="py-3.5 px-4 font-bold">Payment</th>
            <th class="py-3.5 px-4 font-bold">Fulfillment Status</th>
            <th class="py-3.5 px-4 font-bold text-right">Printable Documents</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          {#if filteredOrders.length === 0}
            <tr>
              <td colspan="7" class="text-center py-12 text-slate-400">
                No orders match your filter criteria.
              </td>
            </tr>
          {:else}
            {#each filteredOrders as order}
              <tr class="hover:bg-slate-800/30 transition-colors">
                <!-- Order Number -->
                <td class="py-4 px-4 font-mono font-bold text-white">
                  {order.order_number}
                </td>

                <!-- Date -->
                <td class="py-4 px-4 text-slate-400 whitespace-nowrap">
                  {new Date(order.created_at).toLocaleDateString()}
                </td>

                <!-- Customer Details -->
                <td class="py-4 px-4">
                  <div class="font-bold text-white">{order.customer_name}</div>
                  <div class="text-[11px] text-slate-400">{order.customer_email}</div>
                </td>

                <!-- Total -->
                <td class="py-4 px-4 font-mono font-bold text-white">
                  {formatPrice(order.total_cents)}
                </td>

                <!-- Payment Status & Provider -->
                <td class="py-4 px-4">
                  <div class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-bold uppercase {order.payment_status === 'paid' ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-slate-800 text-slate-400'}">
                    <span>{order.payment_status}</span>
                  </div>
                  <div class="text-[10px] text-slate-400 uppercase font-mono mt-0.5">{order.payment_provider}</div>
                </td>

                <!-- Order Status / Action Dropdown -->
                <td class="py-4 px-4">
                  <select
                    value={order.order_status}
                    on:change={(e) => updateStatus(order.id, e.target.value)}
                    disabled={updatingOrderId === order.id}
                    class="px-2.5 py-1 rounded-lg bg-slate-950 border border-slate-800 text-xs font-semibold text-white focus:outline-none focus:border-orange-500 capitalize"
                  >
                    <option value="pending">Pending</option>
                    <option value="processing">Processing</option>
                    <option value="shipped">Shipped</option>
                    <option value="delivered">Delivered</option>
                    <option value="cancelled">Cancelled</option>
                  </select>
                </td>

                <!-- One-Click Slip Actions -->
                <td class="py-4 px-4 text-right space-x-2 whitespace-nowrap">
                  <!-- Packing Slip Generator -->
                  <a
                    href="http://localhost:8081/api/v1/admin/orders/{order.id}/packing-slip"
                    target="_blank"
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-bold border border-slate-700 transition-colors shadow-sm"
                    title="Generate Warehouse Packing Slip"
                  >
                    <Printer size={13} class="text-orange-400" />
                    <span>Packing Slip</span>
                  </a>

                  <!-- Tax Invoice Generator -->
                  <a
                    href="http://localhost:8081/api/v1/admin/orders/{order.id}/invoice"
                    target="_blank"
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-bold border border-slate-700 transition-colors shadow-sm"
                    title="Generate Legal Tax Invoice"
                  >
                    <FileText size={13} class="text-emerald-400" />
                    <span>Tax Invoice</span>
                  </a>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>
