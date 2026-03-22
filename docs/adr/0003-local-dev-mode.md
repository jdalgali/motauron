# ADR 0003: Local Dev Mode — JSON Store + HTTP API

## Status
Accepted

## Context
The original architecture required a live Firestore connection (via `service-account.json`) to run the backend at all, and the frontend was permanently coupled to Firebase for data. This created friction for local development: cloud credentials had to be present, and any frontend changes needed Firebase to be configured.

Additionally, motoscout24.ch scraping relied on headless Chromium and was unreliable (Cloudflare blocks, long timeouts). It was removed entirely.

## Decision

**Backend:**
- Auto-detect store: if `GOOGLE_APPLICATION_CREDENTIALS` is set or `service-account.json` exists → use `FirestoreListingRepository`. Otherwise → use `JsonListingRepository` (writes `listings.json`).
- Add `--serve` mode: runs an initial scrape then starts an axum HTTP server on port 3001. The server exposes:
  - `GET /api/listings` — reads `listings.json` and returns it as JSON.
  - `POST /api/scrape` — runs the full scrape → merge → score → persist pipeline synchronously.
- Both modes (`--serve` and `--daemon`) can be combined for a local background loop + API.
- `MotorradhandelScraper` now paginates through all results instead of a single page, and auto-derives `category` from brand + model (e.g. `yamaha-tenere-700`) instead of requiring manual category configuration.

**Frontend:**
- When `VITE_API_URL` is set (`.env.local`), `useListings` fetches from the local API instead of Firebase. Firebase is not initialised at all.
- A "↻ Refresh" button appears in local mode, calling `POST /api/scrape` then re-fetching.
- `vite.config.js` proxies `/api/*` to `localhost:3001` so relative URLs work in dev.
- Richer client-side filters: text search, max price, max mileage, year range, private/dealer toggle.

## Consequences
- Zero cloud dependencies for local development — `cargo run -- --serve` is the only command needed.
- The same binary works in both local and cloud modes; the mode is detected at runtime.
- Firestore and JSON stores are both valid `ListingRepository` adapters — no domain or application code changed (ADR 0001 upheld).
- The `category` field is now auto-derived from motorradhandel brand+model data, enabling the scraper to cover all motorcycles without manual URL configuration per model.
- The frontend filter bar scales to any number of models since it no longer uses chip buttons per category.
