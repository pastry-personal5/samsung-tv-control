# Control and monitoring design review

Status: Archived

The repository currently runs a Hello World binary. This review assesses the
planned [software architecture](initial-software-architecture.md), [repository boundaries](planned-repository-architecture.md),
and [GUI](initial-ux-gui.md). It does not claim a working TV remote.

| Finding | Effect on control or monitoring | Design correction |
| --- | --- | --- |
| The dispatcher and session both appeared to own a command queue. | Ordering, backpressure, and cancellation had no single owner. | The application coordinator owns the only bounded queue and request IDs; the Samsung adapter owns socket I/O and serial writes. |
| The ViewModel appeared to own the visible TV selection independently of application state. | A failed or late selection could target a different TV than the one shown. | The application owns the Selected TV and generation; the ViewModel projects the accepted selection. |
| A socket write was easy to confuse with a TV state change. | Power, source, mute, and volume could be shown as successful without evidence. | Request outcomes and observed TV state are separate; only fresh, sourced TV observations populate state readings. |
| Subscription restart and bounded events lacked a complete recovery rule. | A pending indicator or terminal result could be lost. | Atomic snapshot plus event revision, gap resync, and an acknowledged bounded result journal; admission applies backpressure when results cannot be retained. |
| A slider had no matching exact-level application command or trustworthy current value. | Step keys could masquerade as exact control. | Add typed `SetVolumeLevel`; leave the visible slider disabled until exact write support and numeric readback are verified. |
| The GUI did not define a shared message path from Settings. | A user could miss a Pairing or discovery result while in the main window. | A session-only presentation feed drives the Global Messages Pane; a separate Activity View displays structured outcomes and Connection events. |
| Empty, disconnected, rejected, and uncertain states were underspecified. | Controls could invite actions that cannot be sent or claim success too early. | Keep the layout stable, disable unavailable controls with reasons, show contextual recovery, and use distinct pending, sent, not-sent, and uncertain copy. |

These corrections preserve Clean Architecture: domain types express device
concepts; application use cases own command policy and monitoring state;
infrastructure adapters translate to sockets, discovery, and storage;
presentation maps typed outcomes to controls and user-facing messages. The
composition root wires these parts. No inner layer imports Iced or Samsung
wire frames.

## Implementation checks

- With fake transports, verify admission order, queue-full rejection, exactly
  one terminal result per accepted request, cancellation before write, and
  uncertain partial writes without automatic replay.
- Detach and reattach an observer during active requests. Verify snapshot and
  revision resynchronization, no duplicate user messages, and no permanent
  pending indicator.
- Switch TVs during discovery, wake, connection, or a write. Verify that old
  results keep their original TV association and cannot alter current status.
- Reject a TV selection and verify that the checked row and control target
  remain on the previously accepted Selected TV.
- Verify that a sent command never changes a displayed TV reading without a
  fresh TV observation. Exercise a stale numeric volume reading and an
  unsupported exact-level control.
- Open Settings and emit pairing, discovery, permission, and failure results.
  Verify that all user-relevant messages reach the Global Messages Pane in
  order and the Activity View lists request and Connection events separately.
- Verify both lower regions at small window sizes, splitter keyboard access,
  bottom-follow behavior, and preserved scroll position while reading history.

Hardware checks remain necessary for endpoint behavior, trust, individual
keys, exact volume support, source/app events, text input, discovery, and wake.
Wake is placed in the Remote View's persistent Wake Steps and power-control
group; its configuration lives in TV settings. See the
[information architecture](planned-information-architecture.md).
