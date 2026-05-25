import SwiftUI
import PhotosUI
import Foundation

struct ItemDetailView: View {
    let itemID: String
    let baseURLString: String

    @StateObject private var viewModel = ItemDetailViewModel()
    @State private var selectedMediaItems: [PhotosPickerItem] = []

    var body: some View {
        List {
            if let item = viewModel.item {
                Section("Overview") {
                    LabeledContent("Name", value: item.name)
                    LabeledContent("Category", value: item.category ?? "Not set")
                    LabeledContent("Brand", value: item.brand ?? "Not set")
                    LabeledContent("Status", value: item.status ?? "Not set")
                }

                Section("Details") {
                    LabeledContent("Subcategory", value: item.subcategory ?? "Not set")
                    LabeledContent("Size", value: item.size ?? "Not set")
                    LabeledContent("Primary color", value: item.colorPrimary ?? "Not set")
                    LabeledContent("Material", value: item.material ?? "Not set")
                    LabeledContent("Season", value: item.season ?? "Not set")
                    LabeledContent("Current location id", value: item.currentLocationID ?? "Not set")
                }

                Section("Backend Metadata") {
                    LabeledContent("Item id", value: item.id)
                    LabeledContent("Created", value: item.createdAt)
                    LabeledContent("Updated", value: item.updatedAt)
                    if let notes = item.notes, !notes.isEmpty {
                        Text(notes)
                    }
                }

                Section("Media") {
                    if viewModel.media.isEmpty {
                        Text("No media uploaded yet.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(viewModel.media) { media in
                            VStack(alignment: .leading, spacing: 4) {
                                Text(media.originalFilename)
                                    .font(.headline)
                                Text("\(media.mediaKind) · \(media.mimeType)")
                                    .font(.subheadline)
                                    .foregroundStyle(.secondary)
                                if let caption = media.caption, !caption.isEmpty {
                                    Text(caption)
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                }
                            }
                        }
                    }

                    PhotosPicker(
                        selection: $selectedMediaItems,
                        maxSelectionCount: 6,
                        matching: .any(of: [.images, .videos])
                    ) {
                        Label("Select Photos or Videos", systemImage: "photo.on.rectangle")
                    }

                    if viewModel.isUploading {
                        ProgressView(value: viewModel.uploadProgress)
                        Text(viewModel.message)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }
            } else {
                Section("Status") {
                    if viewModel.isLoading {
                        ProgressView("Loading item")
                    } else {
                        Text(viewModel.message)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .navigationTitle("Item Detail")
        .task(id: itemID) {
            await viewModel.loadItem(id: itemID, baseURLString: baseURLString)
        }
        .onChange(of: selectedMediaItems) { _, newItems in
            guard !newItems.isEmpty else {
                return
            }

            Task {
                let uploads = await buildUploads(from: newItems)
                selectedMediaItems = []
                await viewModel.uploadMedia(
                    itemID: itemID,
                    baseURLString: baseURLString,
                    uploads: uploads
                )
            }
        }
    }

    private func buildUploads(from pickerItems: [PhotosPickerItem]) async -> [PendingMediaUpload] {
        var uploads: [PendingMediaUpload] = []

        for item in pickerItems {
            guard let data = try? await item.loadTransferable(type: Data.self) else {
                continue
            }

            let type = item.supportedContentTypes.first
            let mimeType = type?.preferredMIMEType ?? "application/octet-stream"
            let fileExtension = type?.preferredFilenameExtension ?? "bin"
            let fileName = "upload-\(UUID().uuidString).\(fileExtension)"

            uploads.append(
                PendingMediaUpload(
                    data: data,
                    fileName: fileName,
                    mimeType: mimeType,
                    caption: nil
                )
            )
        }

        return uploads
    }
}
