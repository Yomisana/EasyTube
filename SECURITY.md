# Security Policy

## Clipboard Privacy
EasyTube never transmits clipboard contents over the network.
Clipboard contents are only read locally for URL detection and are never logged.

## Reporting a Vulnerability
Please report security issues via GitHub Security Advisories.
Do not open public issues for security vulnerabilities.

## Design Guarantees
- All subprocess spawning uses argument arrays to prevent command injection.
- Output filenames are sanitized to prevent path traversal.
- No telemetry or analytics are collected.
