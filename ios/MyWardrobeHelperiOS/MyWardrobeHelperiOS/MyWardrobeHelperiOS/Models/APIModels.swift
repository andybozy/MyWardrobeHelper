import Foundation

struct HealthResponse: Decodable, Equatable {
    let status: String
    let itemCount: Int
    let locationCount: Int
    let tripCount: Int

    enum CodingKeys: String, CodingKey {
        case status
        case itemCount = "item_count"
        case locationCount = "location_count"
        case tripCount = "trip_count"
    }
}

struct ServerInfoResponse: Decodable, Equatable {
    let application: String
    let version: String
    let bindURL: String
    let localURL: String
    let lanURL: String?
    let dataDirectory: String
    let databaseFile: String

    enum CodingKeys: String, CodingKey {
        case application
        case version
        case bindURL = "bind_url"
        case localURL = "local_url"
        case lanURL = "lan_url"
        case dataDirectory = "data_dir"
        case databaseFile = "database_file"
    }
}
