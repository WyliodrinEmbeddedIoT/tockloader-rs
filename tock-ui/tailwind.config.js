/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Poppins', 'sans-serif'],
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [
    require('daisyui'),
    ],
    
  daisyui: {
  themes: [
    {
      Tockloader: {
        "primary": "#C2C2C2",
        "primary-content": "#4C4C4C",
        "secondary": "#4C4C4C",
        "secondary-content": "#C2C2C2",
        "accent": "#000000",
        "accent-content": "#C2C2C2",
        "neutral": "#C2C2C2",
        "neutral-content": "#383838",
        "base-100": "#383838",
        "base-200": "#4C4C4C",
        "base-300": "#2b2b2b",
        "base-content": "#C2C2C2",
        "info": "#3abff8",
        "info-content": "#1a1a1a",
        "success": "#1E453E",
        "success-content": "#C2C2C2",
        "warning": "#fbbd23",
        "warning-content": "#1a1a1a",
        "error": "#6F0000",
        "error-content": "#C2C2C2",
      },
    },
  ],
  darkTheme: "Tockloader",
},  
}
