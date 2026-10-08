#!/usr/bin/env python3
"""Diagnose Samsung secure-port reachability without pairing or logging TV data."""

import hashlib
import ipaddress
import socket
import ssl


def main():
    raw = input("TV IP address (input is not saved): ").strip()
    try:
        address = ipaddress.ip_address(raw)
    except ValueError:
        print("Invalid IP address. Enter the numeric address shown in TV settings.")
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
        print("This script accepts only a local TV address.")
        return 2

    try:
        with socket.create_connection((str(address), 8002), timeout=4) as plain:
            print("Port 8002: reachable")
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
            # This diagnostic connection sends no credentials. Its certificate
            # fingerprint is an observation, never automatic proof of identity.
            context.check_hostname = False
            context.verify_mode = ssl.CERT_NONE
            with context.wrap_socket(plain, server_hostname=str(address)) as secure:
                secure.settimeout(4)
                certificate = secure.getpeercert(binary_form=True)
                print("TLS: negotiated", secure.version())
                print("Certificate SHA-256:", hashlib.sha256(certificate).hexdigest())
                secure.sendall(
                    b"GET /api/v2/ HTTP/1.1\r\n"
                    + b"Host: "
                    + str(address).encode("ascii")
                    + b"\r\nConnection: close\r\n\r\n"
                )
                first_line = secure.recv(1024).split(b"\r\n", 1)[0]
                parts = first_line.split(b" ")
                if len(parts) >= 2 and parts[0].startswith(b"HTTP/"):
                    print("Metadata HTTP status:", parts[1].decode("ascii", "replace"))
                else:
                    print("Metadata HTTP status: no valid HTTP response")
    except (OSError, ssl.SSLError) as error:
        # Never print the supplied address or a response body.
        category = type(error).__name__
        errno = getattr(error, "errno", None)
        print("Probe failed:", category, "errno", errno)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
