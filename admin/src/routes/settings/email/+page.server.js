export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  let settings = {};

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/settings/system`, {
      headers: {}
    });
    if (res.ok) {
      settings = await res.json();
    }
  } catch (e) {
    console.error('Failed to load settings in email admin:', e);
  }

  return { settings };
}
