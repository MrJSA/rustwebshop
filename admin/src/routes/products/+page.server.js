export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = { 'X-Dev-Mode': 'true' };

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/products`, { headers });
    if (res.ok) {
      const products = await res.json();
      return { products };
    }
  } catch (e) {
    console.error('Failed to load products in admin:', e);
  }

  return { products: [] };
}
