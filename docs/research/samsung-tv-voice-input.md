# Voice input from a macOS remote

Research date: 2026-10-05

## Decision

Do **not** make “send microphone audio from the Mac to Bixby” a v1 feature.
There is no documented Samsung local-remote or SmartThings API that accepts a
macOS audio stream or a natural-language utterance on behalf of this TV.

For the target TV, split voice features into three distinct experiences:

| Experience | Recommendation | What happens |
| --- | --- | --- |
| Trigger the TV's own Bixby/voice interface | Experimental hardware spike | The TV listens through its paired Samsung Smart Remote, not the Mac. |
| Speak commands to the Mac remote | Good future local feature | The Mac transcribes speech, matches a small command grammar, then sends ordinary remote keys. |
| Use Bixby/Alexa/other assistant through SmartThings | Outside the app's v1 scope | The assistant/service receives speech and sends supported device-capability commands. |

Samsung's product page identifies this Korean 2021 TV as Tizen-based, lists
Bixby with Korean support, and says it lacks far-field voice recognition. That
means voice should be assumed to require the Smart Remote's microphone rather
than an always-listening TV microphone. [Official model support
page](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/)

## Experimental path: invoke the TV's voice UI

The community reverse-engineered remote protocol has a `KEY_BT_VOICE` key. A
paired local WebSocket client may emulate holding and releasing the Smart
Remote voice button:

```json
{
  "method": "ms.remote.control",
  "params": {
    "Cmd": "Press",
    "DataOfCmd": "KEY_BT_VOICE",
    "Option": "false",
    "TypeOfRemote": "SendRemoteKey"
  }
}
```

When the user releases the Mac control, send the same frame with
`"Cmd": "Release"`.

This may cause the TV's voice application UI to appear, but it does **not**
carry audio from macOS. The TV will only recognize an utterance if its own
compatible microphone path is active. A community implementation calls this
key a voice-recognition activation and observes `ms.voiceApp.*` lifecycle
events, but it is not Samsung's supported public consumer API. [Observed
`KEY_BT_VOICE` protocol behavior](https://github.com/roberodin/ha-samsungtv-custom/blob/master/custom_components/samsungtv_custom/samsungctl_080b/remote_websocket.py)

Implementation rules if this spike is attempted:

- Make it an opt-in **Use TV voice remote** button, disabled by default.
- Pair through the normal local remote consent flow first.
- Send `Press` only while the user holds the button; always attempt `Release`
  on pointer/key-up, cancellation, disconnect, and app shutdown.
- Watch for `ms.voiceApp.*` events for UI feedback but never infer a recognized
  command or transcript from them.
- Never claim support until this exact model, Korean language configuration,
  and installed firmware have passed the hardware test below.

## What is not available to the macOS app

Samsung's on-TV `VoiceControl` and `VoiceInteraction` APIs are for a Tizen app
*running on the TV* to register commands or react to the TV assistant. They do
not define a network API for a Mac to upload audio or invoke Bixby with a text
utterance. [Samsung VoiceControl API](https://developer.samsung.com/smarttv/develop/api-references/tizen-web-device-api-references/voicecontrol-api.html)
and [Samsung VoiceInteraction API](https://developer.samsung.com/smarttv/develop/api-references/samsung-product-api-references/voiceinteraction-api.html)

Likewise, SmartThings exposes device capability commands. Context7's current
SmartThings Core SDK documentation describes `executeCommand` as sending a
specific device command; it does not expose voice-audio or utterance submission.
Samsung instead documents Bixby/SmartThings as an assistant that discovers a
device's capabilities and acts on recognized requests. [SmartThings and Bixby
overview](https://developer.smartthings.com/docs/advanced/working-with-bixby/)

Therefore, do not:

- send a Mac microphone recording as WebSocket JSON/binary based on guessed
  protocol fields;
- advertise `KEY_BT_VOICE` as Mac voice input;
- use a cloud speech-recognition provider without explicit user consent and a
  separately reviewed privacy design;
- attempt to create a TV-side Tizen application merely to access the on-device
  voice APIs; that is a materially different product architecture.

## Recommended future feature: Mac speech-to-command

This is the useful voice feature the app can safely own. It is **not** Samsung
Bixby integration:

```text
User holds Mac push-to-talk
  -> Mac local speech recognition
  -> strict command matcher
  -> existing RemoteKey or Wake-TV action
  -> TV local WebSocket / WoL
```

Keep the initial command vocabulary closed and confirmation-oriented:

- “volume up/down,” “mute”
- “left/right/up/down,” “select,” “home,” “back”
- “turn off TV” (show confirmation or require a longer hold)
- “wake TV” (only when a saved MAC/WoL capability exists)

Do not map arbitrary transcribed text to TV actions. Show the recognized
command before sending it where latency permits, and make every action use the
same typed command layer as mouse/keyboard controls. This makes the feature
testable without microphone hardware and prevents a speech-recognition error
from issuing a destructive power command.

## Hardware spike

1. Confirm the model has a paired Samsung Smart Remote with a microphone and
   that Bixby works from the physical remote in Korean.
2. Connect the macOS app using the paired WebSocket remote token.
3. Send `KEY_BT_VOICE` `Press`; record whether a voice UI/recording indicator
   appears and whether an `ms.voiceApp.*` event arrives.
4. Speak only into the **physical remote**, then send `Release`; record whether
   the recognized command executes.
5. Repeat after reboot and across Bixby language/settings changes.
6. Verify that no sound from the Mac microphone affects the TV; this confirms
   the intended boundary before the UI is designed.

Record firmware, remote type, result, and event names. Never log speech audio,
recognized text, household names, device IDs, IP/MAC addresses, or pairing
tokens.

## Sources

- [Samsung Korea: target TV specifications and support](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/)
- [Samsung: on-TV VoiceControl API](https://developer.samsung.com/smarttv/develop/api-references/tizen-web-device-api-references/voicecontrol-api.html)
- [Samsung: on-TV VoiceInteraction API](https://developer.samsung.com/smarttv/develop/api-references/samsung-product-api-references/voiceinteraction-api.html)
- [Samsung: SmartThings capability control through Bixby](https://developer.smartthings.com/docs/advanced/working-with-bixby/)
- [Community interoperability evidence for `KEY_BT_VOICE`](https://github.com/roberodin/ha-samsungtv-custom/blob/master/custom_components/samsungtv_custom/samsungctl_080b/remote_websocket.py)

Samsung sources establish the model capability and the supported on-TV/cloud
voice boundaries. The `KEY_BT_VOICE` wire behavior is reverse-engineered,
unsupported, and must be validated on hardware before product use.

## Target model note

Model name: `KU75UA8090FXKR`.
