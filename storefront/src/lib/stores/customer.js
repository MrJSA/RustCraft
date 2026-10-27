import { writable } from 'svelte/store';
import { browser } from '$app/environment';

function createCustomerStore() {
  const initial = browser
    ? JSON.parse(localStorage.getItem('rustwebshop_customer') || 'null')
    : null;

  const { subscribe, set, update } = writable(initial);

  return {
    subscribe,
    login: (token, email, full_name) => {
      const data = { token, email, full_name, isLoggedIn: true };
      if (browser) {
        localStorage.setItem('rustwebshop_customer', JSON.stringify(data));
      }
      set(data);
    },
    logout: () => {
      if (browser) {
        localStorage.removeItem('rustwebshop_customer');
      }
      set(null);
    }
  };
}

export const customer = createCustomerStore();
