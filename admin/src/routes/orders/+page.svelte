<script>
  import {
    ShoppingCart,
    FileText,
    Printer,
    Search,
    CheckCircle,
    Truck,
    Clock,
    AlertCircle,
    X,
    Eye,
    RotateCcw,
    Ban,
    Download,
    CreditCard,
    MapPin,
    Building,
    Phone,
    Mail,
    User
  } from 'lucide-svelte';

  export let data;
  let orders = data.orders || [];
  let filterStatus = '';
  let searchQuery = '';

  // Order Details Modal state
  let selectedOrder = null;
  let selectedOrderItems = [];
  let isLoadingDetails = false;
  let isUpdatingStatus = false;
  let statusChangeTarget = '';
  let trackingNumberInput = '';
  let modalActionMessage = '';
  let modalActionError = '';

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

  async function openOrderDetails(order) {
    selectedOrder = order;
    statusChangeTarget = order.order_status;
    trackingNumberInput = order.tracking_number || '';
    modalActionMessage = '';
    modalActionError = '';
    isLoadingDetails = true;

    try {
      const res = await fetch(`/api/v1/admin/orders/${order.id}`, {
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        const result = await res.json();
        selectedOrder = result.order || result;
        selectedOrderItems = result.items || [];
      }
    } catch (e) {
      console.error('Failed to load order details:', e);
    } finally {
      isLoadingDetails = false;
    }
  }

  async function handleUpdateStatus() {
    modalActionError = '';
    modalActionMessage = '';

    if (statusChangeTarget === 'shipped' && !trackingNumberInput.trim()) {
      modalActionError = 'A tracking number is required before marking order as Shipped.';
      return;
    }

    isUpdatingStatus = true;
    try {
      const res = await fetch(`/api/v1/admin/orders/${selectedOrder.id}/status`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
        body: JSON.stringify({
          order_status: statusChangeTarget,
          tracking_number: statusChangeTarget === 'shipped' ? trackingNumberInput.trim() : selectedOrder.tracking_number
        })
      });

      if (res.ok) {
        modalActionMessage = `Order status successfully updated to ${statusChangeTarget}!`;
        selectedOrder.order_status = statusChangeTarget;
        if (statusChangeTarget === 'shipped') selectedOrder.tracking_number = trackingNumberInput.trim();
        // Update in table
        orders = orders.map((o) => (o.id === selectedOrder.id ? { ...selectedOrder } : o));
        setTimeout(() => modalActionMessage = '', 3500);
      } else {
        const err = await res.text();
        modalActionError = err || 'Failed to update order status.';
      }
    } catch (e) {
      modalActionError = 'Network error updating order status.';
    } finally {
      isUpdatingStatus = false;
    }
  }

  async function handleRefund() {
    if (!confirm(`Are you sure you want to issue a full refund for Order ${selectedOrder.order_number}?`)) return;
    try {
      const res = await fetch(`/api/v1/admin/orders/${selectedOrder.id}/refund`, {
        method: 'POST',
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        selectedOrder.payment_status = 'refunded';
        orders = orders.map((o) => (o.id === selectedOrder.id ? { ...o, payment_status: 'refunded' } : o));
        modalActionMessage = 'Payment marked as refunded.';
        setTimeout(() => modalActionMessage = '', 3500);
      }
    } catch (e) {
      modalActionError = 'Failed to refund order.';
    }
  }

  async function handleCancel() {
    if (!confirm(`Are you sure you want to CANCEL Order ${selectedOrder.order_number}?`)) return;
    try {
      const res = await fetch(`/api/v1/admin/orders/${selectedOrder.id}/cancel`, {
        method: 'POST',
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        selectedOrder.order_status = 'cancelled';
        statusChangeTarget = 'cancelled';
        orders = orders.map((o) => (o.id === selectedOrder.id ? { ...o, order_status: 'cancelled' } : o));
        modalActionMessage = 'Order has been cancelled.';
        setTimeout(() => modalActionMessage = '', 3500);
      }
    } catch (e) {
      modalActionError = 'Failed to cancel order.';
    }
  }

  // Open PDF / Printable slip with authentication token
  async function downloadSlip(type) {
    const token = localStorage.getItem('admin_token') || '';
    const endpoint = `/api/v1/admin/orders/${selectedOrder.id}/${type}?token=${encodeURIComponent(token)}`;
    try {
      const res = await fetch(endpoint, {
        headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) }
      });
      if (!res.ok) {
        // Fallback open with query token
        window.open(endpoint, '_blank');
        return;
      }
      const blob = await res.blob();
      const objectUrl = URL.createObjectURL(blob);
      const win = window.open(objectUrl, '_blank');
      if (!win) {
        const a = document.createElement('a');
        a.href = objectUrl;
        a.download = `${type}-${selectedOrder.order_number}.pdf`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
      }
      setTimeout(() => URL.revokeObjectURL(objectUrl), 60000);
    } catch (e) {
      window.open(endpoint, '_blank');
    }
  }

  function formatPrice(cents) {
    return ((cents || 0) / 100).toFixed(2) + ' €';
  }

  function getStatusBadgeClass(status) {
    switch (status) {
      case 'paid':
      case 'completed':
      case 'delivered':
        return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
      case 'processing':
        return 'bg-sky-500/10 text-sky-400 border-sky-500/20';
      case 'shipped':
        return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
      case 'cancelled':
      case 'refunded':
      case 'failed':
        return 'bg-rose-500/10 text-rose-400 border-rose-500/20';
      default:
        return 'bg-slate-800 text-slate-400 border-slate-700';
    }
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
        Orders & Fulfillment Slips
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Click any order row to view detailed shipping & billing addresses, refund/cancel, enforce tracking numbers, and download PDF slips.
      </p>
    </div>

    <!-- Filters -->
    <div class="flex flex-wrap items-center gap-3">
      <div class="relative">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search order #, customer, email..."
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
        <option value="completed">Completed</option>
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
            <th class="py-3.5 px-4 font-bold">Payment Status</th>
            <th class="py-3.5 px-4 font-bold">Fulfillment Status</th>
            <th class="py-3.5 px-4 font-bold text-right">Details & Actions</th>
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
              <tr
                on:click={() => openOrderDetails(order)}
                class="hover:bg-slate-800/40 transition-colors cursor-pointer group"
              >
                <!-- Order Number -->
                <td class="py-4 px-4 font-mono font-bold text-white group-hover:text-orange-400 transition-colors">
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

                <!-- Payment Status Badge -->
                <td class="py-4 px-4">
                  <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded text-[10px] font-bold uppercase border {getStatusBadgeClass(order.payment_status)}">
                    {order.payment_status}
                  </span>
                  <div class="text-[10px] text-slate-500 uppercase font-mono mt-0.5">{order.payment_provider}</div>
                </td>

                <!-- Fulfillment Status Badge (No Dropdown) -->
                <td class="py-4 px-4">
                  <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded text-[10px] font-bold uppercase border {getStatusBadgeClass(order.order_status)}">
                    {order.order_status}
                  </span>
                  {#if order.tracking_number}
                    <div class="text-[10px] text-orange-400/90 font-mono mt-0.5">#{order.tracking_number}</div>
                  {/if}
                </td>

                <!-- Inspect Details Button -->
                <td class="py-4 px-4 text-right whitespace-nowrap">
                  <button
                    on:click|stopPropagation={() => openOrderDetails(order)}
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-bold border border-slate-700 transition-colors shadow-sm"
                  >
                    <Eye size={13} class="text-orange-400" />
                    <span>View Details</span>
                  </button>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<!-- Detailed Order View Modal -->
{#if selectedOrder}
  <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/85 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="w-full max-w-4xl bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 sm:p-8 space-y-6 max-h-[92vh] overflow-y-auto">
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-slate-800 pb-4">
        <div>
          <div class="flex items-center gap-3">
            <span class="font-mono text-xl font-black text-white">{selectedOrder.order_number}</span>
            <span class="px-2.5 py-0.5 rounded text-[10px] font-bold uppercase border {getStatusBadgeClass(selectedOrder.order_status)}">
              {selectedOrder.order_status}
            </span>
            <span class="px-2.5 py-0.5 rounded text-[10px] font-bold uppercase border {getStatusBadgeClass(selectedOrder.payment_status)}">
              {selectedOrder.payment_status}
            </span>
          </div>
          <p class="text-xs text-slate-400 mt-1">
            Placed on {new Date(selectedOrder.created_at).toLocaleString()} &bull; Provider: <strong class="text-white uppercase font-mono">{selectedOrder.payment_provider}</strong>
          </p>
        </div>
        <button on:click={() => selectedOrder = null} class="p-1 text-slate-400 hover:text-white">
          <X size={22} />
        </button>
      </div>

      {#if modalActionMessage}
        <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
          <CheckCircle size={16} />
          <span>{modalActionMessage}</span>
        </div>
      {/if}

      {#if modalActionError}
        <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={16} />
          <span>{modalActionError}</span>
        </div>
      {/if}

      <!-- Customer Contact Bar -->
      <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs">
        <div class="flex items-center gap-2.5">
          <User size={16} class="text-orange-400" />
          <div>
            <span class="text-slate-400 block text-[10px]">Customer Name</span>
            <span class="font-bold text-white">{selectedOrder.customer_name}</span>
          </div>
        </div>
        <div class="flex items-center gap-2.5">
          <Mail size={16} class="text-sky-400" />
          <div>
            <span class="text-slate-400 block text-[10px]">Email Address</span>
            <span class="font-mono text-slate-300 font-semibold">{selectedOrder.customer_email}</span>
          </div>
        </div>
        <div class="flex items-center gap-2.5">
          <Phone size={16} class="text-emerald-400" />
          <div>
            <span class="text-slate-400 block text-[10px]">Phone Number</span>
            <span class="font-mono text-slate-300 font-semibold">{selectedOrder.customer_phone || selectedOrder.shipping_address?.phone || '+49 (0) 30 123456-78'}</span>
          </div>
        </div>
      </div>

      <!-- Side-by-Side Addresses -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
        <!-- Shipping Address Card -->
        <div class="p-5 rounded-2xl bg-slate-950 border border-slate-800 space-y-2">
          <h4 class="font-bold text-white flex items-center gap-2 text-sm border-b border-slate-800/80 pb-2">
            <MapPin size={16} class="text-orange-400" />
            <span>Shipping / Delivery Address</span>
          </h4>
          <div class="text-slate-300 leading-relaxed font-mono">
            {#if typeof selectedOrder.shipping_address === 'object'}
              <div>{selectedOrder.shipping_address.street || selectedOrder.shipping_address.street_address || selectedOrder.shipping_address.address} {selectedOrder.shipping_address.apartment ? `#${selectedOrder.shipping_address.apartment}` : ''}</div>
              <div>{selectedOrder.shipping_address.postal_code || selectedOrder.shipping_address.zip} {selectedOrder.shipping_address.city}, {selectedOrder.shipping_address.state || ''}</div>
              <div class="text-slate-400 font-sans mt-1">Country: <strong class="text-white">{selectedOrder.shipping_address.country_code || selectedOrder.shipping_address.country || 'DE'}</strong></div>
            {:else}
              <div>{selectedOrder.shipping_address}</div>
            {/if}
          </div>
        </div>

        <!-- Billing Address Card -->
        <div class="p-5 rounded-2xl bg-slate-950 border border-slate-800 space-y-2">
          <h4 class="font-bold text-white flex items-center gap-2 text-sm border-b border-slate-800/80 pb-2">
            <Building size={16} class="text-sky-400" />
            <span>Billing Address</span>
          </h4>
          <div class="text-slate-300 leading-relaxed font-mono">
            {#if typeof selectedOrder.billing_address === 'object'}
              <div>{selectedOrder.billing_address.street || selectedOrder.billing_address.street_address || selectedOrder.billing_address.address} {selectedOrder.billing_address.apartment ? `#${selectedOrder.billing_address.apartment}` : ''}</div>
              <div>{selectedOrder.billing_address.postal_code || selectedOrder.billing_address.zip} {selectedOrder.billing_address.city}, {selectedOrder.billing_address.state || ''}</div>
              <div class="text-slate-400 font-sans mt-1">Country: <strong class="text-white">{selectedOrder.billing_address.country_code || selectedOrder.billing_address.country || 'DE'}</strong></div>
            {:else}
              <div>{selectedOrder.billing_address}</div>
            {/if}
          </div>
        </div>
      </div>

      <!-- Line Items Table -->
      <div class="rounded-2xl bg-slate-950 border border-slate-800 overflow-hidden">
        <div class="p-4 bg-slate-900/60 font-bold text-white text-xs border-b border-slate-800">
          Order Line Items & Versions
        </div>
        <table class="w-full text-left text-xs">
          <thead class="bg-slate-950 text-slate-400 uppercase text-[10px] border-b border-slate-800/60">
            <tr>
              <th class="py-2.5 px-4">Item & Version</th>
              <th class="py-2.5 px-4">SKU</th>
              <th class="py-2.5 px-4 text-center">Qty</th>
              <th class="py-2.5 px-4 text-right">Unit Price</th>
              <th class="py-2.5 px-4 text-right">Total</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-800/60">
            {#each selectedOrderItems as item}
              <tr>
                <td class="py-3 px-4 font-bold text-white">
                  {item.product_title}
                  <span class="block text-[11px] font-normal text-slate-400 font-mono">{item.variant_title}</span>
                </td>
                <td class="py-3 px-4 font-mono text-slate-400">{item.sku}</td>
                <td class="py-3 px-4 text-center font-bold text-white">{item.quantity}</td>
                <td class="py-3 px-4 text-right font-mono">{formatPrice(item.unit_price_cents)}</td>
                <td class="py-3 px-4 text-right font-mono font-bold text-white">{formatPrice(item.total_price_cents)}</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <!-- Totals summary -->
        <div class="p-4 bg-slate-900/40 border-t border-slate-800 space-y-1.5 text-xs">
          <div class="flex justify-between text-slate-400">
            <span>Subtotal:</span>
            <span class="font-mono text-white">{formatPrice(selectedOrder.subtotal_cents)}</span>
          </div>
          <div class="flex justify-between text-slate-400">
            <span>Shipping Fee:</span>
            <span class="font-mono text-white">{formatPrice(selectedOrder.shipping_cost_cents)}</span>
          </div>
          <div class="flex justify-between text-slate-400">
            <span>Estimated VAT:</span>
            <span class="font-mono text-white">{formatPrice(selectedOrder.tax_cents)}</span>
          </div>
          <div class="flex justify-between text-sm font-bold text-white pt-2 border-t border-slate-800">
            <span>Grand Total:</span>
            <span class="font-mono text-orange-400 font-black">{formatPrice(selectedOrder.total_cents)}</span>
          </div>
        </div>
      </div>

      <!-- Action Section: Status Update + Tracking Number Enforcement -->
      <div class="p-5 rounded-2xl bg-slate-950 border border-slate-800 space-y-4 text-xs">
        <h4 class="font-bold text-white text-sm">Fulfillment Status & Logistics Tracking</h4>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label class="block font-semibold text-slate-300 mb-1">Update Order Status</label>
            <select
              bind:value={statusChangeTarget}
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-900 border border-slate-800 text-white font-bold capitalize focus:outline-none focus:border-orange-500"
            >
              <option value="pending">Pending</option>
              <option value="processing">Processing</option>
              <option value="shipped">Shipped (Requires Tracking #)</option>
              <option value="completed">Completed</option>
              <option value="cancelled">Cancelled</option>
            </select>
          </div>

          <div>
            <label class="block font-semibold text-slate-300 mb-1">
              Tracking Number {statusChangeTarget === 'shipped' ? '(Required for Shipped)' : '(Optional)'}
            </label>
            <input
              type="text"
              bind:value={trackingNumberInput}
              placeholder="e.g. DHL-00340434234"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-900 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div class="flex flex-wrap items-center justify-between gap-3 pt-2">
          <!-- Critical Order Actions: Refund & Cancel -->
          <div class="flex items-center gap-2">
            {#if selectedOrder.payment_status === 'paid'}
              <button
                type="button"
                on:click={handleRefund}
                class="px-3.5 py-2 rounded-xl bg-rose-600/10 hover:bg-rose-600/20 text-rose-400 border border-rose-500/30 font-bold flex items-center gap-1.5 transition-colors"
              >
                <RotateCcw size={14} />
                <span>Refund Payment</span>
              </button>
            {/if}

            {#if selectedOrder.order_status !== 'cancelled'}
              <button
                type="button"
                on:click={handleCancel}
                class="px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 font-bold flex items-center gap-1.5 transition-colors"
              >
                <Ban size={14} />
                <span>Cancel Order</span>
              </button>
            {/if}
          </div>

          <button
            type="button"
            on:click={handleUpdateStatus}
            disabled={isUpdatingStatus}
            class="px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold transition-all shadow-md flex items-center gap-2 disabled:opacity-50"
          >
            <Truck size={15} />
            <span>{isUpdatingStatus ? 'Updating...' : 'Save Order Status'}</span>
          </button>
        </div>
      </div>

      <!-- Downloadable Documents Section -->
      <div class="border-t border-slate-800 pt-4 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <span class="text-xs font-bold text-white block">Downloadable Fulfillment Documents (PDF)</span>
          <span class="text-[11px] text-slate-400">Generate warehouse packing slips and legal tax invoices.</span>
        </div>

        <div class="flex items-center gap-3">
          <button
            type="button"
            on:click={() => downloadSlip('packing-slip')}
            class="px-4 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-bold border border-slate-700 transition-colors shadow-sm flex items-center gap-2"
          >
            <Printer size={15} class="text-orange-400" />
            <span>Download Packing Slip (PDF)</span>
          </button>

          <button
            type="button"
            on:click={() => downloadSlip('invoice')}
            class="px-4 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-bold border border-slate-700 transition-colors shadow-sm flex items-center gap-2"
          >
            <FileText size={15} class="text-emerald-400" />
            <span>Download Tax Invoice (PDF)</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
