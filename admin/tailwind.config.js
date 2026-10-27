/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        rust: {
          500: '#f97316',
          600: '#ea580c',
        }
      }
    },
  },
  plugins: [],
}
