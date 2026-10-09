# Research: discovery, pairing, and Wake interaction

Status: Proposal for owner review — 2026-10-10

This note proposes a clearer setup path without changing the current P1-M14
owner decisions. The current interface is documented in
[UX terms](../ux-term.md),
[information architecture](../ux-information-architecture.md), and the
[GUI specification](../ux-gui.md). The recommendations below require design
decisions and implementation work; they are not current behavior.

## What the app does today

- Discovery is explicit. It adds unsaved hosts to a candidate list; manual host
  entry remains available. Saved TVs are listed separately and can be
  reselected.
- A candidate must pass **Check TV** before **Pair** is enabled. Check TV probes
  the secure endpoint, records a certificate fingerprint, and shows name/model
  if supplied. Pairing then waits for approval on the physical TV. A successful
  pair saves, selects, and connects the TV. Re-pair uses the checked Saved TV.
- The Discovery page uses a shared Guidance card for search, probe, pair, and
  recovery status. It has no dedicated scan Cancel, inline row progress, or
  identity-confirmation dialog. Discovery results and pairing outcomes also
  reach Global Messages.
- Wake configuration is on its own page. The user enters wired or Wi-Fi MAC and
  chooses Wired, Wi-Fi, or Disabled; valid settings for a selected TV save
  automatically. Wake sends one packet and waits for paired remote readiness.
  Disconnected Remote power first probes the trusted connection, then Wakes only
  if unreachable and configured. A physical screen-on state is not observed.
- The visual remote has no separate keyboard panel. Play/Pause currently sends
  KEY_PLAY and its pause effect remains unverified. Neither issue should be
  folded into the setup flow.

## Research basis and boundaries

