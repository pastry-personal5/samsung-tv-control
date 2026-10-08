# Samsung TV Remote: initial technical research

Research date: 2026-10-05

Later project decision (2026-10-08): P1-M10 uses only secure port 8002. The
port-8001 fallback discussed below remains research context, not M10 scope.

## Recommendation

Build the first release around the **local Tizen WebSocket remote API** for
Samsung smart TVs that expose it. It offers immediate LAN control without a
cloud account and supports the core remote actions required by this project.

Treat **power-on** as a separate Wake-on-LAN (WoL) / Wake-on-Wireless-LAN
(WoWLAN) capability. Treat cloud control through SmartThings as an optional
future fallback, not the primary remote path. It has an account, OAuth/token,
Internet-connectivity, and model-capability dependency.

The local WebSocket interface is widely implemented and used by mature open
source integrations, but it is not documented as a public consumer remote API
by Samsung. It must therefore be isolated behind a small protocol module,
tested against real TVs, and presented to users as model-dependent.

## Scope and compatibility

| TV family / state | Likely path | Product implication |
| --- | --- | --- |
| Recent Samsung Tizen TV, awake | Local WebSocket remote API on port 8001 or 8002 | Primary supported experience. |
| Recent TV in standby | WoL or WoWLAN, then reconnect | Needs a saved MAC address and a reconnection wait. |
| TV fully disconnected from network power | None | The app cannot turn it on. |
| Older H/J-series TV | May use an encrypted, different protocol | Out of v1 unless a tested compatibility adapter is added. |
| SmartThings-enrolled TV with supported capabilities | SmartThings REST commands | Optional fallback / remote-away-from-home feature. |
| Commercial signage / hospitality display | Separate documented control protocols | Do not assume the consumer-TV remote protocol applies. |

