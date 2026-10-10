# UX information architecture

Status: Active

This document owns where current information and actions live. Use
[UX terms](ux-term.md) for names and the [GUI specification](ux-gui.md) for visual
and interaction details. Discovery, pairing, and Wake improvements under
consideration are in the [research note](research/discovery-pairing-wake-ux.md).

## Window map

| Window | Region | Contents |
| --- | --- | --- |
| Main window | Sidebar | Icon destinations in order: Power, Remote, Sources, Apps, Text Input. Settings button at the bottom. |
| Main window | Main Pane | One titled primary view above the Messages split bar and Global Messages Pane. Remote View opens by default. |
| Settings Window | Settings Sidebar | Discovery, then Wake on LAN. |
| Settings Window | Settings Main Pane | The selected settings page, scrolled independently from the main window. |

The main window opens to Remote View, including on first launch and when no TV
is saved. Settings opens on request in its own window. Changing a primary view
does not change the Selected TV or create another TV session. The Global
Messages Pane stays visible under every primary view and receives user-relevant
messages from both windows.

## Primary views

| View | Current purpose |
| --- | --- |
| Remote View | A single visual remote image with power, direction, navigation, playback, mute, and volume regions. |
| Power View | Power Controls, then Wake Steps. Controls include Wake, Power Toggle, retry, and cancel actions when applicable. |
| Sources View | A named placeholder; source selection is not implemented. |
| Apps View | A named placeholder; app browsing and launching are not implemented. |
| Text Input View | A named placeholder; TV text entry is not implemented. |

Remote View contains no separate “Keyboard controls” section or duplicate action
buttons. Its existing mapped keyboard shortcuts remain available without visible
shortcut chrome.

## Settings ownership

| Page | Current content |
| --- | --- |
| Discovery | TV List card for Saved TVs and unsaved candidates; Discovery and Pairing card with Discover TVs and manual host entry; Guidance card. |
| Wake on LAN | Wake Configuration card with wired and Wi-Fi MAC fields and Wired, Wi-Fi, or Disabled choice; Guidance card. |

The Discovery page keeps Saved TVs separate from unsaved candidates. A Saved TV
row can be selected; the selected row offers Check TV, Re-pair after a matching
check, Retry Connection, and Forget Selected TV. Retry reconnects that selected
TV; Forget removes that row's saved record and pairing credentials. A candidate
row can be staged, checked, and then paired. Manual entry uses the same check
and pair path. Selecting a candidate never pairs it automatically. A successful
pair saves and selects that TV. A Saved TV can remain selected while disconnected.

Wake settings belong to the Selected TV. The user supplies the TV's interface
MAC and chooses one active interface or Disabled. Wake setup is optional for
normal paired control; Power View owns sending a Wake request and showing its
progress.

## Feedback placement

Short task status and guidance stay in the relevant Settings page or Power View.
The Global Messages Pane carries durable session feedback about setup,
connection, Wake, and remote requests. It does not log routine Settings Window
open/close events or Debug-level diagnostics. Remote View has no persistent pairing, connection, or
aggregate status banner. Disabled image regions are dimmed; Power View and
Settings present the available recovery actions.

The [GUI specification](ux-gui.md) describes control states and message
behavior. This architecture does not assign future source, app, or text-input
capabilities to the current placeholders.
