export async function load({ fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const headers = {};
  let settings = {};
  let paymentConfigs = [];
  let shippingProviders = [];

  try {
    const res = await fetch(`${backendUrl}/api/v1/admin/settings/system`, { headers });
    if (res.ok) {
      settings = await res.json();
    }
  } catch (e) {
    console.error('Failed to load system settings in settings overview:', e);
  }

  try {
    const payRes = await fetch(`${backendUrl}/api/v1/admin/settings/payments`, { headers });
    if (payRes.ok) {
      paymentConfigs = await payRes.json();
    }
  } catch (e) {
    console.error('Failed to load payment settings in settings overview:', e);
  }

  try {
    const shipRes = await fetch(`${backendUrl}/api/v1/admin/settings/shipping/providers`, { headers });
    if (shipRes.ok) {
      shippingProviders = await shipRes.json();
    }
  } catch (e) {
    console.error('Failed to load shipping providers in settings overview:', e);
  }

  let adminUsers = [];
  try {
    const usersRes = await fetch(`${backendUrl}/api/v1/admin/users`, { headers });
    if (usersRes.ok) {
      adminUsers = await usersRes.json();
    }
  } catch (e) {
    console.error('Failed to load admin users in settings overview:', e);
  }

  return {
    settings,
    // Saving a form built from missing data would overwrite the real settings with blanks
    settingsLoadFailed: Object.keys(settings).length === 0,
    paymentConfigs,
    shippingProviders,
    adminUsers
  };
}
