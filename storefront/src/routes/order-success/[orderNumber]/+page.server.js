import { error } from '@sveltejs/kit';

export async function load({ params, fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const { orderNumber } = params;

  try {
    const res = await fetch(`${backendUrl}/api/v1/orders/lookup/${orderNumber}`);
    if (res.ok) {
      const data = await res.json();
      return {
        order: data.order,
        items: data.items || []
      };
    }
  } catch (e) {
    console.error(`Failed to load order ${orderNumber}:`, e);
  }

  // Return minimal placeholder if lookup fails
  return {
    order: {
      order_number: orderNumber,
      customer_name: 'Customer',
      payment_status: 'paid',
      order_status: 'processing'
    },
    items: []
  };
}
