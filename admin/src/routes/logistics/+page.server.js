export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = { 'X-Dev-Mode': 'true' };

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/logistics/inventory`, { headers });
    if (res.ok) {
      const inventory = await res.json();
      return { inventory };
    }
  } catch (e) {
    console.error('Failed to load logistics inventory:', e);
  }

  return { inventory: [] };
}
