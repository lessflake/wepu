/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    fontFamily: {
      'sans': ['Inter', 'sans-serif'],
      'serif': ['Crimson Pro', 'EB Garamond', 'Lora', 'serif'],
    },
    extend: {},
  },
  plugins: [],
}

