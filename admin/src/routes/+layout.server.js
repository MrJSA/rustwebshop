export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/settings/system`, {
      headers: { 'X-Dev-Mode': 'true' }
    });
    if (res.ok) {
      const settings = await res.json();
      return { settings };
    }
  } catch (e) {
    console.error('Failed to load system settings in admin layout:', e);
  }

  return {
    settings: {
      store_name: 'RustCraft Gear',
      deployment_mode: 'development',
      debug_mode: true,
      currency_symbol: '€'
    }
  };
}
