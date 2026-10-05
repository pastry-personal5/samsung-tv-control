# Samsung TV local protocol: security and reliability research

Research date: 2026-10-05

## Research target

| Component | Recorded target | Evidence / status |
| --- | --- | --- |
| Mac | MacBook Pro with M5 Max, 128 GB memory | User-provided; exact macOS hardware identifier not recorded. |
| Operating system | macOS Tahoe 26.7.1 | User-provided. |
| Television | Samsung KU75UA8090FXKR | User-provided exact model code. Samsung's Korean support page lists the KU75UA8090F family as a Tizen Smart TV; protocol endpoint behavior still needs direct testing. |

The TV target is a Tizen model, so the local WebSocket approach in this
document is a relevant candidate. The exact firmware, current network
connection, API port availability, TLS certificate behavior, and pairing flow
are not yet verified. Do not treat family-level product information as proof
that this exact firmware exposes the expected remote channel.

## Executive summary

For a native macOS remote, the practical local control path on supported Tizen
televisions is a TV-hosted WebSocket API, commonly exposed as `ws://TV:8001`
and/or `wss://TV:8002`. A first connection generally asks the TV owner to
approve the client; later connections can use a returned token. Samsung's
public documentation covers Smart View application-to-application security
and Tizen TV app APIs, but does not specify this consumer remote-control
endpoint as a stable third-party API. Treat the endpoint and token behavior as
community-documented interoperability, not a Samsung support guarantee.

The strongest risks for this app are local-network command access, token
disclosure, and unsafe TLS certificate handling. The initial release should
prefer port 8002, validate the TV certificate using an explicit pairing-time
trust decision, keep tokens in macOS Keychain, and never expose the service
outside the trusted home LAN. Port 8001 is a compatibility path with no
transport confidentiality; it should be opt-in and clearly disclosed.

This is a threat assessment for the client and its deployment assumptions. It
does not claim that a particular Samsung firmware has a known exploitable
vulnerability.

## Protocol and trust boundary

The commonly observed Tizen flow is:

1. Discover or configure the TV's LAN address.
2. Open `/api/v2/` metadata and a WebSocket channel such as
   `/api/v2/channels/samsung.remote.control`.
3. On first connection, the TV prompts the user to accept the named client.
4. The TV returns a token in a channel-connect event; a client supplies the
   token on later secure-channel connections.
5. Send JSON remote commands such as `ms.remote.control` over the open socket.

Open-source implementations document ports 8001 and 8002, token exchange,
the channel URL, and remote-control message shape. Home Assistant documents
the same local-control family, Tizen-era compatibility, recurring permission
prompts, and H/J legacy exceptions. Samsung's Smart View documentation
describes secure WebSocket operation for its own receiver-app model, but that
is not a protocol specification for the consumer remote endpoint.

The trust boundary is therefore the TV's authorization prompt and its local
network. A pairing token should be treated as a bearer secret: anyone who can
obtain and use it from an allowed network may be able to issue remote commands
without another user prompt. The TV's own device list and revoke controls are
part of the recovery story.

## Risk register

