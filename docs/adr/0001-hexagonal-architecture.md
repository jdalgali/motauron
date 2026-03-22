# ADR 0001: Refactor to Hexagonal Architecture

## Status
Proposed

## Context
`motauron` is a Rust application that scrapes motorcycle listings (e.g., from Motorradhandel), merges them with a historical dataset, calculates price scores, and sends notifications for new/changed/sold bikes.

Currently, the application is somewhat modular (`scrapers`, `store`, `notify`, `scoring`), but concerns are mixed:
- The `store` module is responsible for persisting to `listings_db.csv`, but it *also* contains the core business logic for merging listings, detecting relists, and triggering `scoring`.
- The `models` directory contains structs used pervasively across both IO boundary layers (scraping/JSON) and internal logic.
- `main.rs` directly orchestrates concrete implementations instead of programming against interfaces (ports).

As the application grows (adding autoscout24, new notification methods, or a real database), this tight coupling will make testing and extension difficult.

## Decision
We will adopt a **Hexagonal Architecture** (Ports and Adapters) for `motauron`.

The codebase will be restructured into three distinct layers:
1. **Domain (`src/domain/`)**: Contains the core business logic, entities (`Listing`, `PriceScore`), and pure domain services (e.g., the complex logic of merging previous listings with new ones, detecting price drops, and relists). This layer will have **no dependencies** on external IO, databases, or HTTP clients.
2. **Application (`src/application/`)**: Contains the Use Cases (e.g., `TrackMarketUseCase`) and defines the **Ports** (traits) required by the application to function (e.g., `ScraperPort`, `ListingRepositoryPort`, `NotificationPort`).
3. **Infrastructure (`src/infrastructure/`)**: Contains the **Adapters** that implement the Ports. This includes `reqwest`-based scrapers (`MotorradhandelScraper`), the `csv`-based database (`CsvListingRepository`), and the notification handlers.

## Consequences
- **Positive**: The core merging, relist-detection, and scoring logic will be fully isolated and easily unit-testable without mock IO.
- **Positive**: Adding new scrapers or changing the database to SQLite/Postgres will require zero changes to the `domain` or `application` layers.
- **Negative**: There is a slight overarching increase in boilerplate due to the necessity of defining traits (Ports) and mapping between external DTOs and internal Domain Entities if necessary.
