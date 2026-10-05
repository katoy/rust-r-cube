import { defineConfig } from "vite";

export default defineConfig({
  base: "./",
  worker: { format: "es" },
  build: {
    target: "es2022",
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules/three")) {
            return "three";
          }
        },
      },
    },
  },
});
