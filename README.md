# evgl-interfaces

Canonical Evento Globolo OpenAPI, AsyncAPI, JSON Schema, event, and provider contracts.

The authoritative HTTP and WebSocket contracts are `openapi.yaml` and `asyncapi.yaml`. Capability-aware cross-posting is bounded by `policy/provider-capabilities.json` and `schemas/provider-target.schema.json`: Eventbrite and Meetup use native-event delivery, Meta uses distribution posts, generic integrations use signed webhooks, and Craigslist remains a manual handoff. Provider credentials are environment or token-vault concerns and are never valid target options.

The recovered source under `recovery/provider-cross-posting/` remains as provenance; its missing behavior has been reconciled into the authoritative files above.

```bash
python3 scripts/verify_repo.py
```
