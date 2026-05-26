# iOS Companion App

## Current stage

The repository contains a native SwiftUI companion app at:

- `ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj`

`SEC-011` is now in place:

- a native SwiftUI project exists
- the app stores a backend server profile locally
- the app can test connectivity against `/api/v1/health` and `/api/v1/server-info`
- the backend is explicitly treated as the source of truth

`SEC-012` is now in place:

- the app can list items from the backend
- the app can open an item detail screen backed by `/api/v1/items/:id`
- the app can create a basic item record through `/api/v1/items`

`SEC-013` is now in place:

- the app can select images and videos from the photo library
- the app uploads media to `/api/v1/items/:id/media`
- the item detail screen refreshes backend media metadata after upload
- visible upload progress is shown while files are sent

`SEC-015` is now in place:

- the app has a `TagScannerService` abstraction
- the app includes a backend-backed tag resolution screen
- the scanner path is explicitly present but honestly reports that live scanning is not implemented yet

The Codex-backed photo-analysis slice is now partially in place:

- the create-item flow can analyze a selected photo through the backend
- the backend returns Codex-powered field suggestions
- the create sheet can prefill the item card before save

Richer item editing remains the next section.

## What the app does now

The current app focuses on local-network connection setup:

- edit a stored profile name
- edit a stored backend base URL
- save the profile in `UserDefaults`
- test the connection against the backend JSON API
- show backend health counts and runtime details after a successful test
- browse the backend item list
- open item detail screens
- create a basic item record from iOS
- select one photo during item creation and let the backend prefill fields
- upload one or more images/videos from the photo library to an item
- show uploaded media entries after success
- resolve backend tags manually from iOS
- expose a future scanner boundary without pretending NFC already works

The current Swift files are organized under:

- `Features/Connection/`
- `Features/Items/`
- `Networking/`
- `Models/`
- `Services/`

## Build and run

1. Open `ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj` in Xcode.
2. Choose an iPhone simulator or a physical iPhone/iPad.
3. Build and run the app.

Backend startup for local testing:

```bash
cargo run --release
```

If you only need same-Mac simulator testing, `cargo run -- serve` is still enough.

## Base URL configuration

Recommended examples:

- Simulator on the same Mac as the backend:
  `http://127.0.0.1:8787`
- Physical iPhone/iPad on the same LAN:
  `http://192.168.x.x:8787`

The app does not assume `localhost` on device. Manual URL entry is the current expected path.

## Local network and transport settings

The Xcode project now includes:

- `NSLocalNetworkUsageDescription`
- `NSAppTransportSecurity > NSAllowsLocalNetworking = YES`

That keeps the MVP focused on trusted local-network HTTP connections without introducing a heavier auth or cloud setup yet.

## Connection test behavior

The app runs:

- `GET /api/v1/health`
- `GET /api/v1/server-info`

Displayed information:

- health status
- item/location/trip counts
- backend version
- bind URL, local URL, and optional LAN URL
- backend data directory path

## Current limitations

- no edit-item flow on iOS yet
- no tag-scanning integration yet
- no Bonjour discovery yet
- photo-based autocomplete depends on the backend machine having a working local `codex` CLI login

## Verification note

If full Xcode is installed but not the active developer directory, you can still build with an explicit `DEVELOPER_DIR`, for example:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild \
  -project ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj \
  -scheme MyWardrobeHelperiOS \
  -destination 'id=<device-id>' \
  build
```

If signing is not available in the current shell session, compile-only validation against the iPhoneOS SDK still works with:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild \
  -project ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj \
  -scheme MyWardrobeHelperiOS \
  -destination 'id=<device-id>' \
  CODE_SIGNING_ALLOWED=NO \
  CODE_SIGNING_REQUIRED=NO \
  build
```

When full Xcode is genuinely unavailable, fall back to:

- `plutil -lint ios/.../project.pbxproj`
- `swiftc -typecheck ios/.../*.swift`

## Next steps

- richer item editing on iOS beyond the current basic create flow
- real NFC and/or QR scanning integration behind the existing scanner abstraction
