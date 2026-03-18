# motauron

watches swiss motorcycle marketplaces. tracks when listings appear, drop in price, go sold, or get relisted under a new ad — same bike, new id. scores each listing against its peers so you know what's actually a deal.

## what it does

- scrapes listing data from motorradhandel.ch (price, mileage, year, model, location, seller)
- maintains a persistent csv database across runs
- detects price changes between runs and flags drops and rises
- marks listings as sold when they disappear from a tracked category
- detects relists: same physical bike reposted under a new ad id
- scores each listing relative to year and model peers, adjusting for mileage, canton, and seller type (private vs dealer)

## usage

```
cargo run
```

results are saved to `listings_db.csv` in the project root. run it on a schedule (cron, task scheduler) to build up history over time.

## output

each run prints a summary:

```
openclaw — motorcycle market tracker

  tenere 700 — 10 listings found

  new           1
    + yamaha tenere 700 · 4000km (~2000km/yr) · 2024 · chf 9490 · Flims-Dorf · Cavigelli Motos · good (+7%, vs 4)
      https://motorradhandel.ch/en/d/yamaha/tenere-700/8638348

  price changes 1
    ↓ yamaha tenere 700 · chf 10990 → 9990 (-1000) · Sargans · https://...

  sold          1
    - yamaha tenere 700 · 6900km · 2021 · chf 9000 · Bern · 18 days on market

  relisted      0
  updated       9

  tracking  10 listings — listings_db.csv
```

## price scoring

each listing is scored against its year and model peers (e.g. all 2024 tenere 700s in the current db). the score is a percentage showing how much cheaper or more expensive the listing is relative to what you'd expect given:

- **mileage** — adjusted at ~0.7 chf/km relative to the group median
- **canton** — regional price multipliers (zürich +8%, valais −6%, etc.)
- **seller type** — dealers are expected to charge ~7% more than private sellers for the same bike. a dealer at median price scores "good"; a private seller at that same price scores "fair"

score labels:

| label | meaning |
|---|---|
| `great deal` | >15% cheaper than expected |
| `good` | 7–15% cheaper |
| `fair` | within ±6% of expected |
| `overpriced` | 7–15% more expensive |
| `expensive` | >15% more expensive |
| `n/a` | only one listing in peer group — no comparison possible |

the `vs N` number shows how many listings the score is based on. a score based on 2 peers is weaker than one based on 6.

## adding models to track

edit the `categories` list in `src/main.rs`:

```rust
let categories: &[(&str, &str)] = &[
    ("Tenere_700", "https://motorradhandel.ch/..."),
    ("Africa_Twin", "https://motorradhandel.ch/..."),
];
```

the key is used as the category id in the database — keep it stable once set. get the search url directly from the motorradhandel.ch search page.

## database

plain csv, one row per listing ever seen. opens in excel or libreoffice calc.

| field | description |
|---|---|
| `listing_id` | site's own ad id — stable primary key |
| `fingerprint` | content hash used for relist detection |
| `status` | `active` / `sold` / `relisted` |
| `first_seen` | date first scraped |
| `last_seen` | date last seen active |
| `original_price_chf` | price when first seen — never overwritten |
| `price_chf` | current price — updated each run |
| `is_private` | true if private seller, false if dealer |
| `seller_name` | garage or dealer name, empty for private |
| `location` | city |
| `kanton` | canton code (ZH, BE, VS, …) |
| `price_score` | % cheaper (+) or pricier (−) than expected |
| `price_label` | human-readable score label |
| `score_peers` | number of listings the score is based on |
| `previous_listing_id` | original ad id if this is a detected relist |

## notes

- only listings with a price are tracked
- mileage fingerprinting uses 100km steps below 1000km and 1000km steps above to absorb small discrepancies between relists
- two simultaneously active listings with identical model/year/mileage share a fingerprint — ambiguous cases are not flagged as relists
- world raid and base tenere 700 are scored separately where enough peers exist, otherwise fall back to year-group comparison
