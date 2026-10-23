/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        cream: '#FAF9F6',
        mist: '#F3F4F6',
        blush: '#F8E8EC',
        sage: '#A8BFA8',
        ink: '#374151'
      }
    }
  },
  plugins: []
};
