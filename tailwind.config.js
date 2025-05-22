import {heroui} from "@heroui/react";

/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
        "./node_modules/@heroui/theme/dist/**/*.{js,ts,jsx,tsx}"
    ],
    theme: {
        extend: {},
    },
    fontFamily: {
        sans: ['Inter', 'sans-serif'],
    },
    darkMode: "class",
    plugins: [heroui({
        themes: {
            dark: {
                colors: {
                    primary: {
                        DEFAULT: "#ff32cf",
                        light: "#ff65dd",
                        dark: "#d600a9",
                        foreground: "#fff",
                    },
                    secondary: {
                        DEFAULT: "#fff",
                        foreground: "#000",
                    },
                    background: "#0c061e",
                }
            },
        }
    })]
}