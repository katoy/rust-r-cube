declare global {
  interface Window {
    __DISABLE_SW__?: boolean;
  }
}

export function registerServiceWorker(swUrl = "./sw.js") {
  if (typeof window !== "undefined" && "serviceWorker" in navigator) {
    const params = new URLSearchParams(window.location.search);
    const isDev = Boolean(import.meta.env?.DEV);
    const isAutomatedTest = Boolean(navigator.webdriver);
    const forceSw = params.has("force-sw");

    // 開発環境 (Vite dev) では、自動テスト中または ?force-sw が明示されない限り
    // ローカルキャッシュ汚染と HMR 阻害を防ぐため登録をスキップし既存 SW を解除
    const isDevSuppressed = isDev && !isAutomatedTest && !forceSw;
    const isExplicitlyDisabled = params.has("no-sw") || window.__DISABLE_SW__;

    if (isDevSuppressed || isExplicitlyDisabled) {
      const targetScope = new URL("./", window.location.href).href;
      const targetScript = new URL(swUrl, window.location.href).href;
      navigator.serviceWorker.getRegistrations().then((regs) => {
        regs.forEach((r) => {
          const scriptUrl = (r.active || r.waiting || r.installing)?.scriptURL;
          const matchesScope = r.scope === targetScope;
          const matchesScript = !scriptUrl || scriptUrl === targetScript;
          if (matchesScope && matchesScript) {
            void r.unregister();
          }
        });
      });
      return;
    }

    const register = () => {
      navigator.serviceWorker.register(swUrl, { scope: "./" }).catch((err) => {
        console.warn("ServiceWorker registration failed:", err);
      });
    };
    if (document.readyState === "complete") {
      register();
    } else {
      window.addEventListener("load", register);
    }
  }
}
