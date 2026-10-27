import { error } from '@sveltejs/kit';

export async function load({ params, fetch }) {
  const backendUrl = process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';
  const { slug } = params;

  try {
    const res = await fetch(`${backendUrl}/api/v1/products/${slug}`);
    if (res.ok) {
      const data = await res.json();
      const product = data.product || data;
      const variants = data.variants || [];

      let relatedProducts = [];
      if (product && product.id) {
        try {
          const relRes = await fetch(`${backendUrl}/api/v1/products/${product.id}/related`);
          if (relRes.ok) {
            relatedProducts = await relRes.json();
          }
        } catch (e) {
          console.error('Failed to load related products:', e);
        }
      }

      return {
        product,
        variants,
        relatedProducts
      };
    }
  } catch (e) {
    console.error(`Failed to load product ${slug}:`, e);
  }

  throw error(404, 'Product not found');
}

