export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = { 'X-Dev-Mode': 'true' };

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/settings/system`, { headers });
    if (res.ok) {
      const settings = await res.json();
      return { settings };
    }
  } catch (e) {
    console.error('Failed to load system settings:', e);
  }

  return {
    settings: {
      store_name: 'RustCraft Gear',
      deployment_mode: 'development',
      debug_mode: true,
      currency: 'EUR',
      currency_symbol: '€',
      tax_rate_percent: 19.0,
      support_email: 'support@rustwebshop.local',
      company_address: 'Rustacean Way 42, 10115 Berlin, Germany',
      vat_id: 'DE314159265'
    }
  };
}
