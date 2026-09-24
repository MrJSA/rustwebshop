export async function GET({ params, url, fetch, request }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const target = `${backendUrl}/api/v1/${params.path}${url.search}`;
  const res = await fetch(target, { headers: request.headers });
  return new Response(res.body, {
    status: res.status,
    headers: { 'Content-Type': res.headers.get('content-type') || 'application/json' }
  });
}

export async function POST({ params, url, fetch, request }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const target = `${backendUrl}/api/v1/${params.path}${url.search}`;
  const contentType = request.headers.get('content-type') || '';
  
  let body;
  if (contentType.includes('multipart/form-data') || contentType.includes('image/') || contentType.includes('octet-stream')) {
    body = await request.arrayBuffer();
  } else {
    body = await request.text();
  }

  const res = await fetch(target, {
    method: 'POST',
    headers: request.headers,
    body
  });
  return new Response(res.body, {
    status: res.status,
    headers: { 'Content-Type': res.headers.get('content-type') || 'application/json' }
  });
}

export async function PUT({ params, url, fetch, request }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const target = `${backendUrl}/api/v1/${params.path}${url.search}`;
  const body = await request.text();
  const res = await fetch(target, {
    method: 'PUT',
    headers: request.headers,
    body
  });
  return new Response(res.body, {
    status: res.status,
    headers: { 'Content-Type': res.headers.get('content-type') || 'application/json' }
  });
}

export async function DELETE({ params, url, fetch, request }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const target = `${backendUrl}/api/v1/${params.path}${url.search}`;
  const res = await fetch(target, {
    method: 'DELETE',
    headers: request.headers
  });
  return new Response(res.body, {
    status: res.status,
    headers: { 'Content-Type': res.headers.get('content-type') || 'application/json' }
  });
}
