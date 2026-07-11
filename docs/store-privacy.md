# Store Privacy and Security Disclosure

## Data handling

The extension is a transport library. It does not collect analytics, tracking
identifiers, advertising identifiers, contacts, location, diagnostics, or user
content for the extension author. Application data is transmitted only to the
WebTransport endpoint selected by the integrating application.

Safe trace events are local, disabled by default, and contain no URLs, host
names, certificates, headers, close reasons, or payloads. The host application
controls any storage or external transmission of those metadata events.

## Google Play Data safety

The extension itself declares no collected or shared data. The integrating app
must separately disclose the application payloads, account information,
analytics, and server behavior that it implements. Network traffic is protected
with TLS 1.3 and certificate verification is enabled in release builds.

## Apple privacy manifest

The supplied manifest declares no tracking, collected data types, tracking
domains, or required-reason APIs. The integrating app must merge its own
practices and those of every other SDK into the final App Store submission.

This document is implementation guidance, not legal advice. Store disclosures
must be reviewed against the final application behavior.
