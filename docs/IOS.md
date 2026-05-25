# iOS Companion App

## Current stage

The repository contains a native SwiftUI app placeholder at:

- `ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj`

The iOS product direction is active, but the networking and data flows are not implemented yet. Initial functional delivery is tracked in `SEC-011`, `SEC-012`, and `SEC-013`.

## Product direction

The iOS app will:

- connect to the Rust backend over the local network
- use the documented JSON API as its only backend contract
- treat the backend as the source of truth
- support item browsing, practical editing, and image/video upload
- stay ready for future physical tag reading

## Local network model

The MVP app will use a user-configurable base URL such as `http://192.168.1.10:8787`.

The app must not assume `localhost` when running on a physical device.

## Current limitations

- no server profile storage yet
- no API client yet
- no LAN connection testing yet
- no media upload yet
- no tag-reading integration yet

## Build note

Open the Xcode project above and run the placeholder app locally. Device and simulator network flows will be documented once `SEC-011` lands.
