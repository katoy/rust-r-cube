export function registerServiceWorker(swUrl = "./sw.js") {
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
