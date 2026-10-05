export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = {};
  let settings = {};
  let paymentConfigs = [];
  let shippingProviders = [];

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/settings/system`, { headers });
    if (res.ok) {
      settings = await res.json();
    }
  } catch (e) {
    console.error('Failed to load system settings in settings overview:', e);
  }

  try {
    const payRes = await fetch(`${backendUrl}/api/v1/admin/settings/payments`, { headers });
    if (payRes.ok) {
      paymentConfigs = await payRes.json();
    }
  } catch (e) {
    console.error('Failed to load payment settings in settings overview:', e);
  }

  try {
    const shipRes = await fetch(`${backendUrl}/api/v1/admin/settings/shipping/providers`, { headers });
    if (shipRes.ok) {
      shippingProviders = await shipRes.json();
    }
  } catch (e) {
    console.error('Failed to load shipping providers in settings overview:', e);
  }

  let adminUsers = [];
  try {
    const usersRes = await fetch(`${backendUrl}/api/v1/admin/users`, { headers });
    if (usersRes.ok) {
      adminUsers = await usersRes.json();
    }
  } catch (e) {
    console.error('Failed to load admin users in settings overview:', e);
  }

  return {
    settings: Object.keys(settings).length > 0 ? settings : {
      store_name: 'RustCraft Gear & Software',
      store_subtitle: 'Rust Powered • ACID Fast',
      company_address: 'Rustacean Way 42, 10115 Berlin, Germany',
      support_email: 'support@rustwebshop.local',
      phone: '+49 (0) 30 123456-78',
      vat_id: 'DE314159265',
      tax_notice: 'Gemäß § 19 UStG wird keine Umsatzsteuer berechnet (Kleinunternehmerstatus). / Small business exemption applies according to §19 UStG.',
      currency: 'EUR',
      currency_symbol: '€',
      tax_rate_percent: 19.0,
      order_prefix_enabled: true,
      order_prefix: 'ORD',
      order_date_enabled: true,
      logo_url: ''
    },
    paymentConfigs,
    shippingProviders,
    adminUsers
  };
}
