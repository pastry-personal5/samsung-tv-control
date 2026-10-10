# Research: Tizen Samsung TV state monitoring

Status: Research result — 2026-10-10

## Scope

This note evaluates how a native macOS remote can learn a Samsung Tizen TV's
playback state. It distinguishes a command accepted by the local remote channel
from an observed TV state.

## Result

The current secure local remote connection is a command channel, not a
documented generic state-monitoring API. It can report whether a key write was
completed, but cannot confirm playback state for arbitrary running TV apps.
The app must therefore retain `Unknown`, `AssumedPlaying`, and `AssumedPaused`
as local command inferences. It must not present those values as TV-observed
state.

For a new or reconnected session, `Unknown` sends `KEY_PLAY`. A confirmed
`KEY_PLAY` write changes the local inference to `AssumedPlaying`; a confirmed
`KEY_PAUSE` write changes it to `AssumedPaused`. Uncertain and unsent writes do
not change it. External remote use, app changes, playback ending, and TV
restarts can invalidate the inference.

## Documented interfaces

| Interface | What it provides | Fit for this macOS remote |
| --- | --- | --- |
| Tizen AVPlay | `getState()` exposes the state of the player owned by a Tizen application. | Not a monitor for arbitrary installed apps or external remote clients. |
| Tizen MediaController | A participating media-controller server can publish playback state, position, metadata, and change notifications to its clients. | Suitable for a TV app the project owns; it does not grant the Mac visibility into built-in or third-party apps. |
| Tizen TVInputDevice | Lets a TV application receive remote key events. | Input only; it does not expose resulting playback state. |
| SmartThings | Device capability status and event subscriptions after account authorization. | Possible optional cloud-backed monitor if the selected TV exposes a playback capability; discover capabilities per device first. |
| Matter Media Playback | A Matter endpoint can report current media state through subscription. | Only applies when the TV exposes a compatible Matter endpoint and the user has commissioned it. |

Tizen's [AVPlay API](https://developer.samsung.com/smarttv/develop/api-references/samsung-product-api-references/avplay-api.html)
documents `getState()` for the player available to the calling TV application.
Its [MediaController API](https://developer.samsung.com/smarttv/develop/api-references/tizen-web-device-api-references/mediacontroller-api.html)
supports playback-state change listeners between participating controller and
server applications. The [TVInputDevice API](https://developer.samsung.com/smarttv/develop/api-references/tizen-web-device-api-references/tvinputdevice-api.html)
documents key-event handling, not playback-state reporting.

Samsung documents SmartThings status and event subscriptions as
capability-specific, with the device's advertised capabilities determining what
can be queried or subscribed to. See [SmartThings support](https://developer.samsung.com/automation/smartthings-support.html).
The [Matter Media Playback cluster](https://developer.smartthings.com/docs/edge-device-drivers/matter/generated/clusters/MediaPlayback.html)
supports state subscription, but only for a commissioned compatible endpoint.

## Current implementation review

The application now handles the local remote path correctly:

- The visual Play/Pause intent resolves immediately before a serialized write.
- `KEY_PLAY` is used from `Unknown` or `AssumedPaused`; `KEY_PAUSE` is used
  from `AssumedPlaying`.
- Only a `Written` transport outcome updates the inferred state.
- Session selection, reconnect, and Wake reset inference to `Unknown`.
- Queued presses resolve after earlier terminal outcomes, avoiding two Play
  commands when the first command has already been written.

No parser should convert an undocumented WebSocket frame into observed playback
state. Unknown frames must remain ignored until a documented, model-tested
schema is available.

## Recommended next work

1. Keep the present local inference as the default behavior and label it
   internally as inferred.
2. Add a `PlaybackStateSource` only with a concrete provider: `LocalInference`,
   `SmartThings`, `Matter`, or a project-owned Tizen MediaController app.
3. For a SmartThings provider, require explicit account authorization, inspect
   the selected device's capabilities, subscribe or poll only supported
   playback attributes, and store observation time and source with the value.
4. For an owned Tizen app, use MediaController playback listeners and an
   authenticated app-to-app bridge. Treat that state as scoped to the owned
   app, not the whole TV.
5. Test each provider on the target TV and firmware. Verify updates caused by
   the physical remote, this Mac app, pause at end of media, app switching,
   standby, reconnect, and revoked authorization.

Do not add cloud credentials, device identifiers, playback metadata, or raw
event payloads to logs or committed fixtures.
