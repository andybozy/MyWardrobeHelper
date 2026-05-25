import Foundation

enum APIClientError: LocalizedError {
    case invalidBaseURL
    case invalidResponse
    case badStatusCode(Int)
    case decoding(Error)
    case transport(Error)

    var errorDescription: String? {
        switch self {
        case .invalidBaseURL:
            return "Enter a valid backend URL such as http://192.168.1.10:8787."
        case .invalidResponse:
            return "The backend returned an unexpected response."
        case let .badStatusCode(code):
            return "The backend returned HTTP \(code)."
        case let .decoding(error):
            return "The backend response could not be decoded: \(error.localizedDescription)"
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

    private func sendRequest<Response: Decodable>(
        pathSegments: [String],
        baseURL: URL
    ) async throws -> Response {
        var url = baseURL
        pathSegments.forEach { segment in
            url.appendPathComponent(segment)
        }

        let request = URLRequest(url: url, cachePolicy: .reloadIgnoringLocalCacheData, timeoutInterval: 10)

        do {
            let (data, response) = try await session.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse else {
                throw APIClientError.invalidResponse
            }

            guard 200 ..< 300 ~= httpResponse.statusCode else {
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
