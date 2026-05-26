import Combine
import Foundation

@MainActor
final class TagsViewModel: ObservableObject {
    @Published var tagType = "nfc"
    @Published var externalIdentifier = ""
    @Published var isResolving = false
    @Published var statusMessage = "Resolve a backend tag manually or use the scanner placeholder."
    @Published var resolvedTag: ResolvedPhysicalTagResponse?

    private let apiClient: APIClient
    private let scannerService: TagScannerService

    init(apiClient: APIClient, scannerService: TagScannerService) {
        self.apiClient = apiClient
        self.scannerService = scannerService
    }

    convenience init() {
        self.init(
            apiClient: APIClient(),
            scannerService: UnsupportedTagScannerService()
        )
    }

    var scannerDisplayName: String {
        scannerService.scannerDisplayName
    }

    func resolveTag(baseURLString: String) async {
        let trimmedIdentifier = externalIdentifier.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedIdentifier.isEmpty else {
            statusMessage = "Enter a tag identifier first."
            resolvedTag = nil
            return
        }

        isResolving = true
        defer { isResolving = false }

        do {
            let baseURL = try apiClient.normalizedBaseURL(from: baseURLString)
            let resolved = try await apiClient.resolvePhysicalTag(
                tagType: tagType,
                externalIdentifier: trimmedIdentifier,
                baseURL: baseURL
            )
            resolvedTag = resolved
            statusMessage = "Resolved \(resolved.tag.boundEntityType) tag from the backend."
        } catch {
            resolvedTag = nil
            statusMessage = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
        }
    }

    func beginScannerPlaceholder() async {
        do {
            let scannedIdentifier = try await scannerService.beginScan()
            externalIdentifier = scannedIdentifier
            statusMessage = "Received a scanned identifier placeholder."
        } catch {
            statusMessage = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
        }
    }
}
