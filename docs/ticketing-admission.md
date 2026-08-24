# Ticket inventory and offline admission contract

This repository is the canonical source for the Evento Globolo ticketing wire
types and PostgreSQL schema. Runtime code imports the migration constants from
the pinned `evgl-interfaces` commit; it must not copy or independently evolve
the SQL.

Apply the migrations in order:

1. `sql/001_initial.sql` for the original event baseline where applicable.
2. `sql/002_ticketing_inventory.sql` for inventory and order state.
3. `sql/003_admission.sql` for entitlements, keys, tokens, scanner receipts,
   and deterministic admission decisions.

The new migrations intentionally use `event_inventory` as the capacity
authority instead of assuming that every deployed `events` schema has already
converged on one event shape. An integration migration can add the event
foreign key once DEN-3463's event persistence work lands.

## Inventory transaction boundary

`evgl_reserve_tickets` takes a PostgreSQL transaction-scoped advisory lock for
the event, expires stale holds for that event, locks the event/class rows, and
computes both class and shared event utilization before inserting a hold. That
single serialization boundary prevents two workers or two ticket classes from
overselling the same event cap.

Holds, checkout requests, payment callbacks, cancellations, and waitlist joins
carry stable idempotency keys. Reusing a key with the same request returns the
existing aggregate; reusing it for different data fails closed. Mutable tables
are projections. `ticket_order_history` and `ticket_inventory_ledger` are the
append-only audit record, and their unique idempotency constraints make expiry,
payment, and capacity return exactly-once effects.

Waitlist ordering is the immutable `position` identity. Promotion locks the
oldest waiting row, creates an ordinary expiring hold, and records a separate
offer. An expired offer returns the entry to `waiting` without assigning a new
position. Inventory receipts contain aggregate counts and opaque identifiers,
never attendee data; waitlist callers must provide a one-way attendee reference
rather than an email, name, or phone number.

## Admission token and offline window

An admission token is two URL-safe, unpadded base64 components separated by a
period:

```text
base64url(canonical AdmissionTokenClaims JSON).base64url(Ed25519 signature)
```

Claims bind version, token, event, ticket, order, issuance epoch, key ID,
issuance time, and expiry. A scanner accepts a token only when all of the
following are true:

- the compact value and Ed25519 signature are valid;
- the token event is the scanner's selected event;
- the token is inside its issuance/expiry window;
- its key ID and issuance epoch exist in the snapshot and the token fits the
  key's activation/retirement window;
- the token expires no later than the snapshot itself; and
- the snapshot contains no matching ticket revocation at the same or a newer
  issuance epoch.

The API runtime defaults the maximum token lifetime to 12 hours. Operators may
choose a shorter window, and scanner snapshots impose an independent hard
`valid_until`. Once that instant arrives, the scanner must stop admitting and
refresh; continuing offline would conceal new refunds or revocations. This is
the surfaced stale-revocation risk, not a best-effort warning.

Signing-key rotation uses an explicit overlap: both key descriptors may appear
in a snapshot, but each token remains bound to exactly one key and epoch. Old
tokens cease to verify when either their own expiry, the old key retirement, or
the snapshot boundary is reached. Private admission and scanner keys are never
stored in these tables; only 32-byte Ed25519 public keys are persisted.

## Scanner receipts and reconciliation

Each scanner signs canonical receipt claims containing its scanner ID, key ID,
non-negative sequence, token/event/ticket/order identity, and scan time. The
database makes `(scanner_id, scanner_sequence)` idempotent. An exact replay
returns the original receipt; different signed data at the same sequence is a
protocol conflict.

Delivery order is not assumed to match sequence order. A delayed lower sequence
is preserved and marked `sequence_out_of_order` for audit instead of being
dropped. Every validly signed receipt remains append-only, even when it loses
reconciliation.

For each ticket, the winning candidate is the lexicographic minimum of:

```text
(scanned_at, scanner_id, scanner_sequence, receipt_id)
```

That rule is stable across receipt arrival order and database workers. Exactly
one candidate becomes `accepted`; other candidates are `duplicate_review` and
remain visible to staff. Invalid, expired, revoked, or cross-identity receipts
are `rejected`. Revocation advances the entitlement epoch and recomputes the
decision atomically.

## Runtime integration status

`evgl-api` provides the callable ticketing and admission services, imports these
migrations by pinned Git revision, and tests the contract against a real
PostgreSQL server. HTTP route exposure is deliberately deferred until the event
handler and organizer boundary in evgl-api PR #9 are integrated; adding parallel
routes in this slice would create a second event authority.
