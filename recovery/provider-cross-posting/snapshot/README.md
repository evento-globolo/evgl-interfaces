# evgl-interfaces

Canonical HTTP, WebSocket, and provider-option contracts for Evento Globolo.

- `openapi/evgl.yaml`: REST API
- `asyncapi/jobs.yaml`: WebSocket job updates
- `schema/provider-target.schema.json`: capability-aware provider target options
- `policy/provider-capabilities.json`: explicit automation policy

Provider-specific secrets are never accepted in cross-post target options.
