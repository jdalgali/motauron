# ADR 0002: Feature-Driven Architecture for the Frontend

## Status
Accepted

## Context
The frontend is a React + Vite SPA that displays live motorcycle listings from Firestore. As features grow (filters, stats, alerts), an unstructured component tree becomes a maintenance liability. The backend uses strict hexagonal boundaries — the frontend should have an equivalent discipline.

## Decision
Adopt **Feature-Driven Architecture**. The `src/` directory is split into four zones with enforced import direction:

```
src/
├── shared/      Reusable utilities with no feature dependencies (Firebase connector, types)
├── features/    Self-contained domain slices (listings, market_stats, …)
├── pages/       Route-level components that compose features into views
└── app/         Global setup: routing, top-level context providers
```

**Import rules:**
- `features` may import from `shared`, never from other `features`
- `shared` must not import from `features`, `pages`, or `app`
- Cross-feature communication goes through `pages` or global context in `app`

**Current features:**
- `listings` — real-time feed from Firestore, price-score sorting, deal cards

## Consequences
- Each feature can be built, replaced, or deleted without touching other features.
- The Firebase connector lives in `shared/firebase.ts` and is never duplicated.
- Slight overhead for simple components that must be consciously placed in `features` or `shared` rather than dropped anywhere.
