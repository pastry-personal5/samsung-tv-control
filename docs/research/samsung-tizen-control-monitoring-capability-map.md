# Samsung Tizen TV control and monitoring capability map

Research date: 2026-10-10

## Executive result

A Samsung Tizen television is not one uniform remote-control API. It exposes
several distinct integration surfaces with different trust, deployment, and
compatibility properties. The current macOS application should continue to
prefer its paired, local remote connection for immediate control, but must
discover and validate each feature per TV and firmware.

The most useful near-term capabilities beyond navigation keys are:

| Capability | Best integration surface | What the app can honestly claim |
| --- | --- | --- |
| Wake / power request | Wake-on-LAN plus paired local remote | A wake or power action was requested; remote readiness is observed, panel state is not. |
| Change source | Local remote input keys | Source change requested; the selected source is not observed. |
| List and launch TV apps | Local remote installed-app and launch events | A model-specific installed app was discovered or launch was requested. |
| Type into a TV field | Local remote input-string messages | Text was sent to the focused native Tizen input field; not universal app search. |
| Remote-away control and state | SmartThings, after OAuth authorization | Only the commands and attributes that the selected device advertises. |
| Live playback state | SmartThings capability events, Matter, or an owned TV app | An observed state with source and timestamp, never an inference. |
| Rich media controls and metadata | A Tizen application the project owns | State and metadata for that participating app only. |

