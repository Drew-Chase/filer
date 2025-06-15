import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
    plugins: [react({
        // Include specific file patterns for React refresh
        include: "**/*.{jsx,tsx}",
        // Enable babel for better hot reload support
        babel: {
            plugins: [],
        }
    })],
    css: {
        postcss: './postcss.config.js'
    },
    esbuild: {
        legalComments: "none",
        supported: {
            "top-level-await": true
        }
    },
    clearScreen: false,
    server: {
        host: true,
        port: 9977,
        strictPort: true,
        hmr: {
            protocol: "ws",
            host: "localhost",
            port: 9977,
            clientPort: 9977,
            overlay: true
        },
        watch: {
            ignored: ["**/src-*/**"]
        }
    },
    build: {
        outDir: "target/wwwroot"
    },
    // Optimize dependencies to avoid context reload issues
    optimizeDeps: {
        include: ['react', 'react-dom'],
        force: true
    }
});