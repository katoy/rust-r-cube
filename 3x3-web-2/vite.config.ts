import { defineConfig, type Plugin } from "vite";

const PRODUCTION_CSP = [
  "default-src 'self'",
  "script-src 'self' 'wasm-unsafe-eval'",
  "style-src 'self'",
  "img-src 'self' data: blob:",
  "worker-src 'self'",
  "connect-src 'self'",
  "media-src 'self'",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'none'",
].join("; ");

/**
 * 本番ビルド時のみ厳格な Content-Security-Policy メタタグを注入する。
 * Vite dev サーバーの HMR インラインタグや自動テスト用ヘルパーとは干渉しない。
 * frame-ancestors はメタタグでは効かないため、配信サーバー側の HTTP ヘッダで別途設定すること。
 */
function productionCspPlugin(): Plugin {
  return {
    name: "production-csp",
    apply: "build",
    transformIndexHtml() {
      return [
        {
          tag: "meta",
          attrs: {
            "http-equiv": "Content-Security-Policy",
            content: PRODUCTION_CSP,
          },
          injectTo: "head",
        },
      ];
    },
  };
}

export default defineConfig({
  base: "./",
  plugins: [productionCspPlugin()],
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
