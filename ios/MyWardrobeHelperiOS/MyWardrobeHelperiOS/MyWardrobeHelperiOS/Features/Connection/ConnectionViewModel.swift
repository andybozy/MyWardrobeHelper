import Combine
import Foundation

@MainActor
final class ConnectionViewModel: ObservableObject {
    @Published var profile: ServerProfile
    @Published var healthResponse: HealthResponse?
    @Published var serverInfoResponse: ServerInfoResponse?
    @Published var connectionMessage = "Connection test has not run yet."
    @Published var saveMessage = "The backend remains the source of truth."
    @Published var isTesting = false
    @Published var lastCheckedAt: Date?

    private let profileStore: ServerProfileStore
    private let apiClient: APIClient

    init(profileStore: ServerProfileStore, apiClient: APIClient) {
        self.profileStore = profileStore
        self.apiClient = apiClient
        self.profile = profileStore.load()
    }

    convenience init() {
        self.init(profileStore: ServerProfileStore(), apiClient: APIClient())
    }

    var canTestConnection: Bool {
        !profile.baseURLString.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    func saveProfile() {
        profileStore.save(profile)
        saveMessage = "Saved \(profile.displayName) at \(profile.baseURLString)."
    }

    func testConnection() async {
        guard canTestConnection else {
            connectionMessage = "Enter a backend URL first."
            return
        }

        isTesting = true
        defer { isTesting = false }

        do {
            let baseURL = try apiClient.normalizedBaseURL(from: profile.baseURLString)
            let health = try await apiClient.fetchHealth(baseURL: baseURL)
            let serverInfo = try await apiClient.fetchServerInfo(baseURL: baseURL)

            healthResponse = health
            serverInfoResponse = serverInfo
            lastCheckedAt = Date()
            profile.baseURLString = baseURL.absoluteString.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
            profileStore.save(profile)
            connectionMessage = "Connected to \(serverInfo.application) \(serverInfo.version)."
        } catch {
            healthResponse = nil
            serverInfoResponse = nil
            connectionMessage = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
        }
    }
}
