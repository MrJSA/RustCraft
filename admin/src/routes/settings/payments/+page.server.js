export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = {};

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/settings/payments`, { headers });
    if (res.ok) {
      const paymentConfigs = await res.json();
      return { paymentConfigs };
    }
  } catch (e) {
    console.error('Failed to load payment settings:', e);
  }

  return { paymentConfigs: [] };
}
