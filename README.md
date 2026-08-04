# evgl-interfaces

Canonical Rust, JSON Schema, OpenAPI, AsyncAPI, and PostgreSQL contracts for Evento Globolo.

**Product:** Evento Globolo — A global event discovery and aggregation platform.

Aggregate, normalize, deduplicate, search, and follow events from sources such as Eventbrite, Meetup, LinkedIn, Facebook, and Craigslist through authorized APIs or permitted ingestion paths.

## Safety and production boundary

Provider names are integration targets, not claims of affiliation. Use official APIs and permitted data-access methods; do not bypass authentication, anti-bot, rate-limit, copyright, or platform-policy controls.

This repository is an executable bootstrap, not a production deployment. Before live
use, add authentication, tenant authorization, rate limits, durable migrations,
observability, backups, incident response, dependency review, and secret management.
## Contract authority

- `src/lib.rs` is the Rust model and validation surface.
- `schemas/` contains JSON Schema Draft 2020-12 wire contracts.
- `openapi.yaml` defines REST endpoints.
- `asyncapi.yaml` defines WebSocket event envelopes.
- `sql/` provides a deny-by-default PostgreSQL/Supabase migration baseline.
- `fixtures/` provides cross-language conformance examples.

Downstream services should consume a tagged release and run fixture compatibility
tests before deployment.
