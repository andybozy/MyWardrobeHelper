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

struct WardrobeItem: Codable, Equatable, Identifiable {
    let id: String
    let name: String
    let category: String?
    let subcategory: String?
    let brand: String?
    let size: String?
    let colorPrimary: String?
    let colorSecondary: String?
    let material: String?
    let season: String?
    let formality: String?
    let status: String?
    let currentLocationID: String?
    let notes: String?
    let createdAt: String
    let updatedAt: String

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case category
        case subcategory
        case brand
        case size
        case colorPrimary = "color_primary"
        case colorSecondary = "color_secondary"
        case material
        case season
        case formality
        case status
        case currentLocationID = "current_location_id"
        case notes
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

struct CreateItemRequest: Encodable, Equatable {
    let name: String
    let category: String?
    let subcategory: String?
    let brand: String?
    let size: String?
    let colorPrimary: String?
    let colorSecondary: String?
    let material: String?
    let season: String?
    let formality: String?
    let status: String?
    let currentLocationID: String?
    let notes: String?

    enum CodingKeys: String, CodingKey {
        case name
        case category
        case subcategory
        case brand
        case size
        case colorPrimary = "color_primary"
        case colorSecondary = "color_secondary"
        case material
        case season
        case formality
        case status
        case currentLocationID = "current_location_id"
        case notes
    }
}

struct ItemsListResponse: Decodable, Equatable {
    let items: [WardrobeItem]
}

struct ServerErrorEnvelope: Decodable, Equatable {
    let error: ServerErrorBody
}

struct ServerErrorBody: Decodable, Equatable {
    let code: String
    let message: String
}

struct ItemMediaRecord: Codable, Equatable, Identifiable {
    let id: String
    let itemID: String
    let mediaKind: String
    let relativeFilePath: String
    let originalFilename: String
    let mimeType: String
    let fileSizeBytes: Int
    let durationMS: Int?
    let width: Int?
    let height: Int?
    let caption: String?
    let sortOrder: Int
    let createdAt: String

    enum CodingKeys: String, CodingKey {
        case id
        case itemID = "item_id"
        case mediaKind = "media_kind"
        case relativeFilePath = "relative_file_path"
        case originalFilename = "original_filename"
        case mimeType = "mime_type"
        case fileSizeBytes = "file_size_bytes"
        case durationMS = "duration_ms"
        case width
        case height
        case caption
        case sortOrder = "sort_order"
        case createdAt = "created_at"
    }
}

struct ItemMediaListResponse: Decodable, Equatable {
    let media: [ItemMediaRecord]
}

struct ItemPhotoAnalysisSuggestion: Codable, Equatable {
    let name: String?
    let category: String?
    let subcategory: String?
    let brand: String?
    let size: String?
    let colorPrimary: String?
    let colorSecondary: String?
    let material: String?
    let season: String?
    let formality: String?
    let status: String?
    let notes: String?
    let summary: String
    let warnings: [String]

    enum CodingKeys: String, CodingKey {
        case name
        case category
        case subcategory
        case brand
        case size
        case colorPrimary = "color_primary"
        case colorSecondary = "color_secondary"
        case material
        case season
        case formality
        case status
        case notes
        case summary
        case warnings
    }
}

struct ItemPhotoAnalysisResponse: Decodable, Equatable {
    let suggestion: ItemPhotoAnalysisSuggestion
}

struct PhysicalTagRecord: Codable, Equatable, Identifiable {
    let id: String
    let tagType: String
    let externalIdentifier: String
    let label: String?
    let boundEntityType: String
    let boundEntityID: String
    let notes: String?
    let createdAt: String
    let updatedAt: String

    enum CodingKeys: String, CodingKey {
        case id
        case tagType = "tag_type"
        case externalIdentifier = "external_identifier"
        case label
        case boundEntityType = "bound_entity_type"
        case boundEntityID = "bound_entity_id"
        case notes
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

struct ResolvePhysicalTagRequest: Encodable, Equatable {
    let tagType: String
    let externalIdentifier: String

    enum CodingKeys: String, CodingKey {
        case tagType = "tag_type"
        case externalIdentifier = "external_identifier"
    }
}

struct ResolvedPhysicalTagResponse: Decodable, Equatable {
    let tag: PhysicalTagRecord
    let entityName: String?

    enum CodingKeys: String, CodingKey {
        case tag
        case entityName = "entity_name"
    }
}
