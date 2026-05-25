import Foundation

@MainActor
final class ItemDetailViewModel: ObservableObject {
    @Published var item: WardrobeItem?
    @Published var isLoading = false
    @Published var message = "Item detail has not loaded yet."

    private let apiClient: APIClient

    init(apiClient: APIClient = APIClient()) {
        self.apiClient = apiClient
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
            item = fetchedItem
            message = "Item detail loaded from the backend."
        } catch {
            item = nil
            message = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
        }
    }
}
