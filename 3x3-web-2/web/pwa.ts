export function registerServiceWorker() {
  if ("serviceWorker" in navigator) {
    const register = () => {
      navigator.serviceWorker
        .register("./sw.js", { scope: "./" })
        .catch((err) => {
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
