export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  let categories = [];

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/categories`, {
      headers: { 'X-Dev-Mode': 'true' }
    });
    if (res.ok) {
      categories = await res.json();
    }
  } catch (e) {
    console.error('Failed to load categories:', e);
  }

  return { categories };
}
