# P1-M1: Initial product and platform research

Status: Active
Research date: 2026-10-05

This note condenses the evidence and product constraints needed to start
P1-M2. It separates user-provided targets and vendor-documented platform facts
from community-observed TV protocol behavior and unverified assumptions.

## Recorded targets

| Target | Record | Confidence |
| --- | --- | --- |
| Mac | MacBook Pro, M5 Max, 128 GB memory | Verified on the current host with `system_profiler SPHardwareDataType`. |
| macOS | Tahoe 26.7.1 (build 25G241) | Verified on the current host with `sw_vers`. |
| TV | Samsung KU75UA8090FXKR | User-provided exact model code. Samsung Korea identifies the KU75UA8090F family as a Tizen Smart TV. |

The Mac and macOS targets are verified on the current host. The target TV makes
the local Tizen WebSocket protocol a reasonable first implementation
candidate. This does not prove that the exact firmware exposes the endpoint,
which ports are enabled, or how its TLS and approval flows behave. Those
require a TV hardware check. No hardware serial/UUID, device IP, MAC address,
TV firmware version, or pairing token is recorded here.

## Compatibility and protocol evidence

- The likely awake-TV path is the community-documented local Tizen WebSocket
  API on port 8002 (`wss`) or port 8001 (`ws`). Port 8002 should be preferred;
  8001 has no transport confidentiality and should only be offered as a
  clearly disclosed compatibility choice.
- The commonly observed first-use flow requests approval on the TV and
  returns a token for later sessions. Treat that token as a bearer secret.
- Community projects and Home Assistant document model/year differences,
  key-code variation, repeated approval prompts, and a separate encrypted
  legacy flow on some older H/J sets. None is a Samsung support guarantee for
  this third-party consumer-remote endpoint.
- Discovery method reliability is unsettled. Provide manual host entry even
  if automatic discovery is added.
- Wake is separate from the active WebSocket connection. WoL/WoWLAN requires a
  saved MAC and enabled TV/network settings; fully disconnected power cannot
  be overcome by this client.
- The SmartThings cloud API is a possible later option, but requires account
  authorization, Internet access, device capability checks, and a larger
  privacy surface. It is not needed for the first local-control path.

Protocol details, key examples, power behavior, and sources are in the
[remote protocol research](samsung-tv-remote-protocol.md). Security risks,
mitigations, and hardware validation questions are in the
[security research](samsung-tv-protocol-security.md).

## Minimum v1 remote contract

Initial controls: directional navigation, Select/OK, Back, Home, volume up,
volume down, mute, and power off. Send discrete `Click` key events. Hold/long
press, arbitrary key injection, channel/app launching, text input, voice input,
and cloud control are outside this minimum until separately validated.

Power behavior must be explicit: do not retry a potentially toggling power
command after an uncertain send; treat a socket closing after power-off as an
expected transition; offer Wake as a distinct WoL action. The product must not
claim a reliable on/off state from a failed command or connection alone.

Expected failure behavior:

| Condition | User-facing result / behavior |
| --- | --- |
| No candidate / unreachable TV | Explain that the TV could not be reached; allow retry or manual host entry. |
| Local Network access denied | Explain that macOS permission is required and point to System Settings. |
| TV approval pending or declined | Tell the user to approve on the named TV or cancel; do not retry in a loop. |
| Token rejected / revoked | Stop reconnecting with that token and offer re-pairing. |
| TLS/certificate mismatch | Fail closed and request explicit renewed trust; never silently fall back to plaintext. |
| Unsupported key / model behavior | Report that the action is unsupported; do not repeat it automatically. |
| Wake timeout | Report that wake was not confirmed and mention TV/network wake settings; do not claim the TV is off. |
| Connection lost during ordinary navigation | Mark connection unavailable, reconnect with bounded backoff, and do not replay an uncertain command. |

Responsiveness is a product requirement but has not been measured on this
target. Keep UI input responsive while connecting and sending; establish a
hardware latency baseline during implementation before publishing a numeric
response-time claim.

## macOS, Rust, distribution, and privacy constraints

- macOS local-network privacy applies starting with macOS 15. Direct unicast
  connections to local hosts count, not only Bonjour. Include
  `NSLocalNetworkUsageDescription`; if Bonjour browsing is selected, declare
  the service types in `NSBonjourServices`. Request access in the flow where
  the user adds/connects a TV and explain the purpose.
- For App Store distribution, App Sandbox is required. For a sandboxed client,
  outgoing TCP connections need the network-client entitlement. WoL/discovery
  using UDP broadcast or multicast may require additional network permissions
  and needs a focused entitlement review. Do not request an incoming listener
  unless the chosen discovery design actually needs one.
- For direct distribution, plan for Developer ID signing and notarization;
  Apple requires Hardened Runtime for notarization. Final requirements should
  be checked against the selected packaging route when one is chosen.
- Pairing tokens belong in macOS Keychain. Redact tokens, complete pairing
  events, device identifiers, and local addresses from logs and diagnostics.
  Keep device data local; do not add telemetry or cloud sync by default.
- The product direction is a native macOS UI with a Rust application and
  protocol core. A thin native presentation/permission boundary around Rust
  keeps macOS privacy prompts and Keychain integration explicit. A fully
  Rust-owned UI is also possible, but its platform bridge and packaging
  constraints should be compared in P1-M2 before implementation.

## Assumptions and open questions for P1-M2

1. Treat this one user-provided TV as the first hardware validation target,
   not as evidence for a broad compatibility promise.
2. Decide the UI/FFI boundary and packaging route (Mac App Store sandbox vs
   Developer ID direct distribution); this determines entitlements and
   release work.
3. Test this TV's actual endpoint, TLS chain/fingerprint stability, consent,
   token revocation, key map, power-off, and wake behavior.
4. Choose a discovery mechanism only after testing what this set advertises;
   keep manual entry regardless.
5. Decide whether port 8001 is supported at all. If retained, require an
   explicit per-device insecure-transport choice.
6. Set measurable interaction and wake timeout targets from hardware trials.
7. Define the supported model/firmware boundary and whether any legacy
   adapter is worth maintaining.

## Sources

- Samsung Korea, [KU75UA8090FXKR product support](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/) — family identity and Tizen platform.
- Apple, [TN3179: Understanding local network privacy](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy) — macOS 15+ behavior, purpose string, and Bonjour declarations.
- Apple, [Local Network Usage Description](https://developer.apple.com/documentation/bundleresources/information-property-list/nslocalnetworkusagedescription) — applies to direct local unicast as well as Bonjour/multicast.
- Apple, [Outgoing network client entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.network.client) and [App Sandbox configuration](https://developer.apple.com/documentation/xcode/configuring-the-macos-app-sandbox) — sandboxed network access.
- Apple, [Notarizing macOS software](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution) — Developer ID and Hardened Runtime requirements for direct notarized distribution.
- Apple, [Keychain Services](https://developer.apple.com/documentation/security/keychain-services) — secure storage for small secrets.
- Home Assistant, [Samsung TV integration](https://www.home-assistant.io/integrations/samsungtv) — maintained interoperability notes for local API support, key variation, legacy exceptions, permissions, and network limitations.
- Community, [samsung-tv-ws-api endpoint reference](https://github.com/xchwarze/samsung-tv-ws-api/blob/master/_autodocs/endpoints.md) — reverse-engineered endpoint and message observations.

Apple sources establish macOS platform and distribution behavior. The Samsung
support page establishes product-family/Tizen facts. The consumer remote
channel details remain community-observed and need confirmation against the
recorded TV.
