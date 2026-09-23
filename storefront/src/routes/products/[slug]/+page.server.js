import { error } from '@sveltejs/kit';

export async function load({ params, fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const { slug } = params;

  try {
    const res = await fetch(`${backendUrl}/api/v1/products/${slug}`);
    if (res.ok) {
      const data = await res.json();
      let parts = [];
      try {
        const partsRes = await fetch(`${backendUrl}/api/v1/products/${slug}/parts`);
        if (partsRes.ok) {
          parts = await partsRes.json();
        }
      } catch (e) {
        console.error('Failed to load parts:', e);
      }

      return {
        product: data.product || data,
        variants: data.variants || [],
        parts
      };
    }
  } catch (e) {
    console.error(`Failed to load product ${slug}:`, e);
  }

  throw error(404, 'Product not found');
}

