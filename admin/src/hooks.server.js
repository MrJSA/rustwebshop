import { redirect } from '@sveltejs/kit';
import { SESSION_COOKIE, backendUrl, fetchSessionAdmin } from '$lib/server/session.js';

// Reachable without a session: the login page and the login endpoint itself.
const PUBLIC_PATHS = new Set(['/login', '/api/v1/admin/auth/login']);

const SECURITY_HEADERS = {
  'X-Frame-Options': 'DENY',
  'X-Content-Type-Options': 'nosniff',
  'Referrer-Policy': 'same-origin',
  'Permissions-Policy': 'camera=(), microphone=(), geolocation=(), payment=()',
  'Cross-Origin-Opener-Policy': 'same-origin'
};

export async function handle({ event, resolve }) {
  const token = event.cookies.get(SESSION_COOKIE) || null;
  event.locals.adminToken = token;
  event.locals.admin = null;

  const path = event.url.pathname;
  if (!PUBLIC_PATHS.has(path)) {
    const admin = await fetchSessionAdmin(token);
    if (!admin) {
      if (path.startsWith('/api/') || path.startsWith('/uploads/')) {
        return new Response(JSON.stringify({ error: 'Please log in', code: 'unauthenticated' }), {
          status: 401,
          headers: { 'Content-Type': 'application/json' }
        });
      }
      redirect(303, `/login?next=${encodeURIComponent(path + event.url.search)}`);
    }
    event.locals.admin = admin;
  }

  const response = await resolve(event);
  for (const [name, value] of Object.entries(SECURITY_HEADERS)) {
    response.headers.set(name, value);
  }
  // Admin pages contain customer data: never let shared caches store them
  if (!path.startsWith('/_app/')) response.headers.set('Cache-Control', 'no-store');
  return response;
}

/**
 * Server-side `fetch` calls from load functions to the backend automatically carry the
 * logged-in admin's token; the legacy development bypass header is always removed.
 */
export async function handleFetch({ event, request, fetch }) {
  if (request.url.startsWith(backendUrl())) {
    const headers = new Headers(request.headers);
    headers.delete('x-dev-mode');
    headers.delete('cookie');
    if (event.locals.adminToken) headers.set('Authorization', `Bearer ${event.locals.adminToken}`);
    else headers.delete('authorization');
    request = new Request(request, { headers });
  }
  return fetch(request);
}
