/** @type {import('tailwindcss').Config} */
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
    extend: {},
  },
  plugins: [
    require('@tailwindcss/forms'),
  ],
}

