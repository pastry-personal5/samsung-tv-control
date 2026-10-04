# Fast source changes and app launches

Research date: 2026-10-05

## Recommendation

Use the existing, paired **local remote WebSocket** as the fast path for both
operations. It avoids SmartThings cloud latency and account requirements.

- **Physical input:** send a direct `KEY_HDMI1`, `KEY_HDMI2`, or `KEY_HDMI3`
  remote key. The target TV has three HDMI ports.
- **Installed TV app:** send the `ed.apps.launch` channel event with a cached,
  per-TV application ID.

Both use Samsung's undocumented consumer local-control interface, so probe and
cache capability results per TV/firmware; do not ship a universal hard-coded
app catalog.

Samsung's official Korean page confirms this TV is Tizen-based and has three
HDMI inputs. [Official model support/specification page](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/)

## 1. Source change

### Fast path: direct source key

Send the normal remote-key envelope on the paired local WebSocket connection:

```json
{
  "method": "ms.remote.control",
  "params": {
    "Cmd": "Click",
    "DataOfCmd": "KEY_HDMI1",
    "Option": "false",
    "TypeOfRemote": "SendRemoteKey"
  }
}
```

Substitute `KEY_HDMI2` or `KEY_HDMI3` for the other physical ports. Also keep
`KEY_TV` and `KEY_SOURCE` available as fallback keys, but do not use
`KEY_SOURCE` for a one-tap named-input button: it opens a chooser and requires
additional navigation.

Build the initial model-specific source map as:

| UI label | First command | Fallback / behavior |
| --- | --- | --- |
| HDMI 1 | `KEY_HDMI1` | Disable if not accepted or nothing is connected. |
| HDMI 2 | `KEY_HDMI2` | Disable if not accepted or nothing is connected. |
| HDMI 3 | `KEY_HDMI3` | Disable if not accepted or nothing is connected. |
| TV / tuner | `KEY_TV` | May be unavailable without a configured broadcast source. |
| Sources… | `KEY_SOURCE` | User navigates the TV chooser with the D-pad. |

Samsung's current Home Assistant integration lists `KEY_SOURCE`, `KEY_HDMI`,
and `KEY_TV` as input keys and documents source selection as model-dependent.
[Samsung TV remote input keys](https://www.home-assistant.io/integrations/samsungtv/)

### Source selection rules

- Treat a successful WebSocket write as “requested,” not “changed.” A cable
  may be disconnected, HDMI-CEC can redirect the TV, and the firmware may
  ignore a key.
- Keep labels user-editable: “PlayStation” can map to `KEY_HDMI1`, for example.
- If a direct `KEY_HDMI*` attempt fails in hardware testing, fall back to
  `KEY_SOURCE`, then use a user-visible, manually navigated chooser. Do not
  automate unknown UI sequences.
- The TV is not a Samsung commercial display: do not use signage RS-232/MDC
  protocols.

### Optional fast path: local IP Control

Some Samsung TVs expose `inputSourceControl` over local JSON-RPC (often port
1516), enabling an explicit source value such as `HDMI1`. This is not a safe
v1 dependency for this model: it requires separate pairing/enabling and is
model/firmware-dependent; unsupported TVs return `Method not found`.
Use it only as an optional capability after the WebSocket direct keys have been
tested. [Current compatibility notes for local input-source control](https://github.com/TheFab21/ha-samsungtv-smart)

## 2. Launch an installed Samsung TV app

### Fast path: one local WebSocket event

With an app ID already known for *this TV*, send:

```json
{
  "method": "ms.channel.emit",
  "params": {
    "event": "ed.apps.launch",
    "to": "host",
    "data": {
      "appId": "APP_ID_FROM_THIS_TV",
      "action_type": "DEEP_LINK"
    }
  }
}
```

For example, do not replace `APP_ID_FROM_THIS_TV` with a guessed Netflix or
YouTube ID. App IDs and whether an installed app accepts a deep launch can vary
by region, app revision, and TV firmware.

### Discover and cache the app list

After local pairing, request installed apps:

```json
{
  "method": "ms.channel.emit",
  "params": {
    "event": "ed.installedApp.get",
    "to": "host"
  }
}
```

Parse the corresponding installed-app event, then persist only display name,
app ID, and a last-verified timestamp in the device record. Refresh the list on
first connection after a week, after an app-launch failure, or on explicit user
refresh—not before every button press. That makes launch a single send in the
common case.

Current interoperable Samsung TV clients use this WebSocket event pair for
installed-app discovery and `ed.apps.launch` for a deep launch. [App discovery
and launch implementation](https://github.com/xchwarze/samsung-tv-ws-api/blob/master/samsungtvws/remote.py)

### App-launch rules

- Require a live, paired WebSocket connection. If the TV is asleep, wake it
  first, wait for the socket to reconnect, then launch.
- On write success, display “Launching *App name*…” rather than “Opened.” The
  TV may reject the app, need an update, or remain in a previous app.
- Mark an app unavailable after a clear failure; keep it in the UI only as an
  editable/refreshable stale entry.
- Prefer `DEEP_LINK` with no `metaTag` for an ordinary home launch. A URL or
  app-specific deep-link target is an advanced, per-app feature and should not
  be guessed.
- Do not embed or send account credentials, search text, or private deep-link
  parameters in diagnostics.

Samsung's Smart View documentation establishes that TV apps are addressed by
app ID and can be launched from a mobile sender application. [Samsung Smart
View app-launch overview](https://developer.samsung.com/smarttv/develop/extension-libraries/smart-view-sdk/android-sender-app.html)

## SmartThings: fallback, not the fast local path

SmartThings can execute only the commands advertised by the selected device's
components/capabilities. Query the registered TV first and show an input or
app control only if the device advertises the relevant command. Context7's
current SmartThings Core SDK documentation confirms that it sends specified
device commands, including batches; it does not establish a universal Samsung
TV app-launch API.

For this macOS local remote, prefer WebSocket source/app actions. Add
SmartThings later only when the product needs cloud control or device-specific
capabilities that the local TV reports reliably.

## Validation checklist

1. Pair the local WebSocket remote and verify D-pad commands.
2. Connect a live device to each HDMI port; issue each `KEY_HDMI*` key and
   verify the displayed source after two seconds.
3. Test `KEY_TV` only if broadcast/cable configuration exists.
4. Request installed apps and compare returned names/IDs against the TV's app
   screen.
5. Launch two installed apps using their returned IDs; repeat after a TV wake.
6. Record firmware, key/event results, and app IDs as local test data only.
   Never commit device IPs, MACs, pairing tokens, account data, or deep links.

## Sources

- [Samsung Korea: target TV specifications and support](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/)
- [Samsung: physical source switching guidance](https://www.samsung.com/in/support/tv-audio-video/connect-external-video-devices-and-switch-input-sources-on-a-samsung-smart-tv/)
- [Samsung: Smart View application launch by TV app ID](https://developer.samsung.com/smarttv/develop/extension-libraries/smart-view-sdk/android-sender-app.html)
- [Home Assistant: Samsung TV input key compatibility](https://www.home-assistant.io/integrations/samsungtv/)
- [samsung-tv-ws-api: installed-app and launch WebSocket events](https://github.com/xchwarze/samsung-tv-ws-api/blob/master/samsungtvws/remote.py)
- [SamsungTV Smart: local input-source-control compatibility caveat](https://github.com/TheFab21/ha-samsungtv-smart)

The WebSocket source/app frames are community interoperability findings, not a
public Samsung consumer remote specification. Validate them against the target
TV before claiming compatibility.

## Target model note

Model name: `KU75UA8090FXKR`.
