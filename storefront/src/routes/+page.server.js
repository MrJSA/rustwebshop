export async function load({ fetch, url }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const category = url.searchParams.get('category') || '';
  const subcategory = url.searchParams.get('subcategory') || '';
  const search = url.searchParams.get('search') || '';

  let queryUrl = `${backendUrl}/api/v1/products?`;
  if (category) queryUrl += `category=${encodeURIComponent(category)}&`;
  if (subcategory) queryUrl += `subcategory=${encodeURIComponent(subcategory)}&`;
  if (search) queryUrl += `search=${encodeURIComponent(search)}&`;

  try {
    const res = await fetch(queryUrl);
    if (res.ok) {
      const products = await res.json();
      return {
        products,
        currentCategory: category,
        currentSearch: search
      };
    }
  } catch (e) {
    console.error('Failed to fetch products from backend:', e);
  }

  return {
    products: [],
    currentCategory: category,
    currentSearch: search
  };
}
