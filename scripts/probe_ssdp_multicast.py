#!/usr/bin/env python3
"""Send one SSDP search from a macOS interface without saving TV details."""

import argparse
import ipaddress
import math
import socket
import subprocess
import sys
import time


SSDP_GROUP = "239.255.255.250"
SSDP_PORT = 1900
MAX_RESPONSE_BYTES = 2048
SEARCH = (
    b"M-SEARCH * HTTP/1.1\r\n"
    b"HOST: 239.255.255.250:1900\r\n"
    b'MAN: "ssdp:discover"\r\n'
    b"MX: 2\r\n"
    b"ST: ssdp:all\r\n\r\n"
)


def interface_ipv4(interface):
    result = subprocess.run(
        ["ipconfig", "getifaddr", interface],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise ValueError("The selected interface has no IPv4 address.")
    address = result.stdout.strip()
    try:
        return str(ipaddress.IPv4Address(address))
    except ipaddress.AddressValueError as error:
        raise ValueError("The selected interface has no valid IPv4 address.") from error


def is_samsung_response(data):
    if len(data) > MAX_RESPONSE_BYTES:
        return False
    try:
        lines = data.decode("utf-8").split("\r\n")
    except UnicodeDecodeError:
        return False
    if not lines or not (
        lines[0] == "HTTP/1.1 200" or lines[0].startswith("HTTP/1.1 200 ")
    ):
        return False
    return any(
        line.partition(":")[0].lower() in {"server", "st", "usn"}
        and "samsung" in line.partition(":")[2].lower()
        for line in lines[1:]
    )


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--interface", default="en9", help="macOS interface to send from (default: en9)"
    )
    parser.add_argument(
        "--seconds", type=float, default=5, help="reply window, 1 to 30 seconds (default: 5)"
    )
    parser.add_argument(
        "--show-addresses",
        action="store_true",
        help="print responder IP addresses locally; do not paste them into an issue",
    )
    args = parser.parse_args(argv)
    if not math.isfinite(args.seconds) or not 1 <= args.seconds <= 30:
        parser.error("--seconds must be between 1 and 30")
    if not args.interface.isalnum():
        parser.error("--interface must contain only letters and digits")
    try:
        address = interface_ipv4(args.interface)
    except (OSError, ValueError) as error:
        print(f"Interface check failed: {error}")
        return 2

    print(f"Sending one SSDP M-SEARCH from {args.interface} (IPv4 present).")
    responses = 0
    samsung_peers = set()
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as connection:
            connection.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_IF, socket.inet_aton(address))
            connection.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 2)
            connection.bind((address, 0))
            connection.sendto(SEARCH, (SSDP_GROUP, SSDP_PORT))
            print("Multicast send: succeeded")
            deadline = time.monotonic() + args.seconds
            while True:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    break
                connection.settimeout(remaining)
                try:
                    data, peer = connection.recvfrom(MAX_RESPONSE_BYTES + 1)
                except socket.timeout:
                    break
                responses += 1
                if is_samsung_response(data):
                    samsung_peers.add(peer[0])
    except OSError as error:
        print(f"Socket failed: {type(error).__name__}, errno {error.errno}: {error.strerror}")
        return 1

    print(f"SSDP datagrams received: {responses}")
    print(f"Unconfirmed Samsung-like responders: {len(samsung_peers)}")
    if args.show_addresses:
        for peer in sorted(samsung_peers, key=ipaddress.IPv4Address):
            print(f"Unconfirmed responder address: {peer}")
    if not responses:
        print("No replies arrived. This alone does not identify a permission or TV failure.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
