export async function load({ fetch, locals }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const admin = locals.admin || null;

  // Logged-out visitors (login page) get no store data at all
  if (admin && !admin.is_default) {
    try {
      const res = await fetch(`${backendUrl}/api/v1/admin/settings/system`);
      if (res.ok) {
        return { admin, settings: await res.json() };
      }
    } catch (e) {
      console.error('Failed to load system settings in admin layout:', e);
    }
  }

  return {
    admin,
    settings: {
      store_name: 'Shop Admin',
      deployment_mode: 'development',
      debug_mode: false,
      currency_symbol: '€'
    }
  };
}
