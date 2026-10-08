# P1-M10 Hardware Matrix

Status: In Progress. Record only facts observed on the owner's equipment. Do
not add LAN addresses, certificate fingerprints, device identifiers, tokens,
or raw pairing frames.

| Target | Firmware | Network | Secure endpoint | Pairing | Saved-token reconnect | Discovery | Native bundle | Keys | Latency |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Samsung KU75UA8090FXKR | T-NKLAAKUC-2310.0, BT-S | TV on Ethernet; Mac and TV on the same LAN | Port 8002 reachable; TLS 1.3; metadata HTTP 200 | Owner reports app Pairing working; earlier diagnostic client was also approved | Owner reports successful restart reconnect; agent observed Retry Connection succeed after a transient network failure | Signed bundle found one Samsung-like candidate; manual probes found a responder whose source address matches the saved TV host. Physical identity remains unconfirmed by discovery alone | Owner confirms signed bundle works for Pairing and all ten required keys; permission prompt and denial recovery deferred | Owner reports all ten on-screen keys working: Up, Left, Enter, Right, Down, Back, Home, Mute, Volume Down, and Volume Up | Not measured; owner does not plan a measurement |

The secure endpoint facts come from owner-run Terminal diagnostics. The app
Pairing and key results are owner-reported human verification on 2026-10-08.
The successful run used the signed bundle, as the owner confirmed. The owner
also reports saved-token reconnect and Retry Connection working. The earlier
native connection failure was superseded by this successful run; its cause is
unknown. Do not infer permission recovery or latency from these results.
The owner checked Activity for all ten keys: each said the request was written
to the TV connection and the TV response was unverified. The owner reports all
ten on-screen keys visibly working. The owner also reports M, Home (external
keyboard), Up, Down, Left, Right, minus, Enter, and Escape keyboard shortcuts
working. Shift+= initially failed in a bundle of uncertain age, then worked
after the signed bundle with physical Equal-key handling was rebuilt.
Pairing/Connection labels were not recorded.

## Deferred human verification

The owner deferred the discovery and permission checks, including the first
local-network alert, denial, approval, and retry from the signed bundle. The
owner also deferred failure and switching checks: token rejection and re-pair,
forget, and switching TVs during pending connection work. These remain open
acceptance items; deterministic code checks do not replace them.
The deferral has no revisit date.

## Local-network retest on 2026-10-08

The owner reported granting local-network permission. System Settings showed
Local Network access enabled for the signed app. The Mac's active wired en9
interface was selected by the route lookup. An initial run of **Discover TVs**
reported "Discovery unavailable," **Retry Connection** reported the saved TV
offline or unreachable, and direct multicast and secure-port attempts returned
`No route to host`. After the signed app was restarted, a new discovery found
one unconfirmed Samsung-like candidate. A subsequent **Retry Connection**
reported "Connected to the saved TV." This verifies discovery and saved-token
reconnect on the current network, but does not explain the earlier transient
network failure or verify permission denial and recovery. No TV address,
fingerprint, token, or raw frame was recorded.

For a manual multicast diagnostic on the Mac's wired interface, run
`python3 scripts/probe_ssdp_multicast.py --interface en9`. It prints send and
reply status without device addresses by default.

The owner then ran that diagnostic with pyenv Python on both en0 and en9. Each
interface successfully sent one SSDP search, received 47 datagrams during the
five-second window, and found one Samsung-like responder. Both runs reported
the same source address, which matches the selected saved TV host in local
preferences. This supports the discovery path but does not make an SSDP
advertisement proof of physical TV identity or verify macOS permission denial
and recovery.

## Next observations

- Record the Pairing/Connection labels if the app is opened again.
- The under-150-ms control-to-visible-response target is unverified because no
  latency measurement is planned.
