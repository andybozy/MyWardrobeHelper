import Foundation

struct ServerProfile: Codable, Equatable {
    var displayName: String
    var baseURLString: String

    static let `default` = ServerProfile(
        displayName: "Local Wardrobe Backend",
        baseURLString: "http://127.0.0.1:8787"
    )
}
