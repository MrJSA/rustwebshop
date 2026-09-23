export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = { 'X-Dev-Mode': 'true' };
  let products = [];
  let categories = [];

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/products`, { headers });
    if (res.ok) {
      products = await res.json();
    }
  } catch (e) {
    console.error('Failed to load products in admin:', e);
  }

  try {
    const catRes = await fetch(`${backendUrl}/api/v1/admin/categories`, { headers });
    if (catRes.ok) {
      categories = await catRes.json();
    }
  } catch (e) {
    console.error('Failed to load categories in admin products:', e);
  }

  return { products, categories };
}