Home Assistant documents both REST and WebSocket support for Samsung TV models
and calls out H/J models as an encrypted-protocol exception. It also cautions
that some key codes differ by 2016+ model generation. [Samsung TV integration
documentation](https://www.home-assistant.io/integrations/samsungtv)

## Local protocol: pairing and key presses

### Discovery and endpoint selection

1. Discover candidates on the local network (the exact discovery mechanism is
   still a spike item; users must also be able to enter an IP/host manually).
2. Probe `http://TV_HOST:8001/api/v2/` and, if necessary,
   `https://TV_HOST:8002/api/v2/`, for device metadata and capability clues.
3. Prefer secure WebSocket (`wss`, port 8002) when supported; retain port 8001
   only as a compatibility fallback. Some current models no longer support the
   non-secure port.
4. Open the remote channel with a URL-safe Base64 display name. The first
   connection has no token and triggers consent on the TV. Save the returned
   token only after consent; use it on subsequent connects.

Observed remote channel shape:

```text
wss://TV_HOST:8002/api/v2/channels/samsung.remote.control
    ?name=BASE64_URL_ENCODED_APP_NAME
    &token=PAIRING_TOKEN
```

The `name` must be a clear, stable value such as `Samsung TV Remote for macOS`:
it is what the TV shows during pairing and in its trusted-device list. A
community implementation documents the URL, initial consent/token flow, and
the fact that newer sets may require port 8002. [samsungtv-rc protocol
notes](https://github.com/pkapteijn/samsungtv-rc)

### Key message

Send one JSON message per logical button press:

```json
{
  "method": "ms.remote.control",
  "params": {
    "Cmd": "Click",
    "DataOfCmd": "KEY_RIGHT",
    "Option": "false",
    "TypeOfRemote": "SendRemoteKey"
  }
}
```

For a normal remote, `Cmd: "Click"` is sufficient. Keep `Press` and `Release`
available internally for long-press experimentation, but do not expose them in
the initial UI without hardware validation. Serialize outbound commands and
use a small configurable inter-key delay for sequences so rapid navigation is
not dropped by slower TV firmware.

The message structure and `Click`/`Press`/`Release` command variants are
independently documented by maintained community implementations. [Remote
message example](https://github.com/pkapteijn/samsungtv-rc) and [command-line
reference](https://github.com/xchwarze/samsung-tv-ws-api/blob/master/COMMANDS.md)

### Initial command map

| App action | Local remote key | Notes |
| --- | --- | --- |
| Up | `KEY_UP` | Navigation |
| Down | `KEY_DOWN` | Navigation |
| Left | `KEY_LEFT` | Navigation |
| Right | `KEY_RIGHT` | Navigation |
| OK / Select | `KEY_ENTER` | Navigation |
| Home | `KEY_HOME` | Navigation |
| Back | `KEY_RETURN` | Add alongside Home in the first UI. |
| Volume up | `KEY_VOLUP` | Usually one increment per click. |
| Volume down | `KEY_VOLDOWN` | Note the spelling: `VOLDOWN`, not `VOLDN`. |
| Mute | `KEY_MUTE` | Useful first-release addition. |
| Power off / standby | `KEY_POWER` | Model behavior needs validation. |

These navigation and volume key mappings are used by Home Assistant's Samsung
TV integration, including `KEY_LEFT`, `KEY_RIGHT`, `KEY_ENTER`, and
`KEY_VOLUP`. [Supported key commands](https://www.home-assistant.io/integrations/samsungtv)
Samsung's TV remote-key documentation also confirms the corresponding physical
remote concepts: arrows, Enter, Back, Home-class navigation, and volume
controls vary by remote type. [Samsung remote-control guide](https://developer.samsung.com/smarttv/develop/guides/user-interaction/remote-control.html?device=htv)

Do not hard-code a claim that every key is available. TVs may ignore
unsupported keys, and firmware/model differences exist. Keep the mapping in a
data table and make advanced keys additive.

## Power behavior

### Power off

When the WebSocket is connected, send `KEY_POWER` and regard the subsequent
connection loss as an expected transition rather than a generic transport
failure. Do not retry the same power command automatically: it may wake a TV
that only entered standby.

### Power on

The normal remote channel cannot be assumed reachable while the TV is asleep.
Send a standard WoL magic packet to UDP port 9 (optionally make the port
configurable) using the TV's saved MAC address, then poll/reconnect with a
bounded backoff. WoL works only if the network interface remains powered and
the relevant TV/network setting and physical network conditions permit it.

Samsung's Smart View documentation confirms WoWLAN for a previously connected
Tizen TV, says 2016 TVs also support wired WoL, and requires retaining the
TV's MAC address after a successful connection. [Samsung WoWLAN
guide](https://developer.samsung.com/smarttv/develop/extension-libraries/smart-view-sdk/ios-sender-app/enhanced-features.html)

Product behavior:

- Save the MAC only after user consent and explain why it is needed.
- Offer **Wake TV** separately from a generic power toggle when state is
  uncertain.
- After waking, show “Waking TV…” and wait up to a configurable bound (for
  example, 30 seconds) before reporting failure.
- Tell users to enable the appropriate network/remote-start setting on their
  TV when wake fails; exact labels differ by model and region.

## SmartThings: optional cloud fallback

SmartThings can send device commands with:

```text
POST https://api.smartthings.com/v1/devices/{deviceId}/commands
Authorization: Bearer ACCESS_TOKEN
```

Commands target a component and capability, for example `main` / `switch` /
`off`. The available commands are device-specific; query the selected device's
capabilities before rendering controls. [SmartThings device-control
documentation](https://developer.smartthings.com/docs/service-integrations/control-devices)

This can help where local access is unavailable and may expose power, volume,
input, or Samsung-specific remote capabilities. It should not substitute for
the LAN path in v1 because it introduces authorization, account management,
network latency, and a larger privacy surface. Store OAuth refresh/access
credentials in macOS Keychain, never in project files or logs.

## macOS and Rust implementation implications

- Model the protocol behind `TvTransport` / `RemoteClient`, with a local
  WebSocket implementation and a future SmartThings implementation.
- Keep `RemoteKey` as a strongly typed enum. It prevents invalid UI strings
  from reaching the TV and makes model-specific aliases explicit.
- Keep `DeviceRecord` separate from connection state: friendly name, host,
  port/TLS preference, MAC, and pairing token are durable; socket state is not.
- Place the pairing token and any cloud credentials in Keychain. Do not put
  them in preferences, telemetry, screenshots, test fixtures, or git.
- Accept the TV's TLS certificate deliberately. Consumer TVs commonly use a
  self-signed certificate, so a secure-port client needs a narrowly scoped
  trust policy such as certificate pinning after first trusted pairing. Never
  silently disable verification for arbitrary Internet hosts.
- Make all network calls asynchronous; the app must remain responsive during
  pairing, wake, and reconnection.
- Surface errors in user language: TV not found, approval needed on TV,
  pairing declined, token revoked, wake not supported, and command unsupported.

## Validation plan before committing to a public compatibility claim

Create a hardware matrix with at least one TV per practical generation and
record model, firmware, connection type, selected port, pairing result, all
initial keys, power-off behavior, wake behavior, and recovery after sleep.
Do not commit TV IPs, MACs, pairing tokens, or serial/device identifiers.

Automated tests should use a local fake WebSocket server and fake UDP receiver
to cover:

- initial pairing request and token extraction;
- reconnect with a stored token;
- URL-safe Base64 client-name construction;
- exact JSON for every `RemoteKey`;
- rejected pairing, malformed events, timeout, socket close, and token revocation;
- ordered command sequences and cancellation;
- WoL packet byte layout and broadcast target selection.

## Open research items / recommended spikes

1. **Discovery:** establish which advertised services current consumer Tizen
   TVs reliably expose and implement manual-host fallback from day one.
2. **TLS trust:** inspect real port-8002 certificate behavior; decide between
   user-confirmed certificate fingerprint pinning and a LAN-only constrained
   policy.
3. **Wake:** test wired Ethernet and Wi-Fi on representative models, including
   standby versus fully powered-off behavior.
4. **State:** determine which `/api/v2/` fields and event channels are reliable
   enough for connected/standby/off UI states; do not infer state from a failed
   key press alone.
5. **Model aliases:** validate `KEY_POWER`, `KEY_POWEROFF`, and volume key
   behavior on each tested generation before presenting a unified power button.
6. **SmartThings consent:** if cloud control is added, prototype the current
   OAuth and capability-enumeration flow separately from the local pairing
   flow.

## Sources

- [Samsung: Smart View WoWLAN / wired WoL guidance](https://developer.samsung.com/smarttv/develop/extension-libraries/smart-view-sdk/ios-sender-app/enhanced-features.html)
- [Samsung: TV remote-control keys](https://developer.samsung.com/smarttv/develop/guides/user-interaction/remote-control.html?device=htv)
- [Samsung: SmartThings device commands](https://developer.smartthings.com/docs/service-integrations/control-devices)
- [Home Assistant: Samsung TV integration and key-code compatibility notes](https://www.home-assistant.io/integrations/samsungtv)
- [samsungtv-rc: observed WebSocket endpoint, pairing, and payload](https://github.com/pkapteijn/samsungtv-rc)
- [samsung-tv-ws-api: command behavior and current default secure port](https://github.com/xchwarze/samsung-tv-ws-api/blob/master/COMMANDS.md)

The Samsung sources are primary for supported TV-platform behavior and
SmartThings/WoL concepts. The local remote endpoint, token exchange, and
payload are corroborated by current open-source interoperability projects and
should be verified against hardware during implementation.
