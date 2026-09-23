export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = { 'X-Dev-Mode': 'true' };

  try {
    const [statsRes, analyticsRes, ordersRes] = await Promise.all([
      fetch(`${backendUrl}/api/v1/admin/dashboard/stats`, { headers }),
      fetch(`${backendUrl}/api/v1/admin/dashboard/sales-analytics`, { headers }),
      fetch(`${backendUrl}/api/v1/admin/orders`, { headers })
    ]);

    const stats = statsRes.ok ? await statsRes.json() : {};
    const analytics = analyticsRes.ok ? await analyticsRes.json() : [];
    const orders = ordersRes.ok ? await ordersRes.json() : [];

    return {
      stats,
      analytics,
      recentOrders: orders.slice(0, 5)
    };
  } catch (e) {
    console.error('Failed to load dashboard data:', e);
  }

  return {
    stats: {
      gross_sales_cents: 0,
      total_orders: 0,
      average_order_value_cents: 0,
      total_skus: 0,
      low_stock_skus: 0
    },
    analytics: [],
    recentOrders: []
  };
}
