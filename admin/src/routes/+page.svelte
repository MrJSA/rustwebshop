<script>
  import { 
    DollarSign, ShoppingCart, TrendingUp, AlertTriangle, 
    Layers, ArrowUpRight, FileText, Printer, Eye 
  } from 'lucide-svelte';

  export let data;
  $: stats = data.stats || {};
  $: analytics = data.analytics || [];
  $: recentOrders = data.recentOrders || [];

  function formatPrice(cents) {
    return ((cents || 0) / 100).toFixed(2) + ' €';
  }
</script>

<svelte:head>
  <title>Dashboard Overview | RustCraft Admin</title>
</svelte:head>

<div class="space-y-8 max-w-7xl mx-auto">
  <!-- Page Header -->
  <div>
    <h1 class="text-2xl font-black text-white tracking-tight">Executive Dashboard</h1>
    <p class="text-xs text-slate-400 mt-1">Real-time metrics, warehouse inventory status, and recent customer activity.</p>
  </div>

  <!-- KPI Metric Cards -->
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
    <!-- 1. Gross Revenue -->
    <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
      <div class="flex items-center justify-between text-slate-400 mb-3">
        <span class="text-xs font-semibold uppercase tracking-wider">Gross Sales</span>
        <div class="w-8 h-8 rounded-lg bg-emerald-500/10 text-emerald-400 flex items-center justify-center">
          <DollarSign size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-white font-mono">
        {formatPrice(stats.gross_sales_cents)}
      </div>
      <div class="mt-2 text-[11px] text-emerald-400 flex items-center gap-1">
        <ArrowUpRight size={13} />
        <span>ACID Settled Total</span>
      </div>
    </div>

    <!-- 2. Orders Count -->
    <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
      <div class="flex items-center justify-between text-slate-400 mb-3">
        <span class="text-xs font-semibold uppercase tracking-wider">Orders</span>
        <div class="w-8 h-8 rounded-lg bg-orange-500/10 text-orange-400 flex items-center justify-center">
          <ShoppingCart size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-white font-mono">
        {stats.total_orders || 0}
      </div>
      <div class="mt-2 text-[11px] text-slate-400">
        <span>{stats.processing_orders || 0} in processing</span>
      </div>
    </div>

    <!-- 3. Average Order Value -->
    <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
      <div class="flex items-center justify-between text-slate-400 mb-3">
        <span class="text-xs font-semibold uppercase tracking-wider">Avg Order Value</span>
        <div class="w-8 h-8 rounded-lg bg-sky-500/10 text-sky-400 flex items-center justify-center">
          <TrendingUp size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-white font-mono">
        {formatPrice(stats.average_order_value_cents)}
      </div>
      <div class="mt-2 text-[11px] text-slate-400">
        <span>Per completed cart</span>
      </div>
    </div>

    <!-- 4. Total Active SKUs -->
    <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-md">
      <div class="flex items-center justify-between text-slate-400 mb-3">
        <span class="text-xs font-semibold uppercase tracking-wider">Total SKUs</span>
        <div class="w-8 h-8 rounded-lg bg-purple-500/10 text-purple-400 flex items-center justify-center">
          <Layers size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-white font-mono">
        {stats.total_skus || 0}
      </div>
      <div class="mt-2 text-[11px] text-slate-400">
        <span>Tracked in database</span>
      </div>
    </div>

    <!-- 5. Low Stock Alert -->
    <div class="p-5 rounded-2xl {stats.low_stock_skus > 0 ? 'bg-amber-950/20 border-amber-500/40' : 'bg-slate-900 border-slate-800'} border shadow-md">
      <div class="flex items-center justify-between text-slate-400 mb-3">
        <span class="text-xs font-semibold uppercase tracking-wider text-amber-400">Low Stock Alert</span>
        <div class="w-8 h-8 rounded-lg bg-amber-500/20 text-amber-400 flex items-center justify-center">
          <AlertTriangle size={16} />
        </div>
      </div>
      <div class="text-2xl font-black text-amber-400 font-mono">
        {stats.low_stock_skus || 0}
      </div>
      <div class="mt-2 text-[11px] text-amber-300">
        <a href="/logistics" class="underline hover:text-amber-200">Inspect in Warehouse &rarr;</a>
      </div>
    </div>
  </div>

  <!-- Sales Chart Visualizer -->
  <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h3 class="text-base font-bold text-white tracking-tight">Sales Analytics (Last 30 Days)</h3>
        <p class="text-xs text-slate-400">Time-series daily revenue capture.</p>
      </div>
      <span class="px-3 py-1 rounded-lg bg-slate-800 text-xs font-mono text-slate-300">Auto-updating</span>
    </div>

    {#if analytics.length === 0}
      <div class="py-12 text-center text-xs text-slate-400 bg-slate-950/40 rounded-xl border border-slate-800">
        No sales recorded yet. Place test orders from the storefront (port 8080) to populate graph.
      </div>
    {:else}
      <div class="h-44 flex items-end gap-2 pt-6 px-2">
        {#each analytics as point}
          {@const maxSales = Math.max(...analytics.map(p => p.sales_cents), 1)}
          {@const heightPercent = Math.max(12, Math.round((point.sales_cents / maxSales) * 100))}
          <div class="flex-1 flex flex-col items-center gap-2 group relative">
            <!-- Tooltip -->
            <div class="absolute -top-10 bg-slate-800 border border-slate-700 px-2 py-1 rounded text-[10px] text-white opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-20">
              {point.date}: {formatPrice(point.sales_cents)} ({point.orders_count} orders)
            </div>

            <!-- Bar -->
            <div 
              class="w-full bg-gradient-to-t from-orange-600 to-amber-400 rounded-t-md hover:from-orange-500 hover:to-amber-300 transition-all duration-300"
              style="height: {heightPercent}%;"
            ></div>
            <span class="text-[9px] font-mono text-slate-400 truncate w-full text-center">{point.date.slice(5)}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Recent Orders Table -->
  <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-base font-bold text-white tracking-tight">Recent Orders</h3>
        <p class="text-xs text-slate-400">Incoming customer orders and dispatch status.</p>
      </div>
      <a href="/orders" class="text-xs text-orange-400 hover:text-orange-300 font-bold">View All Orders &rarr;</a>
    </div>

    {#if recentOrders.length === 0}
      <div class="py-12 text-center text-xs text-slate-400 bg-slate-950/40 rounded-xl border border-slate-800">
        No orders received yet.
      </div>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs">
          <thead>
            <tr class="border-b border-slate-800 text-slate-400">
              <th class="py-3 px-3">Order #</th>
              <th class="py-3 px-3">Customer</th>
              <th class="py-3 px-3">Total</th>
              <th class="py-3 px-3">Provider</th>
              <th class="py-3 px-3">Payment</th>
              <th class="py-3 px-3">Status</th>
              <th class="py-3 px-3 text-right">Slips</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-800/60">
            {#each recentOrders as ord}
              <tr class="hover:bg-slate-800/30 transition-colors">
                <td class="py-3 px-3 font-mono font-bold text-white">{ord.order_number}</td>
                <td class="py-3 px-3">
                  <div class="font-bold text-slate-200">{ord.customer_name}</div>
                  <div class="text-[11px] text-slate-400">{ord.customer_email}</div>
                </td>
                <td class="py-3 px-3 font-mono font-bold text-white">{formatPrice(ord.total_cents)}</td>
                <td class="py-3 px-3 uppercase font-mono text-[11px] text-slate-400">{ord.payment_provider}</td>
                <td class="py-3 px-3">
                  <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase {ord.payment_status === 'paid' ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-slate-800 text-slate-400'}">
                    {ord.payment_status}
                  </span>
                </td>
                <td class="py-3 px-3">
                  <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase bg-orange-500/10 text-orange-400 border border-orange-500/20">
                    {ord.order_status}
                  </span>
                </td>
                <td class="py-3 px-3 text-right space-x-1">
                  <!-- One-Click Packing Slip -->
                  <a
                    href="http://localhost:8081/api/v1/admin/orders/{ord.id}/packing-slip"
                    target="_blank"
                    class="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-[11px] font-semibold transition-colors"
                    title="View Warehouse Packing Slip"
                  >
                    <Printer size={12} />
                    <span>Packing</span>
                  </a>
                  <!-- One-Click Tax Invoice -->
                  <a
                    href="http://localhost:8081/api/v1/admin/orders/{ord.id}/invoice"
                    target="_blank"
                    class="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-[11px] font-semibold transition-colors"
                    title="View Legal Tax Invoice"
                  >
                    <FileText size={12} />
                    <span>Invoice</span>
                  </a>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>
