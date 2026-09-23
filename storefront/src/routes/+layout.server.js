export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  let store = {};
  let paymentProviders = [];
  let menuItems = [];

  try {
    const res = await fetch(`${backendUrl}/api/v1/store/info`);
    if (res.ok) {
      const data = await res.json();
      store = data.store || {};
      paymentProviders = data.payment_providers || [];
    }
  } catch (e) {
    console.error('Failed to load store info from backend:', e);
  }

  try {
    const menuRes = await fetch(`${backendUrl}/api/v1/menu`);
    if (menuRes.ok) {
      menuItems = await menuRes.json();
    }
  } catch (e) {
    console.error('Failed to load menu items:', e);
  }

  return {
    store: Object.keys(store).length > 0 ? store : {
      store_name: 'RustCraft Gear & Software',
      currency: 'EUR',
      currency_symbol: '€',
      tax_rate_percent: 19.0,
      debug_mode: true
    },
    paymentProviders,
    menuItems: menuItems.length > 0 ? menuItems : [
      { id: '1', label: 'Catalog', url: '/' },
      { id: '2', label: 'Hardware', url: '/?category=Hardware' },
      { id: '3', label: 'Apparel', url: '/?category=Apparel' },
      { id: '4', label: 'Digital & Books', url: '/?category=Software+%26+Books' },
      { id: '5', label: 'Shipping Policy', url: '/policies/shipment-policy' },
      { id: '6', label: 'Track Order', url: '/track' }
    ]
  };
}

