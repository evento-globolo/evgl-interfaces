# Architecture

Canonical Evento Globolo OpenAPI, AsyncAPI, JSON Schema, event, and provider contracts.

## Fleet

- `evgl-interfaces`
- `evgl-api`
- `evgl-mash-web`
- `evgl-leptos-web`
- `evgl-dioxus-web`
- `evgl-sync`
- `evgl-cli`
- `evgl-infra`
- `evento-globolo-clients`
- `evento-globolo-libs`
- `evento-globolo.github.io`
- `evento-globolo-monorepo`

Interfaces own wire formats; libraries own reusable domain behavior; clients consume versioned contracts; runtimes own deployment behavior; monorepos coordinate pinned revisions. Edge code is allowlisted and never a generic proxy.

`openapi.yaml` and `asyncapi.yaml` define the live `/v1` event, provider, OAuth, connection, cross-post job, and WebSocket surfaces. `policy/provider-capabilities.json` is the closed automation policy, while `schemas/provider-target.schema.json` allows only non-secret provider options. Libraries and services may implement stricter validation, but they must not broaden automation modes or accept credentials in target payloads without first changing these contracts.
