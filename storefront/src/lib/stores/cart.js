import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';

function createCartStore() {
  const initial = browser ? JSON.parse(localStorage.getItem('rustwebshop_cart') || '[]') : [];
  const { subscribe, set, update } = writable(initial);

  if (browser) {
    subscribe((items) => {
      localStorage.setItem('rustwebshop_cart', JSON.stringify(items));
    });
  }

  return {
    subscribe,
    addItem: (item) => {
      update((items) => {
        const existingIndex = items.findIndex((i) => i.variant_id === item.variant_id);
        if (existingIndex > -1) {
          const updated = [...items];
          updated[existingIndex].quantity += (item.quantity || 1);
          return updated;
        }
        return [...items, { ...item, quantity: item.quantity || 1 }];
      });
    },
    updateQuantity: (variant_id, quantity) => {
      update((items) => {
        if (quantity <= 0) {
          return items.filter((i) => i.variant_id !== variant_id);
        }
        return items.map((i) => (i.variant_id === variant_id ? { ...i, quantity } : i));
      });
    },
    removeItem: (variant_id) => {
      update((items) => items.filter((i) => i.variant_id !== variant_id));
    },
    clear: () => set([])
  };
}

export const cart = createCartStore();

export const cartCount = derived(cart, ($cart) =>
  $cart.reduce((sum, item) => sum + item.quantity, 0)
);

export const cartSubtotal = derived(cart, ($cart) =>
  $cart.reduce((sum, item) => sum + (item.price_cents * item.quantity), 0)
);

export const isCartOpen = writable(false);
