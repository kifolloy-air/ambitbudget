// Ambit Kids — service worker
// Bump CACHE_VERSION whenever you ship app changes to force refresh.
const CACHE_VERSION = "ambit-kids-v1";
const CACHE_NAME = CACHE_VERSION;

const APP_SHELL = [
  "./app.html",
  "./manifest.json",
  "./images/full-body.png",
  "./images/head.png",
  "./images/idea.png",
  "./images/pointing.png",
  "./images/thinking.png",
  "./images/thumbs-up.png"
];

self.addEventListener("install", event => {
  event.waitUntil(
    caches.open(CACHE_NAME).then(cache => cache.addAll(APP_SHELL))
      .then(() => self.skipWaiting())
  );
});

self.addEventListener("activate", event => {
  event.waitUntil(
    caches.keys().then(keys =>
      Promise.all(keys.filter(k => k !== CACHE_NAME).map(k => caches.delete(k)))
    ).then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", event => {
  const req = event.request;
  if (req.method !== "GET") return;

  // Network-first for the HTML so updates land; cache fallback for offline
  if (req.mode === "navigate" || req.destination === "document") {
    event.respondWith(
      fetch(req).then(resp => {
        const copy = resp.clone();
        caches.open(CACHE_NAME).then(c => c.put(req, copy)).catch(()=>{});
        return resp;
      }).catch(() => caches.match(req).then(r => r || caches.match("./app.html")))
    );
    return;
  }

  // Cache-first for static assets
  event.respondWith(
    caches.match(req).then(cached =>
      cached || fetch(req).then(resp => {
        if (resp && resp.status === 200 && resp.type === "basic") {
          const copy = resp.clone();
          caches.open(CACHE_NAME).then(c => c.put(req, copy)).catch(()=>{});
        }
        return resp;
      }).catch(() => cached)
    )
  );
});
