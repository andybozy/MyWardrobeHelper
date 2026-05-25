import Foundation

final class ServerProfileStore {
    private let defaults: UserDefaults
    private let storageKey = "server_profile"

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    func load() -> ServerProfile {
        guard
            let data = defaults.data(forKey: storageKey),
            let profile = try? JSONDecoder().decode(ServerProfile.self, from: data)
        else {
            return .default
        }

        return profile
    }

    func save(_ profile: ServerProfile) {
        if let data = try? JSONEncoder().encode(profile) {
            defaults.set(data, forKey: storageKey)
        }
    }
}