Samsung's documented Smart View SDK supports discovery, launching a compatible
TV application, and communication between a sender and its paired receiver
app; it is not a general-purpose macOS system-remote SDK. The documented Tizen
APIs are primarily APIs **inside a TV application**. The local consumer
WebSocket used by this project is valuable interoperability behavior, but is
not a stable Samsung public API contract. [Smart View SDK overview](https://developer.samsung.com/smarttv/develop/extension-libraries/smart-view-sdk/introduction.html)

## Control surfaces

### 1. Paired local consumer remote: best Phase 2 fit

This is the existing secure, same-LAN connection. It is low latency and has no
Samsung-account dependency, but availability varies by model, firmware,
region, and TV settings. Treat every non-key capability as a per-device probe
result, not a global promise.

- **Source control:** direct HDMI, tuner, and source-menu remote keys can make
  source selection much faster than navigating the UI. A successful write only
  establishes that the request left the Mac; HDMI connection state and final
  source are not reliably exposed by this channel.
- **Installed-app catalog and launch:** interoperable clients can request the
  TV's installed app list and launch an app by the returned, per-TV app ID.
  Cache a validated catalog and display “Launching…” rather than “Opened.”
- **Native-field text input:** a Base64-encoded UTF-8 input string can work
  when a native Tizen IME field is active. It is not a Bluetooth/HID keyboard
  and normally cannot type into an OTT app's custom on-screen letter grid.
- **Power and wake:** Wake-on-LAN can request wake before the remote service is
  available. Remote reconnection is a useful readiness signal but does not
  prove that the physical panel is illuminated.
- **Experimental extensions:** pointer/mouse frames, Art Mode, app deep-link
  parameters, explicit JSON-RPC source state, and undocumented inbound events
  should remain unshipped until a target-TV hardware spike specifies a schema,
  security properties, and failure behavior.

The existing detailed evidence and safe handling rules are in [source changes
and app launches](source-changes-and-app-launches.md) and [text input from a
macOS remote](samsung-tv-text-input.md).

### 2. SmartThings cloud: broad capability-dependent control and observation

SmartThings is the appropriate route when the product needs control outside the
home LAN or genuinely observed device state. An API Access App uses OAuth,
scoped authorization, a registered client, and—when event-driven updates are
needed—a publicly reachable HTTPS webhook. That means it is not merely a Rust
client addition: it requires a backend, consent UI, token lifecycle, webhook
verification, and a privacy/support posture.

After the user authorizes access, the app can list devices, inspect each
device's components/capabilities, read complete or capability-specific status,
and issue advertised commands. A television might expose attributes such as
power (`switch`), volume/mute, input source, media playback status, track
controls, or custom Samsung capabilities; it might expose only a subset. The
capability document is authoritative for that individual device.

SmartThings can also deliver subscribed attribute changes caused by the
physical remote, the SmartThings app, or this application. Preserve both the
event time and source (`SmartThings`) in any UI projection. The platform's
standard `mediaPlayback` capability includes playback status and commands, but
its presence on one TV does not establish availability on another. [SmartThings
device status](https://developer.smartthings.com/docs/service-integrations/query-and-list-devices),
[production capabilities](https://developer.smartthings.com/docs/devices/capabilities/capabilities-reference),
and [event delivery](https://developer.smartthings.com/docs/service-integrations/webhook-events)

Useful monitoring candidates, conditional on advertised capability and fresh
status, are:

| Observation | Confidence | Important caveat |
| --- | --- | --- |
| Online/offline or power state | Potentially observed | It is cloud/device-reported, not panel-light sensing. |
| Volume and mute | Potentially observed | Available only if the device exposes those capabilities. |
| Input/source | Potentially observed | Do not substitute it for a local source result unless a current update arrives. |
| Playback state | Potentially observed | Not a guarantee of title, position, or state for every streaming app. |
| Playback metadata/position | Rare / provider-specific | Do not design UI around it until a target device actually reports it. |

### 3. Matter: future optional local observation/control

Matter has media clusters, but a TV must expose and the user must commission a
compatible endpoint. It is therefore a potential provider, not a baseline
assumption for Samsung Tizen televisions. If evaluated later, model it as an
independent `Matter` state source with its own commissioning and subscription
flow; do not tunnel it through the Samsung local-remote adapter.

### 4. A project-owned Tizen TV app: rich, narrowly scoped integration

If the product eventually includes a companion Tizen app, documented TV APIs
can provide high-quality control and state for media that *that app* owns. For
example, the Tizen MediaController API can publish playback state, position,
metadata, supported operations, and change listeners between a participating
server and client. The TV Audio Control API can get/set the associated TV
audio state from a TV application. These APIs do not grant a macOS client
visibility or control over arbitrary Samsung, Netflix, YouTube, or other
third-party apps. [MediaController API](https://developer.samsung.com/smarttv/develop/api-references/tizen-web-device-api-references/mediacontroller-api.html),
[TV Audio Control API](https://developer.samsung.com/smarttv/develop/api-references/tizen-web-device-api-references/tvaudiocontrol-api.html)

## What cannot be promised

- A generic local API for the active app, title, episode, playback position, or
  reliable playing/paused state.
- Universal text entry into streaming-service search screens.
- A portable installed-app catalog or app IDs shared among TVs, countries, or
  firmware revisions.
- Confirmation that a local power key or Wake-on-LAN packet changed physical
  panel power.
- Unattended cloud access without a user-authorized SmartThings integration.
- State from Tizen APIs owned by another app.

## Product boundary for Phase 2

Phase 2 should deliver conditional, local control for the app's existing
Sources, Apps, and Text Input destinations. It should add a single
capability/proof vocabulary so an accepted command is never represented as an
observed TV state. It should *not* add SmartThings OAuth, a hosted webhook
service, Matter commissioning, an owned Tizen application, or unsupported
telemetry.

That sequencing gives users three visible, locally useful features without
expanding the product into an account-linked cloud service. Cloud observation
should be a separately planned phase after a backend and privacy decision.

## Hardware validation matrix

Run every local feature on the target firmware and retain results only in local
test notes:

| Feature | Test result to record | Pass condition |
| --- | --- | --- |
| Sources | Each connected HDMI input, tuner, source menu, wake/reconnect | UI says requested; no incorrect selected-source assertion. |
| Apps | Returned catalog, two launches, stale entry, wake/reconnect | IDs are per-TV; launch result is honestly pending/failed. |
| Text | Native browser/settings fields, Korean, emoji, password handling, OTT grids | Native text works if supported; OTT limitation is visible. |
| Monitoring spike | Device capabilities/status before and after physical remote actions | Only actual attributes become later UI commitments. |

Never commit IP addresses, MAC addresses, pairing tokens, SmartThings device
IDs, account data, raw event payloads, or typed text.
