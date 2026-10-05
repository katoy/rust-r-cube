# Offline verification

`npm run offline -- --headless --fresh` rebuilds before checking offline boot.
`--build` is an alias for `--fresh`; without either option, the build-input
manifest determines whether a rebuild is needed. Omit `--headless` to keep the
offline browser open. `--port=4187` selects a port other than the default 4173.

Before launching Chromium, the verifier checks the SHA-256 content of every
served build file against the local `dist` directory. A reused server must serve
that exact build; an unrelated or stale server fails verification rather than
producing an apparent success. Reused servers are never stopped by the verifier.
Only the preview process started by this invocation is stopped during cleanup.

Service Worker activation and controller acquisition have a 15-second deadline.
A failed installation or unavailable controller exits nonzero and closes owned
browser/server resources.

The Service Worker returns network documents while online, but keeps each
version's precached document and assets immutable. Only a successful complete
precache installation replaces the offline version. Runtime-only assets can
revalidate in the background; a failed optional cache write is diagnosed without
discarding a successful network response.

Run the isolated regressions without rebuilding or starting shared app servers:

```sh
npx playwright test --config tests/infra-offline.config.ts
```
