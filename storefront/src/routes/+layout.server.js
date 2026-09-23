export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  try {
    const res = await fetch(`${backendUrl}/api/v1/store/info`);
    if (res.ok) {
      const data = await res.json();
      return {
        store: data.store || {},
        paymentProviders: data.payment_providers || []
      };
    }
  } catch (e) {
    console.error('Failed to load store info from backend:', e);
  }

  return {
    store: {
      store_name: 'RustCraft Gear & Software',
      currency: 'EUR',
      currency_symbol: '€',
      tax_rate_percent: 19.0,
      debug_mode: true
    },
    paymentProviders: []
  };
}
