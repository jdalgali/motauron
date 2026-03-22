# ADR 0002: Frontend Architecture (Feature-Driven)

## Status
Proposed

## Context
As we extend `motauron` to include a graphical interface using Vite and React, we need an architectural pattern that prevents the UI codebase from becoming a tangled "big ball of mud." Since the backend strictly adheres to Hexagonal Architecture, the frontend should have similarly rigid, logical boundaries.

## Decision
We will adopt a **Feature-Driven Architecture** for the React application. The codebase will be organized into strict directories restricting imports across bounds:

1. `src/app/`: Global application setup (routing, top-level context providers).
2. `src/pages/`: Route-level components that compose features into a full view.
3. `src/features/`: Domain-specific functionality (e.g., `listings`, `market_stats`). Each feature is self-contained with its own components, hooks, and localized logic.
4. `src/shared/`: Reusable primitives that belong to no specific business domain (e.g., generic `Button`, `Card`, typography, and pure Firebase connector logic).

### Rules of Import
- **Features** may import from `shared`, but **never** from other `features`. If features must communicate, they do so through `pages` or global context in `app`.
- **Shared** must not import from `features`, `pages`, or `app`.

## Consequences
- **Positive**: High modularity. It will be easy to delete or replace entire features without breaking the app.
- **Positive**: Clean code scaling. Multiple agents or engineers can build separate features with minimal merge conflicts.
- **Negative**: Slight overhead for simple components, requiring them to be properly placed in either `features` or `shared`.
