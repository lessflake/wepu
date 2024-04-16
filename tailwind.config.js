/** @type {import('tailwindcss').Config} */
const colors = require('tailwindcss/colors')

module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    fontFamily: {
      'sans': ['Inter', 'sans-serif'],
      'serif': ['Crimson Pro', 'serif'],
    },
    screens: {
      'sm': '480px',
      'md': '720px',
      'lg': '1024px',
      'xl': '1280px',
    },
    colors: {
      transparent: 'transparent',
      current: 'currentColor',
      zinc: colors.zinc,
      sky: colors.sky,
      'sepia': {
        light: '#F2E2C9',
        dark: '#34281C',
      },
    },
    extend: {},
  },
  plugins: [
    require('@tailwindcss/forms'),
  ],
}

