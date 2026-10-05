export function GET({ url }) {
  const body = [
    'User-agent: *',
    'Allow: /',
    'Disallow: /account',
    'Disallow: /checkout',
    'Disallow: /order-success',
    'Disallow: /track',
    'Disallow: /api/',
    'Disallow: /*?search=',
    '',
    `Sitemap: ${url.origin}/sitemap.xml`,
    ''
  ].join('\n');

  return new Response(body, {
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      'Cache-Control': 'public, max-age=86400'
    }
  });
}
