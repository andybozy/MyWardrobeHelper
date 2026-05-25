import Foundation

@MainActor
final class ItemDetailViewModel: ObservableObject {
    @Published var item: WardrobeItem?
    @Published var media: [ItemMediaRecord] = []
    @Published var isLoading = false
    @Published var isUploading = false
    @Published var uploadProgress: Double = 0
    @Published var message = "Item detail has not loaded yet."

    private let apiClient: APIClient
    private let mediaUploadClient: MediaUploadClient

    init(
        apiClient: APIClient = APIClient(),
        mediaUploadClient: MediaUploadClient = MediaUploadClient()
    ) {
        self.apiClient = apiClient
        self.mediaUploadClient = mediaUploadClient
    }

    func loadItem(id: String, baseURLString: String) async {
        guard !baseURLString.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            message = "Save a backend URL in the Connection tab first."
            item = nil
            return
        }

        isLoading = true
        defer { isLoading = false }

        do {
            let baseURL = try apiClient.normalizedBaseURL(from: baseURLString)
            let fetchedItem = try await apiClient.fetchItem(id: id, baseURL: baseURL)
            let fetchedMedia = try await apiClient.fetchItemMedia(itemID: id, baseURL: baseURL)
            item = fetchedItem
            media = fetchedMedia
            message = fetchedMedia.isEmpty
                ? "Item detail loaded from the backend."
                : "Item detail and media loaded from the backend."
        } catch {
            item = nil
            media = []
            message = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
        }
    }

    func uploadMedia(
        itemID: String,
        baseURLString: String,
        uploads: [PendingMediaUpload]
    ) async {
        guard !uploads.isEmpty else {
            message = "Select at least one image or video first."
            return
        }

        isUploading = true
        uploadProgress = 0
        defer {
            isUploading = false
            uploadProgress = 0
        }

        do {
            let baseURL = try apiClient.normalizedBaseURL(from: baseURLString)
            _ = try await mediaUploadClient.uploadItemMedia(
                itemID: itemID,
                uploads: uploads,
                baseURL: baseURL
            ) { [weak self] progress, text in
                self?.uploadProgress = progress
                self?.message = text
            }

            await loadItem(id: itemID, baseURLString: baseURLString)
            message = "Uploaded \(uploads.count) media file(s) to the backend."
        } catch {
            message = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
        }
    }
}
