const CACHE_PREFIX = "cube-studio-";
const CACHE_NAME = `${CACHE_PREFIX}v1`;

const PRECACHE_ASSETS = [
  "./",
  "./index.html",
  "./manifest.webmanifest",
  "./icon.svg",
  "./icon-192.png",
  "./icon-512.png",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE_NAME)
      .then((cache) => cache.addAll(PRECACHE_ASSETS))
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key.startsWith(CACHE_PREFIX) && key !== CACHE_NAME)
            .map((key) => caches.delete(key)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const { request } = event;

  // GET リクエストのみキャッシュ対象
  if (request.method !== "GET") return;

  const url = new URL(request.url);

  // http / https スキームのみ
  if (!url.protocol.startsWith("http")) return;

  // ナビゲーションリクエスト（HTML ドキュメント）
  if (request.mode === "navigate") {
    event.respondWith(
      fetch(request)
        .then((response) => {
          if (response.ok) {
            const clone = response.clone();
            caches.open(CACHE_NAME).then((cache) => cache.put(request, clone));
          }
          return response;
        })
        .catch(async () => {
          const cached = await caches.match(request);
          if (cached) return cached;
          const scope = self.registration.scope;
          const indexUrl = new URL("./index.html", scope).toString();
          const rootCached =
            (await caches.match(indexUrl)) ||
            (await caches.match(scope)) ||
            (await caches.match("./index.html")) ||
            (await caches.match("./"));
          if (rootCached) return rootCached;
          return caches.match("/");
        }),
    );
    return;
  }

  // 静的アセット（JS, WASM, CSS, 画像, プリセット JSON 等）
  event.respondWith(
    (async () => {
      const cached =
        (await caches.match(request)) ||
        (await caches.match(request.url)) ||
        (await caches.match(url.pathname));
      if (cached) return cached;

      try {
        const networkResponse = await fetch(request);
        if (networkResponse.ok) {
          const clone = networkResponse.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(request, clone));
        }
        return networkResponse;
      } catch (err) {
        return cached;
      }
    })(),
  );
});
