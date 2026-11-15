# evgl-interfaces

Canonical Evento Globolo OpenAPI, AsyncAPI, JSON Schema, event, and provider contracts.

The transactional ticket inventory, bounded offline admission-token protocol,
and deterministic scanner-reconciliation contract are documented in
[`docs/ticketing-admission.md`](docs/ticketing-admission.md).

Initialized through `DEN-1889` as a testable `interfaces` foundation. Product behavior continues through focused pull requests.

For repository verification and Rust-backed contract regression tests:

```bash
cargo test --locked --all-targets
cargo run --locked --bin verify_repo
```

These commands are used by the primary CI job. The separate JSON Schema
meta-schema workflow still uses a legacy Python validator until a behaviorally
equivalent Rust implementation is verified. New tests and automation should
use Rust; see `AGENTS.md` and the organization-wide `.github` policy.