Apple says a first local-network operation can prompt for access on macOS 15 and
later, and that TCP, multicast, and broadcast traffic are covered. The operation
can fail before the person responds, so the UI needs a retry path. Permission
can later be changed in System Settings → Privacy & Security → Local Network.
Apple's design guidance favors requesting access in the context of an action
that needs it, with a clear purpose string. This supports putting permission
guidance next to Discover TVs and Check TV rather than creating an automatic
launch prompt.
[Apple TN3179](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy),
[Apple privacy guidance](https://developer.apple.com/design/human-interface-guidelines/privacy/)

Manual address entry is a discovery fallback, not a permission bypass: the
direct TCP check also needs Local Network access. A first prompt can produce a
transient failed operation before the person answers, so distinguish a
confirmed denial from a pending or newly granted permission and offer Retry.
The same permission diagnosis applies to Wake's UDP broadcast. If a Saved TV
stops connecting, check permission before suggesting re-pairing.

Apple recommends a transient, descriptive progress indicator and Cancel when an
operation can safely be interrupted. This supports per-task progress rather than
a single status line that can be overwritten by another operation.
[Apple progress guidance](https://developer.apple.com/design/human-interface-guidelines/progress-indicators/)

Samsung's consumer IP-control worksheet distinguishes a TV that still accepts IP
control shortly after standby from one that may require Wake on LAN later. It
identifies the interface MAC, TV settings, and network path as possible Wake
failure points. That worksheet is model-dependent guidance, not proof that any
particular TV or Wi-Fi configuration will wake. Samsung's support page shows how
to find a TV's MAC under About This TV; menus vary by model and region.
[Samsung IP-control worksheet](https://image-us.samsung.com/SamsungUS/samsungbusiness/tv-ci-resources/Samsung-IP-Control.pdf),
[Samsung network and MAC guidance](https://www.samsung.com/ca/support/tv-audio-video/verify-network-status-on-your-samsung-tv/)

The design inference is to let people finish trusted pairing first, then offer
Wake as an optional capability. “Packet sent,” “remote ready,” and “screen on”
must remain separate outcomes. The current app cannot certify physical panel
state or Wake support from a MAC format check.

## Proposed setup journey

| Step | Primary action | Feedback and exit |
| --- | --- | --- |
| Find | **Discover TVs**, with **Enter TV address** always visible | Show Searching, found count, empty, permission denied, timeout, or network failure in the Discovery card. Keep Saved TVs usable. Allow Retry. |
| Inspect | Choose a candidate, then **Check TV** | Show Checking for that row. Present host, observed name/model, and certificate fingerprint together. A changed host clears the old observation. |
| Trust | **Pair this TV** after the user has reviewed the checked host | Explain “Approve the request on your TV.” Show Waiting for TV approval with a bounded wait and safe Cancel. Only success moves it to Saved TVs and selects it. |
| Use | Connect to the saved TV | Name Connected, Connection failed, or Re-pair needed separately. Preserve selection when offline. |
| Optional Wake | **Set up Wake** after pairing, or skip | Guide the user to choose the connected interface's MAC. Save explicitly or show unambiguous automatic-save state. Offer a test with a real TV and record only observed remote readiness. |

This keeps the owner's explicit Check TV → Pair decision and the separate Wake
on LAN page. A lightweight “checked TV” review state can live in the existing
card; it need not be a modal. Discovery must never pair or select a candidate by
itself.

### Discovery states

Replace the single mutable Guidance sentence with a task status beside Discover
TVs. During search, show “Searching your local network…” and a spinner. If
cancellation is technically reliable, show Cancel; if not, show a bounded search
and avoid promising cancellation. On completion:

| Result | Suggested copy | Action |
| --- | --- | --- |
| None | “No TVs found. Make sure the TV and Mac are on the same local network, or enter its address.” | Retry; manual entry |
| Permission denied | “Local Network access is off for this app. Allow it in System Settings, then try again.” | Open relevant System Settings pane if supported; Retry. Keep manual entry visible but explain it also needs access. |
| Timeout or unavailable | “Search could not finish. You can retry or check a TV by address.” | Retry; manual entry |
| Candidates found | “Found 2 possible TVs. Check the one you want to pair.” | Check TV on a row |

Do not label a candidate as “your TV” based on an advertisement alone. If a
manual address matches a Saved TV, route to that Saved TV's recovery actions
instead of showing a duplicate candidate. Keep a failed scan from replacing the
selected Saved TV or clearing manual entry.

An old Saved TV address is a separate problem from lost trust. If discovery
finds the same TV at a new address, propose an address update only after a
fresh secure check against its saved certificate identity. A new or mismatched
certificate requires explicit identity review and re-pair; an advertisement
alone must not silently retarget the Saved TV.

### Check and pair states

The checked row should show the host and all observed identity details as one
unit, with the full SHA-256 fingerprint available to copy or expand. Treat
missing name/model as “Not supplied by TV,” not as a failed check. Show a
concise security explanation: “Check that this is the TV you intend to control.
The certificate is recorded when you pair.” The app must invalidate this
observation when host, selected candidate, or certificate changes.

Pair should be a separate action only after a fresh matching check. Pair
progress belongs to that row: “Waiting for approval on the TV…” and “Use your TV
remote to choose Allow.” Denial, timeout, missing token, and changed certificate
need distinct recovery copy. Re-pair should clearly warn that the TV may show
another approval prompt. A sent request is not a successful pairing; show Saved
only after credential storage succeeds. The current trust and Keychain behavior
should remain the security boundary.

### Wake setup and power interaction

The Wake page should open with a one-sentence choice: “Wake is optional. Set it
up if your TV stops accepting remote commands while off.” For each MAC field,
show which TV interface it belongs to and how to find it on the TV. Validate on
blur or as typed, with a specific inline error; do not leave “Wake configuration
saved” visible beside an invalid, unsaved draft. A clear **Saved for [TV name]**
confirmation after an explicit Save, or a visible **Saving… / Saved / Unsaved
changes** state for automatic save, would remove ambiguity. Retain the existing
Wired, Wi-Fi, Disabled choice and apply it only to the Selected TV.

On Power View, describe stages as **Checking saved connection → Sending one Wake
packet if needed → Waiting for remote → Remote ready / Could not reconnect**.
Keep a single manual Wake action; disable it while an attempt is running or the
paired remote is already ready. Cancel should say whether the packet was already
sent. Do not send a second packet automatically and do not convert a ready
connection into a KEY_POWER toggle: power toggle can turn an already-on TV off.

Suggested result copy:

| Evidence | Copy |
| --- | --- |
| Trusted connection succeeded before Wake | “Remote ready. No Wake packet needed.” |
| Packet send succeeded; still waiting | “Wake packet sent. Waiting for the TV's remote connection…” |
| Paired remote connected | “Remote ready. Screen state is not measured.” |
| Wait expired after send | “The Wake packet was sent, but the remote did not reconnect. Check the TV's Wake setting, active MAC, and network path.” |
| Pairing needs attention | “Re-pair this TV in Discovery. No Wake packet was sent.” |

Do not promise that a valid MAC or successful UDP send means the panel turned
on. If a future TV protocol can report an actual power state, add that as a
separately labeled observation with freshness.

## Decisions to ask the owner before implementation

1. Should the Discovery card use an inline checked-TV review with an explicit
   **Pair this TV** button, or keep the current Pair label and place identity
   details beside it? Both preserve the deliberate two-step flow.
2. Should Wake settings gain an explicit Save button, or retain automatic save
   with visible draft/saving/saved states? Either choice needs per-field
   validation and TV-scoped feedback.
3. Should a post-pair prompt offer **Set up Wake** and **Not now**? The Wake
   page remains separately reachable either way.
4. Should the disconnected Remote power face keep its current combined
   connection-check/Wake behavior, or eventually use a distinct **Wake** label?
   Preserve the current P1-M13 behavior until this is decided.

## Validation before adoption

- Deterministic tests: empty/denied/timed-out discovery, stale scan completion,
  manual candidate matching a Saved TV, changed-host or changed-certificate
  invalidation, denied/timed-out pairing, saved credential failure, wrong-TV
  Wake draft, and no duplicate wake packet.
- Native review: minimum window sizes, progress and error copy, keyboard focus,
  screen-reader order and labels, Local Network denial and re-allowance on a
  signed macOS app. The visual remote's known pointer-only accessibility gap
  needs its own resolution.
- Hardware matrix: at least one wired and one Wi-Fi Samsung TV,
  reachable-on-standby and fully offline cases, physical approval and rejection,
  Wake configured and disabled, same and changed network, and a wait beyond the
  TV's standby transition. Record packet attempt, remote readiness, and physical
  panel observation separately. Current live-TV Wake and Play/Pause behavior are
  unverified.
