# motauron

watches swiss motorcycle marketplaces. tracks when listings appear, drop in price, go sold, or get relisted under a new ad — same bike, new id.

## what it does

- scrapes listing data from motorradhandel.ch (price, mileage, year, model)
- maintains a local csv database across runs
- detects relists using a content fingerprint (model · year · mileage)
- marks listings as sold when they disappear from a tracked category

## usage

```
cargo run
```

results are saved to `listings_db.csv` in the project root.

## adding models to track

edit the `categories` list in `src/main.rs`:

```rust
let categories: &[(&str, &str)] = &[
    ("Tenere_700", "https://motorradhandel.ch/..."),
    ("Africa_Twin", "https://motorradhandel.ch/..."),
];
```

the key is used as the category id in the database — keep it stable once set.

## database

plain csv file. columns:

| field | description |
|---|---|
| `listing_id` | site's own ad id — primary key |
| `fingerprint` | content hash used for relist detection |
| `status` | `active` / `sold` / `relisted` |
| `first_seen` | date first scraped |
| `last_seen` | date last seen active |
| `previous_listing_id` | original ad id if this is a detected relist |
| `price_chf` | updated on each run if the seller changes it |

## notes

- only listings with a price are tracked
- mileage is rounded when fingerprinting (100km steps below 1000km, 1000km steps above) to absorb minor discrepancies between relists
- two simultaneously active listings with identical model/year/mileage will share a fingerprint — this edge case is not flagged as a relist
