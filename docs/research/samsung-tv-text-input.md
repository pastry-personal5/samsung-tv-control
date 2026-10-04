# Text input from a macOS remote

Research date: 2026-10-05

## Recommendation

Implement text input as an **opportunistic local WebSocket feature** on the
already-paired TV connection. It can enter UTF-8 text into a focused **native
Tizen IME field**—for example the TV browser address bar, Settings search,
Wi-Fi password, or a Tizen-native login form.

Do not promise universal typing. The target is a 2021 Korean Tizen TV,
but Samsung does not publish a model-specific guarantee for this undocumented
consumer remote protocol. It must be hardware-tested on this set. The model is
confirmed by Samsung as a Tizen smart TV with SmartThings mobile-app
compatibility. [Official model support page](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/)

## Supported path: native Tizen text field

Prerequisites:

1. The TV is awake, connected to the app's paired local remote WebSocket.
2. The user has navigated to an input control and the **on-screen Tizen
   keyboard is visible**. Do not try to open a text field automatically.
3. The macOS app receives/has observed the IME-start state, where supported,
   and enables its text box.

To enter `안녕하세요`, encode the original UTF-8 text as standard Base64, then
send this remote-control frame:

```json
{
  "method": "ms.remote.control",
  "params": {
    "Cmd": "7JWI64WV7ZWY7IS47JqU",
    "DataOfCmd": "base64",
    "TypeOfRemote": "SendInputString"
  }
}
```

The value in `Cmd` is Base64 of the UTF-8 bytes; it is **not** raw text and it
is not URL encoding. After a completed entry, send:

```json
{
  "method": "ms.remote.control",
  "params": {
    "TypeOfRemote": "SendInputEnd"
  }
}
```

The established interoperability implementation `samsung-tv-ws-api` performs
exactly this UTF-8/Base64 encoding for `SendInputString` and emits
`SendInputEnd` to finish input. It also listens for the TV's IME start/end
events and resets its input session accordingly. [Implementation and message
shape](https://github.com/xchwarze/samsung-tv-ws-api/blob/master/samsungtvws/remote.py)

## Required session behavior

- Maintain one `TextInputSession` per WebSocket connection, rather than a
  permanent “keyboard mode.” The TV may end IME when focus changes.
- On IME start, enable the macOS text field; on IME end, clear it and disable
  Send. If no event arrives on this model, allow an explicit **Send text to
  focused field** action with an explanatory warning.
- Send the supplied string as one `SendInputString` message first. If hardware
  testing exposes message-size or composition problems, split only on Unicode
  scalar boundaries—not UTF-8 byte boundaries—and test Korean, emoji, symbols,
  and ASCII separately.
- End the session with `SendInputEnd` when the user submits, changes TV focus,
  disconnects, or cancels. A normal remote `KEY_ENTER` is a separate user
  action; do not automatically submit forms after inserting text.
- Keep entered content in memory only long enough to send it. Never log text,
  Base64 payloads, or password-form values.

Samsung describes the TV IME as a virtual keyboard and documents standard
keyboard-character support, including letters, numbers, punctuation, and
symbols. [Samsung IME interaction FAQ](https://developer.samsung.com/smarttv/develop/faq/user-interaction.html)

## Important limitation: streaming-app search screens

`SendInputString` does **not** provide a network HID keyboard. Netflix,
YouTube, Apple TV, Prime Video, and similar apps commonly render their own
on-screen letter grid, rather than exposing a native Tizen IME text field.
Direct injection will not fill those grids.

For such a screen, the only local-remote fallback is a per-app keyboard-layout
driver that uses `KEY_UP`, `KEY_DOWN`, `KEY_LEFT`, `KEY_RIGHT`, and `KEY_ENTER`
to move the grid selection. It is inherently fragile: layout, locale,
application version, adverts, and current focus can all change it. Do **not**
ship automatic app-grid text entry in v1. At most, add a clearly experimental,
user-confirmed layout later after testing the target applications on this TV.

This limitation is documented by a current Home Assistant Samsung TV
integration: `SendInputString` works in focused native Tizen fields but not
in streaming apps' custom character grids; the WebSocket remote API exposes
remote keys rather than alphabet/HID events. [Text-input scope and
limitation](https://github.com/TheFab21/ha-samsungtv-smart)

## Product design for the macOS app

- Show a **Text input** panel only when the TV is connected. Prefer enabling it
  on IME-start; otherwise make the capability visibly conditional.
- Copy text from the Mac's focused input field only after the user presses
  Send. Do not monitor the clipboard.
- Mark password entry as sensitive: no persistent history, no analytics, no
  diagnostic logging, and visually mask it on the Mac.
- Provide a concise empty-state message: “Open a TV text field first. Some
  streaming-app search keyboards cannot accept direct text.”
- Keep the ordinary D-pad visible while a text session exists; it lets the user
  recover focus or use a custom app grid manually.

## Hardware test checklist

1. Pair local remote access; confirm normal D-pad commands first.
2. Open the built-in web browser address bar and test ASCII, Korean, mixed
   Korean/English, punctuation, emoji, backspace behavior, and `SendInputEnd`.
3. Test Settings search and a Wi-Fi password field separately; record whether
   the app receives IME-start/end events.
4. Open YouTube and Netflix search (if installed); confirm that direct input is
   rejected/ignored and that D-pad navigation still works.
5. Repeat after a power-off/wake cycle and after reconnecting with the stored
   pairing token.
6. Record TV firmware and exact results, but never commit IP addresses, MAC
   addresses, pairing tokens, typed credentials, or search text.

## Sources

- [Samsung Korea: target TV specifications and support](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/)
- [Samsung: TV keyboard and IME support](https://developer.samsung.com/smarttv/develop/guides/user-interaction/keyboardime.html?device=htv)
- [Samsung: IME character/input guidance](https://developer.samsung.com/smarttv/develop/faq/user-interaction.html)
- [samsung-tv-ws-api: `SendInputString`, Base64 UTF-8, and input-session implementation](https://github.com/xchwarze/samsung-tv-ws-api/blob/master/samsungtvws/remote.py)
- [Home Assistant Samsung TV Smart: verified native-IME versus streaming-grid limitation](https://github.com/TheFab21/ha-samsungtv-smart)

## Target model note

Model name: `KU75UA8090FXKR`.
