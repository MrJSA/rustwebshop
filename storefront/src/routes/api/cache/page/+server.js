import { json } from '@sveltejs/kit';
import { pageCache } from '$lib/server/pageCache.js';

export async function POST({ request }) {
  try {
    const data = await request.json();
    const { slug, title, content_markdown, is_published } = data;

    if (!slug) {
      return json({ error: 'Missing slug' }, { status: 400 });
    }

    const cached = pageCache.get(slug) || {};
    pageCache.set(slug, {
      page: {
        slug,
        title: title || cached.page?.title || slug,
        content_markdown: content_markdown !== undefined ? content_markdown : cached.page?.content_markdown || '',
        is_published: is_published !== undefined ? is_published : true,
        updated_at: new Date().toISOString()
      },
      providers: cached.providers || []
    });

    return json({ success: true, cached_slug: slug });
  } catch (e) {
    return json({ error: e.message }, { status: 500 });
  }
}
