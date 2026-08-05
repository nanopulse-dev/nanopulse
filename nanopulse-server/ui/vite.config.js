import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
// https://vitejs.dev/config/
export default defineConfig({
    server: {
        proxy: {
            "^/api/ws": {
                target: "ws://localhost:3030",
                changeOrigin: true,
                ws: true,
            },
            "^/api.*": {
                target: "http://localhost:3030",
                changeOrigin: true,
            },
        },
    },
    base: "./",
    plugins: [react()],
});
