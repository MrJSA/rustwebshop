export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = { 'X-Dev-Mode': 'true' };

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/settings/shipping`, { headers });
    if (res.ok) {
      const shippingZones = await res.json();
      return { shippingZones };
    }
  } catch (e) {
    console.error('Failed to load shipping zones:', e);
  }

  return { shippingZones: [] };
}
