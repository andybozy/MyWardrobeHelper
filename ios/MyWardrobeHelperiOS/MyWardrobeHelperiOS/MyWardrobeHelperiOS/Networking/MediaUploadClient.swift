import Foundation

struct PendingMediaUpload: Equatable {
    let data: Data
    let fileName: String
    let mimeType: String
    let caption: String?
}

struct MediaUploadClient {
    private let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }

    func uploadItemMedia(
        itemID: String,
        uploads: [PendingMediaUpload],
        baseURL: URL,
        progressHandler: @escaping @Sendable (Double, String) -> Void
    ) async throws -> [ItemMediaRecord] {
        guard !uploads.isEmpty else {
            return []
        }

        var createdMedia: [ItemMediaRecord] = []
        for (index, upload) in uploads.enumerated() {
            await MainActor.run {
                progressHandler(
                    Double(index) / Double(uploads.count),
                    "Uploading \(index + 1) of \(uploads.count)"
                )
            }

            let response = try await uploadSingleItemMedia(
                itemID: itemID,
                upload: upload,
                baseURL: baseURL
            )
            createdMedia.append(contentsOf: response.media)

            await MainActor.run {
                progressHandler(
                    Double(index + 1) / Double(uploads.count),
                    "Uploaded \(index + 1) of \(uploads.count)"
                )
            }
        }

        return createdMedia
    }

    private func uploadSingleItemMedia(
        itemID: String,
        upload: PendingMediaUpload,
        baseURL: URL
    ) async throws -> ItemMediaListResponse {
        var url = baseURL
        ["api", "v1", "items", itemID, "media"].forEach { segment in
            url.appendPathComponent(segment)
        }

        let boundary = "MyWardrobeHelperBoundary-\(UUID().uuidString)"
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "Content-Type")
        request.httpBody = multipartBody(for: upload, boundary: boundary)

        do {
            let (data, response) = try await session.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse else {
                throw APIClientError.invalidResponse
            }

            guard 200 ..< 300 ~= httpResponse.statusCode else {
                if let serverError = try? JSONDecoder().decode(ServerErrorEnvelope.self, from: data) {
                    throw APIClientError.serverMessage(serverError.error.message)
                }
                throw APIClientError.badStatusCode(httpResponse.statusCode)
            }

            do {
                return try JSONDecoder().decode(ItemMediaListResponse.self, from: data)
            } catch {
                throw APIClientError.decoding(error)
            }
        } catch let error as APIClientError {
            throw error
        } catch {
            throw APIClientError.transport(error)
        }
    }

    private func multipartBody(for upload: PendingMediaUpload, boundary: String) -> Data {
        var body = Data()
        let lineBreak = "\r\n"

        if let caption = upload.caption, !caption.isEmpty {
            body.append(Data("--\(boundary)\(lineBreak)".utf8))
            body.append(Data("Content-Disposition: form-data; name=\"caption\"\(lineBreak)\(lineBreak)".utf8))
            body.append(Data(caption.utf8))
            body.append(Data(lineBreak.utf8))
        }

        body.append(Data("--\(boundary)\(lineBreak)".utf8))
        body.append(Data("Content-Disposition: form-data; name=\"file\"; filename=\"\(upload.fileName)\"\(lineBreak)".utf8))
        body.append(Data("Content-Type: \(upload.mimeType)\(lineBreak)\(lineBreak)".utf8))
        body.append(upload.data)
        body.append(Data(lineBreak.utf8))
        body.append(Data("--\(boundary)--\(lineBreak)".utf8))

        return body
    }
}
