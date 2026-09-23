export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/menu`, {
      headers: { 'X-Dev-Mode': 'true' }
    });
    if (res.ok) {
      const items = await res.json();
      return { menuItems: items };
    }
  } catch (e) {
    console.error('Failed to load menu items:', e);
  }

  return { menuItems: [] };
}
