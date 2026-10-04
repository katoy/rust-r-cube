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

// A version's document and assets are committed together by addAll(), never
// replaced by responses from a deployment whose installation may have failed.
const PRECACHE_URLS = new Set(
  PRECACHE_ASSETS.map((asset) =>
    new URL(asset, self.registration.scope).toString(),
  ),
);

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
    (async () => {
      const ownCache = await caches.open(CACHE_NAME);
      const keys = await caches.keys();
      const oldKeys = keys.filter(
        (key) =>
          key.startsWith(CACHE_PREFIX) &&
          key !== CACHE_NAME &&
          /^(v\d+|old-v\d+|[0-9a-f]{8,})$/i.test(
            key.slice(CACHE_PREFIX.length),
          ),
      );

      // 直近 1 世代の旧キャッシュのみを移行対象とし、無制限な旧資産コピーによるキャッシュ肥大化（Cache Bloat）を防止
      const sortedOldKeys = [...oldKeys].sort();
      const mostRecentOldKey = sortedOldKeys.pop();

      if (mostRecentOldKey) {
        try {
          const oldCache = await caches.open(mostRecentOldKey);
          const requests = await oldCache.keys();
          let migratedCount = 0;
          const MAX_MIGRATED_ITEMS = 15;

          for (const req of requests) {
            if (migratedCount >= MAX_MIGRATED_ITEMS) break;
            const alreadyCached = await ownCache.match(req);
            if (!alreadyCached) {
              const res = await oldCache.match(req);
              if (res && res.ok) {
                await ownCache.put(req, res); // ignore-guardrail: SW cache migration preserves existing cached request keys
                migratedCount++;
              }
            }
          }
        } catch {
          // 旧キャッシュアクセスエラーは無視
        }
        await caches.delete(mostRecentOldKey);
      }

      // 過去全世代の古いキャッシュは移行せず安全に完全削除
      for (const oldKey of sortedOldKeys) {
        try {
          await caches.delete(oldKey);
        } catch {
          // 削除エラーは無視
        }
      }

      await self.clients.claim();
    })(),
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
      fetch(request).catch(async () => {
        const ownCache = await caches.open(CACHE_NAME);
        const canonicalUrl = url.origin + url.pathname;
        const cached =
          (await ownCache.match(canonicalUrl)) ||
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
        return Response.error();
      }),
    );
    return;
  }

  // 静的アセット（同一オリジンの JS, WASM, CSS, 画像, プリセット JSON 等）
  const selfOrigin = self.location
    ? self.location.origin ||
      (self.location.href ? new URL(self.location.href).origin : undefined)
    : self.registration
      ? new URL(self.registration.scope).origin
      : undefined;
  if (selfOrigin && url.origin !== selfOrigin) return;

  event.respondWith(
    (async () => {
      const ownCache = await caches.open(CACHE_NAME);
      const canonicalKey = url.origin + url.pathname;
      const cached = url.search
        ? (await ownCache.match(request)) || (await ownCache.match(url.href))
        : (await ownCache.match(request)) ||
          (await ownCache.match(url.href)) ||
          (await ownCache.match(canonicalKey)) ||
          (await ownCache.match(url.pathname));

      // プリキャッシュ済み資産でキャッシュがある場合はネットワーク不要
      if (cached && PRECACHE_URLS.has(canonicalKey)) {
        return cached;
      }

      const updatePromise = fetch(request)
        .then(async (networkResponse) => {
          if (networkResponse.ok && !PRECACHE_URLS.has(canonicalKey)) {
            const clone = networkResponse.clone();
            try {
              const cacheKey = url.search ? url.href : canonicalKey;
              await ownCache.put(cacheKey, clone); // ignore-guardrail: static asset queries (e.g. Vite ?url or ?v=) require full URL keying to prevent binary collisions
            } catch (error) {
              console.warn(
                "[Service Worker] Runtime cache write failed:",
                error,
              );
            }
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
