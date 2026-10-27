export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';

  let store = {};
  let paymentProviders = [];
  let shippingProviders = [];

  const [infoRes, shippingRes] = await Promise.allSettled([
    fetch(`${backendUrl}/api/v1/store/info`),
    fetch(`${backendUrl}/api/v1/shipping/providers`)
  ]);

  try {
    if (infoRes.status === 'fulfilled' && infoRes.value.ok) {
      const data = await infoRes.value.json();
      store = data.store || {};
      paymentProviders = data.payment_providers || [];
    }
  } catch (e) {
    console.error('Failed to load checkout settings:', e);
  }

  // The destination countries offered at checkout come from the shipping zones
  try {
    if (shippingRes.status === 'fulfilled' && shippingRes.value.ok) {
      shippingProviders = (await shippingRes.value.json()) || [];
    }
  } catch (e) {
    console.error('Failed to load shipping zones:', e);
  }

  return { store, paymentProviders, shippingProviders };
}
