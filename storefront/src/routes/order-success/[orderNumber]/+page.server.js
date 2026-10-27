export async function load({ params, url, fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const { orderNumber } = params;
  // Secret handed out at checkout; without it (or the email) the order stays private
  const token = url.searchParams.get('token') || '';

  if (token) {
    try {
      const res = await fetch(
        `${backendUrl}/api/v1/orders/lookup/${encodeURIComponent(orderNumber)}?token=${encodeURIComponent(token)}`
      );
      if (res.ok) {
        const data = await res.json();
        return { order: data.order, items: data.items || [], verified: true };
      }
    } catch (e) {
      console.error(`Failed to load order ${orderNumber}:`, e);
    }
  }

  // No proof of ownership: show only the order number, never customer data
  return {
    order: { order_number: orderNumber, customer_name: '', payment_status: '', order_status: 'processing' },
    items: [],
    verified: false
  };
}
