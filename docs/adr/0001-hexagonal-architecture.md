# ADR 0001: Hexagonal Architecture for the Backend

## Status
Accepted

## Context
`motauron` scrapes motorcycle listings from multiple marketplaces, merges them with historical data, calculates price scores, and sends notifications. The system has grown to include two scrapers (motorradhandel.ch and motoscout24.ch), a Firestore database, and an ntfy.sh notifier.

Without clear boundaries, scraping logic, business rules, and I/O concerns collapse into each other. Adding a new marketplace or swapping the database becomes risky and requires touching unrelated code.

## Decision
Adopt **Hexagonal Architecture** (Ports and Adapters). The codebase is split into three layers:

1. **Domain (`src/domain/`)** — Pure business logic with no external dependencies.
   - `MotorcycleListing` entity and fingerprinting
   - `MergerService` — detects price changes, sold listings, and relists
   - `ScorerService` — relative price scoring per model/year group

2. **Application (`src/application/`)** — Use cases and port definitions.
   - `TrackMarketUseCase` — orchestrates scrape → merge → score → persist → notify
   - Ports (traits): `Scraper`, `ListingRepository`, `Notifier`

3. **Infrastructure (`src/infrastructure/`)** — Concrete adapter implementations.
   - `MotorradhandelScraper` — JSON extraction from `window.__store__`
   - `MotoscoutScraper` — headless Chromium for JS-rendered pages
   - `FirestoreListingRepository` — Firestore persistence
   - `NtfyNotifier` — push notifications via ntfy.sh

## Consequences
- Domain and application layers have zero dependencies on HTTP, databases, or notification services — they are fully unit-testable in isolation.
- Adding a new scraper (e.g. autoscout24.ch) requires only a new infrastructure adapter implementing the `Scraper` trait. No domain or application code changes.
- Swapping Firestore for another database only requires a new `ListingRepository` implementation.
- Slight boilerplate overhead from defining traits and mapping between external DTOs and domain entities.