| Risk | Likelihood / impact | Why it matters | Client mitigation |
| --- | --- | --- | --- |
| Plaintext on port 8001 | Medium / High on shared or hostile LAN | Commands and any protocol credentials on the connection can be observed or altered by a network attacker. | Prefer 8002. Make 8001 an explicit compatibility fallback with a visible warning; never silently downgrade after TLS failure. |
| Weak TLS verification on 8002 | Medium / High | TVs may use self-signed certificates. Disabling validation globally makes an active LAN attacker able to impersonate a TV and capture commands or credentials. | At first pairing, display the certificate fingerprint and TV identity for confirmation; pin the accepted certificate per device and fail closed on change until the user re-confirms. Scope trust to the TV host, not the system TLS stack. |
| Pairing token disclosure | Medium / High | A stored token can provide persistent control to someone who copies it from app data, logs, a crash report, or backup. | Store tokens only in Keychain; redact query strings and protocol frames from logs; never serialize tokens into preferences, diagnostics, or URLs shown to users. Support forget/re-pair and explain TV-side revocation. |
| Unwanted first-time pairing | Low to Medium / High | A malicious or mistaken client on the LAN could prompt the TV owner to approve it. | Use a recognizable, stable app name; show the user the matching name before opening pairing; do not auto-accept, retry pairing repeatedly, or hide the TV's prompt. |
| Local-network command injection / CSRF-like abuse | Medium / High | Any app or host with LAN reachability may probe the endpoint; a TV may accept commands from a trusted segment. | Require user-selected/discovered device confirmation, constrain targets to private/link-local addresses, avoid arbitrary URL inputs in automated flows, and never create port forwarding or a public relay by default. |
| Discovery spoofing / wrong-device pairing | Medium / Medium | Discovery advertisements and hostnames can be spoofed; users can approve a neighboring TV. | Show model/name/address, allow manual selection, confirm identity on the TV, and bind stored trust to a device identity plus certificate fingerprint rather than an IP alone. |
| Token revocation and permission behavior vary | Medium / Medium | TV settings can request approval on every connection or invalidate prior tokens, appearing as flaky networking. | Model unauthorized/re-pair-required as a first-class state; do not loop; guide the user to inspect/revoke entries in the TV's device manager. |
| Cross-subnet routing and firewall differences | Medium / Medium | Some integration environments report TV WebSockets fail across VLANs/subnets; discovery also depends on multicast/broadcast boundaries. | Support manual host entry, report routing/TLS/auth failures distinctly, and document that putting IoT devices on a VLAN may block control unless narrowly permitted. |
| Legacy protocol differences | Medium / Medium | H/J families may use a distinct encrypted pairing/remote flow; ports, keys, and auth differ by model/firmware. | Keep legacy support in a separate adapter; do not infer compatibility from a successful TCP connection; qualify supported model families. |
| Power command ambiguity | Medium / Medium | `KEY_POWER` can act as toggle, and connection loss after power-off is expected. Blind retries can turn the TV back on. | Do not automatically retry power toggles; distinguish power-off from wake-on-LAN and represent uncertain state honestly. |

Ratings are qualitative engineering estimates, not measured incident rates.

## Security decisions for the macOS implementation

### Connection policy

- Try `wss` on 8002 first. Use platform TLS verification as the baseline.
- If the TV's certificate is self-signed or otherwise untrusted, make trust a
  pairing-time user decision. Show the fingerprint and observed TV metadata;
  then pin the certificate for that configured TV. A changed certificate
  requires a renewed confirmation.
- Do not implement “accept any certificate” as a general setting, trust a
  certificate solely because it came from a private address, or silently
  downgrade from TLS to plaintext.
- Offer `ws` on 8001 only when secure connection is unavailable and the user
  explicitly enables that compatibility mode. Explain that traffic is
  unencrypted and keep the option local to the selected TV.
- Set connection, handshake, and command timeouts. Bound frame sizes and JSON
  depth; reject malformed or unexpected events; avoid unbounded reconnects.

The exact 8002 certificate chain and hostname behavior should be measured on
representative hardware before implementing pinning. Use an explicit
trust-on-first-use confirmation only if strict public-CA validation cannot be
made to work with actual devices.

### Pairing and token lifecycle

- Pair only after a deliberate user action. Present the client name and TV
  identity before initiating the handshake.
- Keep the token in macOS Keychain with device-scoped lookup. Do not place it
  in `UserDefaults`, config files, crash reports, analytics, or source control.
- Ensure diagnostic logging redacts token query parameters, authorization
  data, complete pairing events, device identifiers, and LAN addresses.
- Treat an authorization error as a stopped state. Offer a clear re-pair
  action and mention the TV's trusted-device list as a revocation point.
- Forgetting a TV should delete its Keychain token and local certificate pin.

### Network scope and least authority

- This protocol is for a user's local TV. Do not expose it through Internet
  port forwarding, UPnP, or a default cloud proxy.
- Restrict manual target entry and redirects to local/private/link-local
  destinations. Do not let a crafted discovery record or metadata response
  redirect the app to arbitrary Internet hosts.
