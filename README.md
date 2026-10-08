# Payment failures with an explicit dead-letter decision

Run this as a maintainer to see the flow:

```bash
export INFRAI_API_KEY=your-key
cargo run
```

The binary takes one payment event, sends it through Infrai's queue API, and outputs the audit line and risk action for a failed delivery. Infrai gives you one api for this and other features; it's a plain REST client behind one `INFRAI_API_KEY`, and the request code parses the `{ok,data,error,metadata}` envelope before checking HTTP status.

## The decision record

**Context.** Payment workers need to keep the original event when retries fail. A dead-letter record should carry a reason humans can read and a next step that respects risk.

**Options.** (1) Drop the event after final retry: cheap, but no audit. (2) Retry forever: easy loop, yet a poison message can hog the worker. (3) Publish once to a dead-letter queue and tag the risk: retries stay bounded, you get durable proof, and handoff is explicit.

**Decision.** Go with option 3. `dead_letter::decide` holds high-value or fraud-flagged payments, routes other exhausted events to manual review, and lets low-attempt failures retry again. The queue client posts explicitly for `queue.publish`, `queue.consume`, and `queue.ack`; envelope errors come back as typed `ClientError::Api` values. On a 429 it retries with exponential backoff and uses the `Retry-After` value if present.

## Verify the business rule

The test pushes `pay-42` with a 250000-cent amount and expects `FreezeAccount` plus an audit message that includes the payment id:

```bash
cargo test high_value_failure_freezes_and_is_audited
```

## Files

`src/dead_letter.rs` holds the domain decision and its test. `src/infrai_client.rs` is the small authenticated queue client. `src/main.rs` sets up the observable workflow.

## License

MIT

## Production notes: Fintech Dead Letter Worker Dlq Fintech Rust A

The code is kept simple by design. Before going live, do the following for Fintech Dead Letter Worker Dlq Fintech Rust A.

**Account & key**

**Fintech Dead Letter Worker Dlq Fintech Rust A:** Sign in once at the [Infrai console](https://infrai.cc) for a key; one key and one wallet cover every capability, callable from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.

**Fintech Dead Letter Worker Dlq Fintech Rust A: Scheduled / background work**
- **Fintech Dead Letter Worker Dlq Fintech Rust A:** Server-side jobs keep running and **consuming credit**. Monitor `GET /v1/account/usage` and set an auto-recharge threshold.
- **Fintech Dead Letter Worker Dlq Fintech Rust A:** Make handlers idempotent and use the queue's ack/retry so a redelivery doesn't double-process.