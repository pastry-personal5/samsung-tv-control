# P1-M9: Remote Command Admission Policy

Status: Complete

## Approach

Build on the pure application state from P1-M8 and the typed command from
P1-M5. A single synchronous application policy function validates the request
against one current snapshot and returns either a typed rejection or an
eligible command value. It must not mutate connection state or perform side
effects.

Distinguish policy approval from dispatch admission and transport outcome.
There is no queue in this milestone, so the success case should be named and
documented as eligible/validated rather than sent, completed, or confirmed.
Only the selected identity at the current generation may pass, and Pairing and
Connection must independently be ready.

## Module responsibilities

- `application::command`: retain the typed command and define focused policy
  result/rejection types.
- `application` policy/service: evaluate target and lifecycle preconditions
  against a consistent state value. Keep it synchronous and deterministic.
- `presentation::iced::view_model`: derive control availability and safe
  rejection explanations from the same policy types.
- `presentation::iced::app`: invoke the policy boundary for presentation
  intents; do not create an asynchronous task or transport call.

Capability checks are deferred until capability evidence exists. This policy
must not treat unknown support as supported; a later capability-aware step can
extend it before transport dispatch is introduced.

## Sequencing

1. Define the rejection and eligible policy result types.
2. Implement the pure request evaluation and table-driven tests for all
   preconditions, including a stale target and passing case.
3. Project the policy decision into button enabled state and accessible reason
   text.
4. Run Cargo gates and verify that an eligible decision does not publish a
   sent/confirmed message or cause I/O.

After P1-M9, a separate milestone can plan a bounded command dispatcher and a
transport adapter once the remaining hardware and protocol decisions are
ready.
