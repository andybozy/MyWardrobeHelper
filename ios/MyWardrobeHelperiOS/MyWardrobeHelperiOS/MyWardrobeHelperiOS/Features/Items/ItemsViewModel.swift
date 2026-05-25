import Foundation

@MainActor
final class ItemsViewModel: ObservableObject {
    @Published var items: [WardrobeItem] = []
    @Published var isLoading = false
    @Published var isCreating = false
    @Published var statusMessage = "Load items from the backend after saving a valid server profile."
    @Published var selectedItem: WardrobeItem?

    private let apiClient: APIClient

    init(apiClient: APIClient = APIClient()) {
        self.apiClient = apiClient
    }

    func loadItems(baseURLString: String) async {
        guard !baseURLString.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            items = []
            statusMessage = "Enter and save a backend URL in the Connection tab first."
            return
        }

        isLoading = true
        defer { isLoading = false }

        do {
            let baseURL = try apiClient.normalizedBaseURL(from: baseURLString)
            let fetchedItems = try await apiClient.fetchItems(baseURL: baseURL)
            items = fetchedItems
            statusMessage = fetchedItems.isEmpty
                ? "Connected to the backend. No items exist yet."
                : "Loaded \(fetchedItems.count) item(s) from the backend."
        } catch {
            items = []
            statusMessage = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
        }
    }

    func createItem(
        name: String,
        category: String,
        brand: String,
        baseURLString: String
    ) async -> Bool {
        let trimmedName = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedName.isEmpty else {
            statusMessage = "Item name is required."
            return false
        }

        isCreating = true
        defer { isCreating = false }

        do {
            let baseURL = try apiClient.normalizedBaseURL(from: baseURLString)
            let createdItem = try await apiClient.createItem(
                CreateItemRequest(
                    name: trimmedName,
                    category: emptyToNil(category),
                    brand: emptyToNil(brand)
                ),
                baseURL: baseURL
            )
            selectedItem = createdItem
            await loadItems(baseURLString: baseURLString)
            statusMessage = "Created \(createdItem.name) from the backend API."
            return true
        } catch {
            statusMessage = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
            return false
        }
    }

    private func emptyToNil(_ value: String) -> String? {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }
}
