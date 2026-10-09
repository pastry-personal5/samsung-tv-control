# Lowest-latency Wake-on-LAN research

Status: Archived

Research date: 2026-10-05. Scope: a native macOS remote on the same local
network as the target Korean Samsung TV. This is a design and test
plan, **not a measured wake-time claim**: no access to this physical TV or its
firmware was available during research.

## The fastest defensible strategy

Send a correctly addressed WoL magic packet **immediately on the user's power-on
action**. In parallel, if Samsung Consumer IP Control has been enabled, paired,
and validated on this TV, send an **explicit IP `powerOn` command**. Begin
checking for actual readiness immediately; do not put discovery, ping, a fixed
sleep, or a packet-retry timer in front of the first packet.

Samsung's [Consumer IP Control Worksheet](https://image-us.samsung.com/SamsungUS/samsungbusiness/tv-ci-resources/Samsung-IP-Control.pdf)
explains why both paths matter. For roughly the first minute after shutdown,
the TV *may* still have an active IP server and accept IP power-on. After about
a minute, the server goes offline and WoL is required. Samsung explicitly
recommends sending **both** IP Power ON and WoL because the state is uncertain.
These are approximate stages, not a timer to wait for. The timing and IP
behavior have not been measured on the target TV.

This minimizes **app-added** delay; it cannot remove the TV's physical standby
exit, boot, panel, and remote-service startup time. No credible published
latency measurement for this exact SKU was found.

| TV state | Critical action | Why |
| --- | --- | --- |
| Just turned off; IP service may still exist | Send WoL now **and** explicit paired IP `powerOn` now | Either can win; waiting to classify state only loses time. |
| Deep standby; IP service gone | Send WoL now | IP commands cannot reach an offline server. |
| Already on | Do not send a toggle | An explicit `powerOn` is intended to leave it on; verify this behavior on the target. `KEY_POWER` could turn it off. |

