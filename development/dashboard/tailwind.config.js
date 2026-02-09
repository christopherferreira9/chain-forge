/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        solana: {
          light: '#9945FF',
          DEFAULT: '#7C3AED',
          dark: '#5B21B6',
        },
        bitcoin: {
          light: '#F7931A',
          DEFAULT: '#F59E0B',
          dark: '#D97706',
        },
      },
    },
  },
  plugins: [],
}
