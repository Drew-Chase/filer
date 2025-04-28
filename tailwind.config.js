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
    darkMode: "class",
    plugins: [heroui({
        themes: {
            light: {
                colors: {
                    primary: {
                        DEFAULT: "#ff32cf",
                        foreground: "#fff",
                    },
                    secondary: "#151515",
                    background: "#d4cbf1",
                }
            },
            dark: {
                colors: {
                    primary: "#ff32cf",
                    secondary: "#3b82f6",
                    background: "#0c061e",
                }
            },
        }
    })]
}