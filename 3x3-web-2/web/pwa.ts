declare global {
  interface Window {
    __DISABLE_SW__?: boolean;
  }
}

export function registerServiceWorker(swUrl = "./sw.js") {
  if (typeof window !== "undefined") {
    const params = new URLSearchParams(window.location.search);
    if (params.has("no-sw") || window.__DISABLE_SW__) {
      if ("serviceWorker" in navigator) {
        navigator.serviceWorker.getRegistrations().then((regs) => {
          regs.forEach((r) => r.unregister());
        });
      }
      return;
    }
  }

  if ("serviceWorker" in navigator) {
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
