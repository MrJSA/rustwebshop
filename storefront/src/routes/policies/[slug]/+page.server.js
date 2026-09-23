import { error } from '@sveltejs/kit';
import { pageCache } from '$lib/server/pageCache.js';

export async function load({ params, fetch, setHeaders }) {
  const { slug } = params;
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';

  // Cache-Control headers for lightning-fast edge & browser caching
  setHeaders({
    'Cache-Control': 'public, max-age=3600, stale-while-revalidate=86400'
  });

  // Check in-memory server cache first (0ms load)
  if (pageCache.has(slug)) {
    const cached = pageCache.get(slug);
    if (cached && cached.page) {
      return {
        page: cached.page,
        providers: cached.providers || []
      };
    }
  }

  try {
    const res = await fetch(`${backendUrl}/api/v1/pages/${slug}`);
    if (res.ok) {
      const data = await res.json();
      const payload = {
        page: data.page,
        providers: data.providers || []
      };
      // Populate memory cache
      pageCache.set(slug, payload);
      return payload;
    }
  } catch (e) {
    console.error(`Failed to fetch policy page ${slug}:`, e);
  }

  throw error(404, 'Policy page not found');
}
