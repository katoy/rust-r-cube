function getScopeSlug() {
  try {
    let pathname = "/";
    if (self.registration && self.registration.scope) {
      pathname = new URL(self.registration.scope).pathname;
    } else if (self.location && self.location.href) {
      const u = new URL(self.location.href);
      pathname = u.pathname.substring(0, u.pathname.lastIndexOf("/") + 1);
    }
    const rawSegments = pathname
      .replace(/^\/+|\/+$/g, "")
      .split("/")
      .filter(Boolean);

    if (rawSegments.length === 0) {
      return "root";
    }

    // 記号を一意にエスケープ（_ は _u_, - は _h_, . は _d_, その他は _xHH_）
    const escapeSegment = (seg) =>
      seg.replace(/[^a-zA-Z0-9]/g, (ch) => {
        if (ch === "_") return "_u_";
        if (ch === "-") return "_h_";
        if (ch === ".") return "_d_";
        return `_x${ch.charCodeAt(0).toString(16)}_`;
      });

    // /root/ は "root_" とし、ルートスコープ "/" ("root") との衝突・前方一致を防止
    if (rawSegments.length === 1 && rawSegments[0] === "root") {
      return "root_";
    }

    const segments = rawSegments.map(escapeSegment);

    // /root/... 配下は "root_" で開始してルートスコープ "root-" との前方一致巻き込みを防止
    if (segments[0] === "root") {
      return "root_" + segments.slice(1).join("-");
    }

    return segments.join("-");
  } catch {
    return "root";
  }
}

const SCOPE_SLUG = getScopeSlug();
const CACHE_PREFIX = `cube-studio-${SCOPE_SLUG}-`;
const CACHE_VERSION = "v1";
const CACHE_NAME = `${CACHE_PREFIX}${CACHE_VERSION}`;

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
            .filter(
              (key) =>
                key.startsWith(CACHE_PREFIX) &&
                key !== CACHE_NAME &&
                /^(v\d+|old-v\d+|[0-9a-f]{8,})$/i.test(
                  key.slice(CACHE_PREFIX.length),
                ),
            )
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
            const updatePromise = caches
              .open(CACHE_NAME)
              .then((cache) => cache.put(request, clone));
            event.waitUntil(updatePromise);
          }
          return response;
        })
        .catch(async () => {
          const ownCache = await caches.open(CACHE_NAME);
          const cached =
            (await ownCache.match(request)) ||
            (await ownCache.match(request.url)) ||
            (await ownCache.match(request, { ignoreSearch: true }));
          if (cached) return cached;
          const scope = self.registration ? self.registration.scope : "./";
          const indexUrl = new URL("./index.html", scope).toString();
          const scopeUrl = new URL("./", scope).toString();
          const rootCached =
            (await ownCache.match(indexUrl)) ||
            (await ownCache.match(scopeUrl)) ||
            (await ownCache.match(scope)) ||
            (await ownCache.match("./index.html")) ||
            (await ownCache.match("./"));
          if (rootCached) return rootCached;
          return ownCache.match("/");
        }),
    );
    return;
  }

  // 静的アセット（JS, WASM, CSS, 画像, プリセット JSON 等）
  event.respondWith(
    (async () => {
      const ownCache = await caches.open(CACHE_NAME);
      const cached =
        (await ownCache.match(request)) ||
        (await ownCache.match(request.url)) ||
        (await ownCache.match(url.pathname));

      const updatePromise = fetch(request)
        .then(async (networkResponse) => {
          if (networkResponse.ok) {
            const clone = networkResponse.clone();
            await ownCache.put(request, clone);
          }
          return networkResponse;
        })
        .catch(() => cached || Response.error());

      // バックグラウンドキャッシュ更新を event.waitUntil に接続して SW 早期終了を防止
      event.waitUntil(updatePromise);

      return cached || updatePromise;
    })(),
  );
});
