#!/usr/bin/env python3
"""Test consent on secure port 8002 without printing or saving a pairing token."""

import base64
import hashlib
import ipaddress
import json
import os
import socket
import ssl
import sys
from urllib.parse import quote

CLIENT_NAME = "Samsung TV Remote for macOS"
MAX_FRAME = 16 * 1024
KEYS = (
    ("Up", "KEY_UP"),
    ("Down", "KEY_DOWN"),
    ("Left", "KEY_LEFT"),
    ("Right", "KEY_RIGHT"),
    ("Select", "KEY_ENTER"),
    ("Back", "KEY_RETURN"),
    ("Home", "KEY_HOME"),
    ("Mute", "KEY_MUTE"),
    ("Volume Up", "KEY_VOLUP"),
    ("Volume Down", "KEY_VOLDOWN"),
)


def read_exact(connection, length):
    data = bytearray()
    while len(data) < length:
        chunk = connection.recv(length - len(data))
        if not chunk:
            raise EOFError("connection closed")
        data.extend(chunk)
    return bytes(data)


def send_pong(connection, payload):
    mask = os.urandom(4)
    connection.sendall(
        bytes([0x8A, 0x80 | len(payload)])
        + mask
        + bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
    )


def send_click(connection, key):
    payload = json.dumps(
        {
            "method": "ms.remote.control",
            "params": {
                "Cmd": "Click",
                "DataOfCmd": key,
                "Option": "false",
                "TypeOfRemote": "SendRemoteKey",
            },
        },
        separators=(",", ":"),
    ).encode("ascii")
    mask = os.urandom(4)
    connection.sendall(
        bytes([0x81, 0x80 | 126])
        + len(payload).to_bytes(2, "big")
        + mask
        + bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
    )


def next_frame(connection):
    header = read_exact(connection, 2)
    opcode = header[0] & 0x0F
    if header[0] & 0x80 == 0 or header[1] & 0x80:
        raise ValueError("unexpected frame flags")
    length = header[1] & 0x7F
    if length == 126:
        length = int.from_bytes(read_exact(connection, 2), "big")
    elif length == 127:
        length = int.from_bytes(read_exact(connection, 8), "big")
    if length > MAX_FRAME:
        raise ValueError("oversized frame")
    payload = read_exact(connection, length)
    if opcode == 9:
        send_pong(connection, payload)
        return None
    if opcode == 8:
        raise EOFError("TV closed the channel")
    if opcode != 1:
        return None
    return payload


def main():
    test_keys = sys.argv[1:] == ["--keys"]
    if sys.argv[1:] and not test_keys:
        print("Usage: python3 scripts/probe_pairing_8002.py [--keys]")
        return 2
    raw_address = input("TV IP address: ").strip()
    expected_fingerprint = input("Expected certificate SHA-256 from reachability probe: ").strip().lower()
    try:
        address = ipaddress.ip_address(raw_address)
    except ValueError:
        print("Invalid numeric TV address")
        return 2
    local_ranges = (
        ipaddress.ip_network("10.0.0.0/8"),
        ipaddress.ip_network("172.16.0.0/12"),
        ipaddress.ip_network("192.168.0.0/16"),
        ipaddress.ip_network("169.254.0.0/16"),
        ipaddress.ip_network("fc00::/7"),
        ipaddress.ip_network("fe80::/10"),
    )
    if not any(address in network for network in local_ranges):
        print("A local TV address is required")
        return 2
    if len(expected_fingerprint) != 64 or any(c not in "0123456789abcdef" for c in expected_fingerprint):
        print("The expected SHA-256 fingerprint must contain 64 hex digits")
        return 2

    print("Watch the physical TV for a pairing prompt and approve this client name:", CLIENT_NAME)
    try:
        with socket.create_connection((str(address), 8002), timeout=5) as plain:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
            context.check_hostname = False
            context.verify_mode = ssl.CERT_NONE
            with context.wrap_socket(plain, server_hostname=str(address)) as connection:
                observed = hashlib.sha256(connection.getpeercert(binary_form=True)).hexdigest()
                if observed != expected_fingerprint:
                    print("Certificate changed; no pairing request sent")
                    return 1
                print("Certificate matches the fingerprint supplied for this run")
                connection.settimeout(45)
                key = base64.b64encode(os.urandom(16)).decode("ascii")
                encoded_name = quote(base64.b64encode(CLIENT_NAME.encode()).decode(), safe="")
                path = "/api/v2/channels/samsung.remote.control?name=" + encoded_name
                request = (
                    f"GET {path} HTTP/1.1\r\n"
                    f"Host: {address}:8002\r\n"
                    "Upgrade: websocket\r\nConnection: Upgrade\r\n"
                    f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
                )
                connection.sendall(request.encode("ascii"))
                response = bytearray()
                while b"\r\n\r\n" not in response:
                    if len(response) >= 8192:
                        raise ValueError("oversized handshake")
                    response.extend(read_exact(connection, 1))
                header = response.decode("iso-8859-1")
                if not header.startswith("HTTP/1.1 101 "):
                    print("WebSocket upgrade rejected; HTTP status:", header.split(" ", 2)[1])
                    return 1
                expected_accept = base64.b64encode(
                    hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()
                ).decode()
                if f"sec-websocket-accept: {expected_accept.lower()}" not in header.lower():
                    raise ValueError("invalid WebSocket handshake")
                print("Secure WebSocket: open; waiting up to 45 seconds for TV consent")
                while True:
                    frame = next_frame(connection)
                    if frame is None:
                        continue
                    if len(frame) > MAX_FRAME:
                        raise ValueError("oversized event")
                    event = json.loads(frame)
                    event_type = event.get("event")
                    if event_type == "ms.channel.connect":
                        data = event.get("data") or {}
                        token = data.get("token") if isinstance(data, dict) else None
                        print("Pairing accepted; token present:", isinstance(token, str) and bool(token))
                        print("Token discarded by this diagnostic; the app must pair again to save it in Keychain")
                        if test_keys:
                            results = []
                            for label, key_name in KEYS:
                                choice = input(f"Press Enter to send {label}, or type s to skip: ").strip().lower()
                                if choice == "s":
                                    results.append((label, "skipped"))
                                    continue
                                send_click(connection, key_name)
                                observed = input("Did the physical TV respond as expected? [y/n/u]: ").strip().lower()
                                results.append((label, {"y": "worked", "n": "failed"}.get(observed, "uncertain")))
                            print("Observed key results:")
                            for label, result in results:
                                print(f"  {label}: {result}")
                        return 0
                    if event_type in ("ms.channel.unauthorized", "ms.channel.error"):
                        print("Pairing denied or rejected")
                        return 1
    except (OSError, ssl.SSLError, EOFError, ValueError, json.JSONDecodeError) as error:
        print("Pairing probe failed:", type(error).__name__)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
