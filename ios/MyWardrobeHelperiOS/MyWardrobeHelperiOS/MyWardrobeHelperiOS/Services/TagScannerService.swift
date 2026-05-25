import Foundation

enum TagScannerError: LocalizedError {
    case unavailable(String)

    var errorDescription: String? {
        switch self {
        case let .unavailable(message):
            return message
        }
    }
}

protocol TagScannerService {
    var scannerDisplayName: String { get }
    func beginScan() async throws -> String
}

struct UnsupportedTagScannerService: TagScannerService {
    let scannerDisplayName = "Future NFC / QR Scanner"

    func beginScan() async throws -> String {
        throw TagScannerError.unavailable(
            "Live tag scanning is not implemented yet. Use manual tag resolution for now."
        )
    }
}
