# P1-M7: In-Memory TV Selection

Status: Complete

## Approach

Extend the small application state introduced in P1-M6 with an optional
selected device and a monotonically increasing selection generation. Selection
is a pure state transition. It does not mean the TV is paired, reachable, or
connected.

Use the P1-M5 `DeviceId` as the local record reference. Keep the selected
device projection deliberately small: safe display text and identity only.
Do not put a network endpoint or credential in the presentation snapshot.

## Module responsibilities

- `application::state` (or the state owner established by P1-M6): own the
  selected identity, display projection, and generation.
- `application` transition: select or clear a supplied known device; selecting
  the current identity does not advance the generation.
- `presentation::iced::view_model`: display the application snapshot and
  continue to derive whether a TV is selected from that snapshot.
- `presentation::iced::app`: compose initial empty state; no persistence or
  device repository is introduced.

Do not add an ID generator or `DeviceRepository` just to manufacture a device
for this milestone. Tests can use explicit synthetic IDs and safe labels.

## Sequencing

1. Add the application selected-device state and generation transitions.
2. Add tests for initial empty state, selection, clear, repeated selection,
   and changing from one identity to another.
3. Project state into Iced and keep the empty state behavior from P1-M6.
4. Run the Cargo gates and verify no endpoint or credential enters the
   presentation surface.

P1-M8 adds pairing and connection facts. A selected identity alone must not
enable remote controls.