The [model's Samsung specification](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/)
confirms Tizen, a wired Ethernet port, Wi-Fi 5, and SmartThings compatibility.
It does **not** explicitly certify WoL or WoWLAN for this exact SKU. The listed
0.3 W standby consumption does not establish the consumption or wake behavior
with network wake enabled. Samsung's general IP-control guidance is strong
evidence for the implementation path, but exact-model compatibility remains a
hardware-test item.

## One-time setup that keeps the click path fast

1. Connect the TV by Ethernet for the first test and leave AC power connected.
   Wired WoL is the most straightforward packet path; whether it is *faster*
   than Wi-Fi wake on this unit must be measured. Mac on Wi-Fi and TV on
   Ethernet is fine if the access point and switch bridge them into the same
   broadcast domain. Guest Wi-Fi, client isolation, and routed VLANs may block
   the broadcast.
2. While the TV is on, enable **Power On With Mobile** under **Settings →
   General → Network → Expert Settings** (the path Samsung gives for 2021 and
   older sets). Enable **IP Remote** there too *only if* testing the parallel
   Consumer IP Control path. Korean labels and exact firmware menus may differ;
   verify on the set. [Samsung worksheet](https://image-us.samsung.com/SamsungUS/samsungbusiness/tv-ci-resources/Samsung-IP-Control.pdf),
   [Samsung's 2017–2021 mobile-power instructions](https://www.samsung.com/au/support/tv-audio-video/power-tv-with-the-smartthings-app/)
3. Record **Settings → Support → About This TV → Wired MAC Address** for wired
   wake, or **Wireless MAC Address** for Wi-Fi testing. Samsung calls out a
   wrong MAC as a common failure. Do not substitute the TV's IP or a `wifiMac`
   field for the wired MAC. Store the chosen interface/MAC pairing in the
   app's device record. [Samsung worksheet](https://image-us.samsung.com/SamsungUS/samsungbusiness/tv-ci-resources/Samsung-IP-Control.pdf)
4. Pair Consumer IP Control with the TV **on**, accept its on-screen prompt,
   and store its token securely if this optional path proves to work. This is
   separate from any WebSocket remote pairing. Arrange a DHCP reservation if
   using a stored TV IP; this avoids stale-address connection failures but
   is **not** what delivers the magic packet. Samsung recommends a static IP
   when its preferred discovery approach is unavailable.
   [Samsung worksheet](https://image-us.samsung.com/SamsungUS/samsungbusiness/tv-ci-resources/Samsung-IP-Control.pdf)
5. During macOS onboarding, trigger and obtain Local Network permission, then
   test a wake packet while the user can resolve a permission denial. On
   macOS 15+, Apple lists outgoing UDP broadcast as a permission-gated local
   network operation. A first-use system prompt on the power button would be
   avoidable latency. Recheck the route and broadcast address whenever the
   Mac's network changes. [Apple TN3179](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy)

Do not expose every possible Eco or HDMI/CEC setting as a mandatory toggle.
Samsung says they **may** affect IP-control timing/reliability; change one at a
time only if a repeatable problem occurs. The model specification says this
SKU has **no far-field voice recognition**, so Samsung's optional Bixby
standby-server trick is not a suitable recommendation here.
[Samsung worksheet](https://image-us.samsung.com/SamsungUS/samsungbusiness/tv-ci-resources/Samsung-IP-Control.pdf),
[model specification](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/)

## Magic packet and the macOS path

The standard packet is 102 bytes, not a Samsung-specific command:

```text
FF FF FF FF FF FF || (six raw bytes of the selected TV MAC repeated 16 times)
```

The [original AMD Magic Packet paper](https://www.amd.com/content/dam/amd/en/documents/archived-tech-docs/white-papers/20213.pdf)
defines this byte pattern. UDP port **9** is a common transport convention,
not part of the magic sequence; [Home Assistant's WoL documentation](https://www.home-assistant.io/integrations/wake_on_lan/)
uses broadcast UDP/9 in its example. WoL is one-way: successful `sendto` means
the Mac queued/sent a datagram, **not** that the TV received it or powered on.

For a local Rust/macOS implementation:

- Build and validate the 102-byte payload once after storing the MAC; never do
  MAC discovery or parsing on the button's critical path.
- Choose the egress interface that reaches the TV's LAN. Compute that
  interface's directed broadcast as `(IPv4 address & subnet mask) |
  (~subnet mask)` and send to it on UDP/9 with `SO_BROADCAST`. A limited
  broadcast to `255.255.255.255` can be an alternate test, but on a multihomed
  Mac it may leave via the wrong interface. Apple DTS specifically calls out
  `IP_BOUND_IF` when broadcast goes to the wrong interface.
  [Apple DTS discussion](https://developer.apple.com/forums/thread/661861)
- Use a BSD UDP socket (Rust `std::net::UdpSocket` can enable broadcast);
  Apple's Network framework does **not** support UDP broadcast. If necessary
  on a multihomed Mac, bind the socket to the selected Darwin interface with
  `IP_BOUND_IF`. Validate this at the network boundary; do not infer success
  from an in-memory test. [Apple TN3151](https://developer.apple.com/documentation/technotes/tn3151-choosing-the-right-networking-api),
  [Apple DTS discussion](https://developer.apple.com/forums/thread/661861)
- If the macOS app is sandboxed, give it outgoing-network entitlement. Apple's
  UDP entitlement rules govern **data flow** as well as connection setup; add
  the incoming entitlement too if the app receives UDP discovery packets.
  Separately, Local Network permission applies regardless of networking API.
  [Apple client entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.network.client),
  [Apple server entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.network.server),
  [Apple TN3179](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy)

Broadcast is preferable to sending the packet to the TV's last-known unicast
IP: when the TV is asleep, a router's ARP/neighbor mapping can expire, so an
IP-unicast frame may never reach the TV's MAC. The AMD paper describes this
failure mode and broadcast delivery. A DHCP reservation does not by itself
solve sleeping-target ARP expiry.
[AMD Magic Packet paper](https://www.amd.com/content/dam/amd/en/documents/archived-tech-docs/white-papers/20213.pdf)

## Parallel IP power-on: useful, but a separate capability

Samsung's worksheet says to send IP Power ON alongside WoL, but does not
publish a complete consumer-TV command schema. A
[community-tested Consumer IP Control reference](https://github.com/TheFab21/ha-samsungtv-smart/blob/master/IP_Control_Protocol_Reference.md)
reports HTTPS JSON-RPC on port `1516` for 2020+ TVs, an on-screen
`createAccessToken` pairing flow, and `powerControl` with
`"power":"powerOn"`. Its measurements are on other Samsung consumer TVs,
especially Frame models, **not the target TV**. Treat this as an
experimental, capability-gated path until tested on the target. Samsung's
commercial-display IP command list should not be assumed to be a consumer-TV
compatibility guarantee.

Illustrative request shape **after pairing** (not yet validated on this SKU):

```json
{"jsonrpc":"2.0","id":1,"method":"powerControl","params":{"AccessToken":"<stored token>","power":"powerOn"}}
```

Send the IP request concurrently with, **never before**, the first WoL send.
Do not wait for TCP connection failure before WoL. Give an unavailable IP
service a short, bounded timeout so it cannot stall the UI; tune it from
hardware measurements. Do not use WebSocket `KEY_POWER` as the parallel
fallback: it is a **toggle**, not an explicit on command. Even when an IP or
WebSocket server is reachable shortly after power-off, that reachability is
not proof the panel is on. [Samsung worksheet](https://image-us.samsung.com/SamsungUS/samsungbusiness/tv-ci-resources/Samsung-IP-Control.pdf),
[community protocol reference](https://github.com/TheFab21/ha-samsungtv-smart/blob/master/IP_Control_Protocol_Reference.md)

Samsung's [Smart View SDK WoWLAN guide](https://developer.samsung.com/smarttv/develop/extension-libraries/smart-view-sdk/ios-sender-app/enhanced-features.html)
describes an SDK-managed **Wi-Fi** wake path requiring a previously connected
TV and retained MAC. Those are requirements of that SDK workflow, not of a
raw wired magic packet. Its discovery/reconnect timeout behavior is not a
measured physical-wake latency for this TV. Evaluate WoWLAN only after the
wired path and MAC identity are understood.

## Click-path design: no artificial wait

```text
During setup / on network change:
    verify TV MAC and chosen network interface
    precompute magic packet and LAN broadcast address
    obtain macOS Local Network permission
    optionally pair/test Consumer IP Control

On Power On click:
    t0 = monotonic_time()
    dispatch magic packet immediately to selected broadcast:9
    concurrently, if validated and paired, dispatch explicit IP powerOn
    immediately start one readiness probe; retry with bounded, measured backoff
    report "remote ready" only when the paired remote channel accepts control
    stop retries when ready, user cancels, network changes, or deadline expires
```

There is **no fixed 200 ms spacing and no three-packet gate** before
connection attempts. Those values in the earlier note were unsupported as an
optimal latency choice. An extra immediate packet or an early timed retry
*might* improve delivery on a lossy network, but that is a hypothesis to
A/B-test, not a Samsung requirement. Never continue late wake retries after
the user cancels; delayed broadcasts can cause an unwanted power-on.

Keep the app resident if it is a menu-bar remote. Reusing prepared state can
remove process launch, TV discovery, permission prompting, and route lookup
from the click path. The first UDP send should be measured as `t_packet - t0`;
there is no honest basis yet for claiming a particular millisecond number.

## Exact-model validation and latency benchmark

The remaining uncertainty is not the 102-byte packet format; it is whether
this unit/firmware keeps the selected NIC wake-capable in each standby state,
whether the network delivers broadcast, and how long its own startup takes.
Use the following **on the physical target TV**:

1. Record model, firmware version, wired/wireless MAC, TV settings, network
   topology, Mac interface, and whether Consumer IP Control paired. Keep IPs,
   MACs, tokens, and LAN details out of committed logs.
2. Test three conditions separately: wake within 30 seconds of power-off,
   after more than two minutes, and after overnight standby. Unplugging the TV
   is a separate negative control, not a normal WoL case.
3. For each condition compare **WoL only**, **IP `powerOn` only** (if paired),
   and **both dispatched together**. Repeat enough times for a distribution
   (e.g. 20+ per cell), then compare median, p95, and failure rate. Repeat on
   Wi-Fi only if wired results justify it. Randomize order to avoid thermal or
   cache-state bias.
4. Capture `t0` button action, `t1` first UDP packet observed leaving the
   selected Mac interface, `t2` panel visibly on (camera/video measurement),
   and `t3` paired remote command accepted. `t1-t0` is app/network-dispatch
   delay; `t2-t1` is TV/network wake; `t3-t0` is interactive readiness. A TCP
   connection or ping alone is **not** a valid proxy for the lit panel.
5. If wake fails, inspect TV **Support → About This TV → Event Log** for a
   `WOL`/`eMac phy (Magic Packet)` power-on reason after a successful retry.
   Samsung documents that event reason. Use a packet capture on the chosen Mac
   interface to establish that UDP/9 actually left it; `sendto` success alone
   is insufficient. [Samsung event-log guidance](https://www.samsung.com/sa_en/support/tv-audio-video/how-to-troubleshoot-the-samsung-tv-that-keeps-turning-on-by-itself/)

Troubleshoot in this order: wrong **wired vs wireless** MAC; Power On With
Mobile disabled; Mac Local Network permission; wrong egress interface or stale
broadcast address; Wi-Fi/guest isolation or VLAN; switch/AP filtering; standby
Ethernet link down; then firmware/Eco settings. If a packet visibly reaches
the TV's LAN and still never produces a WOL event after deep standby, do not
promise WoL support for this particular firmware. Try the wireless MAC/WoWLAN
path as a separate experiment, or offer a clearly labeled alternative such as
HDMI-CEC hardware if the user wants to broaden beyond WoL. There is no
software-only trick that wakes a set with no standby power or no listening
network interface.

## Evidence and confidence

- **High confidence:** Samsung's general two-stage IP/WoL behavior and
  send-both recommendation; the magic-packet bytes; macOS broadcast and
  privacy requirements. These come from Samsung, AMD, and Apple primary
  documentation linked above.
- **Model-confirmed:** Ethernet, Wi-Fi, Tizen, and no far-field voice
  recognition in the [exact model specification](https://www.samsung.com/sec/support/model/KU75UA8090FXKR/).
- **Not established:** this exact TV's WoL/Consumer IP Control acceptance,
  firmware-specific menus, wired-versus-Wi-Fi winner, best retry cadence,
  and real wake-time distribution. These require the benchmark above.

## Target model note

Model name: `KU75UA8090FXKR`.
