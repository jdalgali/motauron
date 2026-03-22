# motauron

Watches Swiss motorcycle marketplaces. Tracks when listings appear, drop in price, go sold, or get relisted under a new ad — same bike, new ID. Scores each listing against its peers so you know what's actually a deal.

## What it does

- Scrapes listing data from **motorradhandel.ch** (JSON extraction) and **motoscout24.ch** (headless browser)
- Persists a historical database in **Firestore** across runs
- Detects price changes, sold listings, and relists (same physical bike, new ad ID)
- Scores each listing relative to year and model peers, adjusting for mileage, canton, and seller type
- Sends push notifications via **ntfy.sh** for new deals and price drops
- Exposes a live dashboard at Firebase Hosting backed by real-time Firestore reads

## Architecture

```
motauron/
├── backend/     Rust scraper — hexagonal architecture (domain / application / infrastructure)
├── frontend/    React + Vite SPA — feature-driven, reads from Firestore in real time
└── docs/adr/    Architecture Decision Records
```

The backend is designed to run as a **Cloud Run Job** triggered by **Cloud Scheduler** every 4 hours. The frontend is deployed to **Firebase Hosting**.

## Local development

### Backend

Requires Rust and a `service-account.json` Firestore key in `backend/`.

```
cd backend
cp config.toml.example config.toml   # edit with your ntfy URL
cargo run
```

For continuous mode (skips Cloud Scheduler — useful for local testing):

```
cargo run -- --daemon
```

### Frontend

```
cd frontend
cp .env.local.example .env.local     # add your Firebase API key
npm install
npm run dev
```

## Deployment

### Backend — Cloud Run Job

Build and push the Docker image:

```
cd backend
docker build -t gcr.io/motauron-ch/motauron .
docker push gcr.io/motauron-ch/motauron
```

Create the Cloud Run Job (one-time):

```
gcloud run jobs create motauron \
  --image gcr.io/motauron-ch/motauron \
  --region europe-west6 \
  --service-account motauron-runner@motauron-ch.iam.gserviceaccount.com \
  --set-env-vars NTFY_URL=https://ntfy.sh/your-private-topic
```

The service account needs `roles/datastore.user` on the project. Because the job runs with workload identity, no key file is mounted — `GOOGLE_APPLICATION_CREDENTIALS` is handled automatically.

Schedule via Cloud Scheduler:

```
gcloud scheduler jobs create http motauron-4h \
  --schedule "0 */4 * * *" \
  --uri "https://REGION-run.googleapis.com/apis/run.googleapis.com/v1/namespaces/motauron-ch/jobs/motauron:run" \
  --oauth-service-account-email motauron-runner@motauron-ch.iam.gserviceaccount.com \
  --location europe-west6
```

Both Cloud Run Jobs and Cloud Scheduler are free within normal usage bounds (6 runs/day × 30 days = 180 executions/month; free tier covers 240,000 vCPU-seconds).

### Frontend — Firebase Hosting

```
cd frontend
npm run build
firebase deploy
```

## Configuration

The backend reads `config.toml` on startup, with **environment variables taking priority**:

| Env var                         | Config equivalent         | Purpose                                    |
|---------------------------------|---------------------------|--------------------------------------------|
| `NTFY_URL`                      | `notify.ntfy.url`         | ntfy.sh topic URL for push notifications   |
| `NTFY_TOKEN`                    | `notify.ntfy.token`       | ntfy auth token (self-hosted only)         |
| `GOOGLE_APPLICATION_CREDENTIALS`| —                         | Path to GCP service account key (local only) |
| `CHROME_PATH`                   | —                         | Chromium binary path (set in Docker image) |

On Cloud Run, `GOOGLE_APPLICATION_CREDENTIALS` is not required — workload identity is used automatically.

## Adding models to track

Edit the category lists in `backend/src/main.rs`:

```rust
let mh_categories: &[(&str, &str)] = &[
    ("tenere-700", "https://motorradhandel.ch/en/..."),
    ("africa-twin", "https://motorradhandel.ch/en/..."),
];

let ms_categories: &[(&str, &str)] = &[
    ("tenere-700", "https://www.motoscout24.ch/de/s/..."),
    ("africa-twin", "https://www.motoscout24.ch/de/s/..."),
];
```

The category key is the stable ID in Firestore — keep it consistent once set. Get URLs directly from each marketplace's search page.

## Price scoring

Each listing is scored against its year and model peers (e.g. all 2024 Tenere 700s in the current database). The score is a percentage showing how much cheaper or more expensive the listing is relative to what you'd expect given:

- **Mileage** — adjusted at ~0.7 CHF/km relative to the group median
- **Canton** — regional price multipliers (Zürich +8%, Valais −6%, etc.)
- **Seller type** — dealers are expected to charge ~7% more than private sellers for the same bike

Score labels:

| Label        | Meaning                                  |
|--------------|------------------------------------------|
| `great deal` | >15% cheaper than expected               |
| `good`       | 7–15% cheaper                            |
| `fair`       | within ±6% of expected                   |
| `overpriced` | 7–15% more expensive                     |
| `expensive`  | >15% more expensive                      |
| `n/a`        | only one listing in peer group           |

The `vs N` number shows how many listings the score is based on — a score from 2 peers is weaker than one from 6.

Where enough peers exist, World Raid and base Tenere 700 variants are scored separately; otherwise they fall back to the year-group comparison.

## Firestore data model

One document per listing ever seen, keyed by `listing_id`.

| Field                | Type     | Description                                              |
|----------------------|----------|----------------------------------------------------------|
| `listing_id`         | u64      | Marketplace ad ID — stable primary key                   |
| `fingerprint`        | u64      | Content hash used for relist detection                   |
| `status`             | string   | `active` / `sold` / `relisted`                           |
| `first_seen`         | date     | Date first scraped                                       |
| `last_seen`          | date     | Date last seen active                                    |
| `original_price_chf` | u32      | Price when first seen — never overwritten                |
| `price_chf`          | u32      | Current price — updated each run                         |
| `is_private`         | bool     | True if private seller, false if dealer                  |
| `seller_name`        | string   | Garage or dealer name, empty for private sellers         |
| `location`           | string   | City                                                     |
| `kanton`             | string   | Canton code (ZH, BE, VS, …)                             |
| `price_score`        | i32      | % cheaper (+) or pricier (−) than expected               |
| `price_label`        | string   | Human-readable score label                               |
| `score_peers`        | u32      | Number of listings the score is based on                 |
| `previous_listing_id`| u64?     | Original ad ID if this is a detected relist              |

## Notes

- Only listings with a price are tracked
- Mileage fingerprinting uses 100 km steps below 1000 km and 1000 km steps above to absorb small discrepancies between relists
- Two simultaneously active listings with identical model/year/mileage share a fingerprint — ambiguous cases are not flagged as relists
- motoscout24.ch is scraped via headless Chromium to handle the Cloudflare JS challenge
