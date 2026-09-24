export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  let store = {};
  let paymentProviders = [];
  let headerMenu = [];
  let footerMenu = [];

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
    const hRes = await fetch(`${backendUrl}/api/v1/menu?location=header`);
    if (hRes.ok) {
      headerMenu = await hRes.json();
    }
  } catch (e) {
    console.error('Failed to load header menu:', e);
  }

  try {
    const fRes = await fetch(`${backendUrl}/api/v1/menu?location=footer`);
    if (fRes.ok) {
      footerMenu = await fRes.json();
    }
  } catch (e) {
    console.error('Failed to load footer menu:', e);
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
    headerMenu: headerMenu.length > 0 ? headerMenu : [
      { id: '1', label: 'Catalog', url: '/' },
      { id: '2', label: 'Hardware', url: '/?category=Hardware' },
      { id: '3', label: 'Apparel', url: '/?category=Apparel' },
      { id: '4', label: 'Digital & Books', url: '/?category=Software+%26+Books' },
      { id: '5', label: 'Shipping Policy', url: '/policies/shipment-policy' },
      { id: '6', label: 'Track Order', url: '/track' }
    ],
    footerMenu: footerMenu.length > 0 ? footerMenu : [
      { id: 'f1', label: 'Shipping Policy', url: '/policies/shipment-policy' },
      { id: 'f2', label: 'Terms & Conditions', url: '/policies/terms-and-conditions' },
      { id: 'f3', label: 'Privacy Policy', url: '/policies/privacy-policy' },
      { id: 'f4', label: 'Legal Notice', url: '/policies/legal-notice' },
      { id: 'f5', label: 'Contact Us', url: '/policies/contact-information' }
    ],
    menuItems: headerMenu.length > 0 ? headerMenu : [
      { id: '1', label: 'Catalog', url: '/' },
      { id: '2', label: 'Hardware', url: '/?category=Hardware' },
      { id: '3', label: 'Apparel', url: '/?category=Apparel' },
      { id: '4', label: 'Digital & Books', url: '/?category=Software+%26+Books' },
      { id: '5', label: 'Shipping Policy', url: '/policies/shipment-policy' },
      { id: '6', label: 'Track Order', url: '/track' }
    ]
  };
}

