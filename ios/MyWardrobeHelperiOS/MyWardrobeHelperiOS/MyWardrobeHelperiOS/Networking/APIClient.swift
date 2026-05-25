import Foundation

enum APIClientError: LocalizedError {
    case invalidBaseURL
    case invalidResponse
    case badStatusCode(Int)
    case serverMessage(String)
    case decoding(Error)
    case encoding(Error)
    case transport(Error)

    var errorDescription: String? {
        switch self {
        case .invalidBaseURL:
            return "Enter a valid backend URL such as http://192.168.1.10:8787."
        case .invalidResponse:
            return "The backend returned an unexpected response."
        case let .badStatusCode(code):
            return "The backend returned HTTP \(code)."
        case let .serverMessage(message):
            return message
        case let .decoding(error):
            return "The backend response could not be decoded: \(error.localizedDescription)"
        case let .encoding(error):
            return "The request body could not be encoded: \(error.localizedDescription)"
        case let .transport(error):
            return error.localizedDescription
        }
    }
}

struct APIClient {
    private let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }

    func normalizedBaseURL(from rawValue: String) throws -> URL {
        let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            throw APIClientError.invalidBaseURL
        }

        let candidate = trimmed.contains("://") ? trimmed : "http://\(trimmed)"
        guard var url = URL(string: candidate), url.scheme != nil, url.host != nil else {
            throw APIClientError.invalidBaseURL
        }

        if url.path.isEmpty {
            url.append(path: "")
        }

        return url
    }

    func fetchHealth(baseURL: URL) async throws -> HealthResponse {
        try await sendRequest(pathSegments: ["api", "v1", "health"], baseURL: baseURL)
    }

    func fetchServerInfo(baseURL: URL) async throws -> ServerInfoResponse {
        try await sendRequest(pathSegments: ["api", "v1", "server-info"], baseURL: baseURL)
    }

    func fetchItems(baseURL: URL) async throws -> [WardrobeItem] {
        let response: ItemsListResponse = try await sendRequest(
            pathSegments: ["api", "v1", "items"],
            baseURL: baseURL
        )
        return response.items
    }

    func fetchItem(id: String, baseURL: URL) async throws -> WardrobeItem {
        try await sendRequest(pathSegments: ["api", "v1", "items", id], baseURL: baseURL)
    }

    func createItem(_ requestBody: CreateItemRequest, baseURL: URL) async throws -> WardrobeItem {
        let bodyData: Data
        do {
            bodyData = try JSONEncoder().encode(requestBody)
        } catch {
            throw APIClientError.encoding(error)
        }

        let item: WardrobeItem = try await sendRequest(
            pathSegments: ["api", "v1", "items"],
            method: "POST",
            bodyData: bodyData,
            baseURL: baseURL
        )
        return item
    }

    func fetchItemMedia(itemID: String, baseURL: URL) async throws -> [ItemMediaRecord] {
        let response: ItemMediaListResponse = try await sendRequest(
            pathSegments: ["api", "v1", "items", itemID, "media"],
            baseURL: baseURL
        )
        return response.media
    }

    func resolvePhysicalTag(
        tagType: String,
        externalIdentifier: String,
        baseURL: URL
    ) async throws -> ResolvedPhysicalTagResponse {
        let bodyData: Data
        do {
            bodyData = try JSONEncoder().encode(
                ResolvePhysicalTagRequest(
                    tagType: tagType,
                    externalIdentifier: externalIdentifier
                )
            )
        } catch {
            throw APIClientError.encoding(error)
        }

        let resolved: ResolvedPhysicalTagResponse = try await sendRequest(
            pathSegments: ["api", "v1", "tags", "resolve"],
            method: "POST",
            bodyData: bodyData,
            baseURL: baseURL
        )
        return resolved
    }

    private func sendRequest<Response: Decodable>(
        pathSegments: [String],
        method: String = "GET",
        bodyData: Data? = nil,
        baseURL: URL
    ) async throws -> Response {
        var url = baseURL
        pathSegments.forEach { segment in
            url.appendPathComponent(segment)
        }

        var request = URLRequest(url: url, cachePolicy: .reloadIgnoringLocalCacheData, timeoutInterval: 10)
        request.httpMethod = method

        if let bodyData {
            request.httpBody = bodyData
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        }

        do {
            let (data, response) = try await session.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse else {
                throw APIClientError.invalidResponse
            }

            guard 200 ..< 300 ~= httpResponse.statusCode else {
                if
                    let serverError = try? JSONDecoder().decode(ServerErrorEnvelope.self, from: data)
                {
                    throw APIClientError.serverMessage(serverError.error.message)
                }
                throw APIClientError.badStatusCode(httpResponse.statusCode)
            }

            do {
                let decoder = JSONDecoder()
                return try decoder.decode(Response.self, from: data)
            } catch {
                throw APIClientError.decoding(error)
            }
        } catch let error as APIClientError {
            throw error
        } catch {
            throw APIClientError.transport(error)
        }
    }
}
