import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
    plugins: [react()],
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
    }
});