- Do not assume subnet membership itself authenticates a TV. Discovery data is
  a hint; pairing confirmation and the pinned identity establish the device
  trust decision.
- Use the smallest supported command set in the product. Avoid sending
  arbitrary method names or user-provided JSON to the TV.

## macOS-specific considerations

Apple's Local Network privacy permission may be required for LAN discovery or
connections, depending on the implementation and macOS version. The app
should request access only when the user starts adding/connecting a TV and
explain the purpose in its privacy text. Keep device metadata and tokens local
unless the user explicitly chooses a separately designed sync feature.

Store the host, friendly name, and capabilities as ordinary app data only when
needed; treat token and certificate pin as security-sensitive. Do not log
WebSocket URLs verbatim because the token may be encoded in their query
string.

## Reliability and compatibility risks

This is not a single stable protocol across all products. Samsung model year,
firmware, region, network settings, and TV family can change endpoint support,
key behavior, consent prompts, standby behavior, and state reporting. Home
Assistant currently calls out 2016+ Tizen/WebSocket support, H/J encrypted
exceptions, varying key support, repeated access notification settings, and
LAN segmentation limitations. These are useful interoperability clues, not
Samsung's support contract for this app.

Key commands are not inherently idempotent. Power is especially risky because
it can toggle; command retry policy must distinguish safe navigation retries
from uncertain toggles. A dropped socket after sending a command does not
prove whether the TV received it. State should come from an explicit TV event
or query, not an assumption based on the send result.

Wake-on-LAN is a separate unauthenticated LAN broadcast mechanism, not a
secure extension of the WebSocket channel. Use it only for a user-selected,
previously paired device and the stored MAC address. It cannot wake a TV whose
network interface is unpowered or whose settings disable wake.

## Validation questions before claiming secure support

On each supported hardware family, record sanitized results for:

1. Whether 8001 and 8002 are listening and which operations each accepts.
2. TLS certificate subject, issuer, validity, stability across reboot/firmware
   update, and behavior from a clean macOS trust store.
3. Whether the TV returns a token only after acceptance, and whether that
   token works from another host on the same LAN.
4. Whether TV-side removal revokes the token immediately and what error the
   client receives afterward.
5. Consent behavior with settings set to every connection versus first time.
6. Reachability from same subnet, another VLAN, and guest Wi-Fi, without
   weakening the home network to make the test pass.
7. Power command outcome and socket-close timing, including whether repeated
   identical commands toggle the set back on.

For the recorded KU75UA8090FXKR specifically, add its firmware version,
network type, and the sanitized results of all seven checks above after
testing. The current research record does not include a TV IP, MAC, serial
number, or pairing token.

Do not record real IP addresses, MAC addresses, serial numbers, tokens, or
unredacted pairing frames in the repository.

## Source quality and limits

- Samsung's [Smart View receiver documentation](https://developer.samsung.com/smarttv/develop/extension-libraries/smart-view-sdk/receiver-apps.html)
  documents secure WebSocket support for Smart View receiver applications.
  It does not define the consumer remote endpoint discussed here.
- Samsung's [Security Q&A](https://developer.samsung.com/smarttv/develop/faq/security.html?device=htv)
  discusses TLS support in the TV web platform, not certificate policy for the
  remote WebSocket service.
- Samsung Korea's [KU75UA8090FXKR support page](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/)
  identifies the KU75UA8090F product family and lists Tizen as its operating
  system. Product specifications do not confirm the local remote endpoint.
- The [Home Assistant Samsung TV integration](https://www.home-assistant.io/integrations/samsungtv)
  is maintained interoperability documentation for current local behavior,
  device generations, auth prompts, legacy exceptions, and network limits.
- The [samsung-tv-ws-api endpoint reference](https://github.com/xchwarze/samsung-tv-ws-api/blob/master/_autodocs/endpoints.md)
  describes observed ports, URL format, token events, and messages. It is
  reverse-engineered community documentation, not a Samsung contract.

Recheck these sources and the hardware matrix when changing support claims;
protocol observations may lag model and firmware changes.
