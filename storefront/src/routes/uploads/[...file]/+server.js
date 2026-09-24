export async function GET({ params, fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const filePath = params.file;
  
  try {
    const response = await fetch(`${backendUrl}/uploads/${filePath}`);
    if (!response.ok) {
      return new Response('Not Found', { status: response.status });
    }

    const contentType = response.headers.get('content-type') || 'application/octet-stream';
    const cacheControl = response.headers.get('cache-control') || 'public, max-age=86400';

    return new Response(response.body, {
      status: 200,
      headers: {
        'content-type': contentType,
        'cache-control': cacheControl
      }
    });
  } catch (err) {
    return new Response('Internal Server Error', { status: 500 });
  }
}
