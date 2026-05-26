import SwiftUI
import PhotosUI

struct CreateItemView: View {
    @Environment(\.dismiss) private var dismiss

    @ObservedObject var itemsViewModel: ItemsViewModel
    let baseURLString: String

    @State private var name = ""
    @State private var category = ""
    @State private var subcategory = ""
    @State private var brand = ""
    @State private var size = ""
    @State private var colorPrimary = ""
    @State private var colorSecondary = ""
    @State private var material = ""
    @State private var season = ""
    @State private var formality = ""
    @State private var status = ""
    @State private var notes = ""
    @State private var selectedAnalysisImage: PhotosPickerItem?
    @State private var analysisMessage = "Select a photo to let Codex suggest item fields."
    @State private var analysisWarnings: [String] = []
    @State private var isAnalyzing = false

    var body: some View {
        NavigationStack {
            Form {
                Section("Photo Analysis") {
                    Text(analysisMessage)
                        .foregroundStyle(.secondary)

                    if !analysisWarnings.isEmpty {
                        ForEach(analysisWarnings, id: \.self) { warning in
                            Text(warning)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        }
                    }

                    PhotosPicker(
                        selection: $selectedAnalysisImage,
                        matching: .images
                    ) {
                        Label("Select Photo", systemImage: "photo")
                    }

                    if isAnalyzing {
                        ProgressView("Analyzing photo")
                    }
                }

                Section("New Item") {
                    TextField("Name", text: $name)
                    TextField("Category", text: $category)
                    TextField("Subcategory", text: $subcategory)
                    TextField("Brand", text: $brand)
                    TextField("Size", text: $size)
                    TextField("Primary color", text: $colorPrimary)
                    TextField("Secondary color", text: $colorSecondary)
                    TextField("Material", text: $material)
                    TextField("Season", text: $season)
                    TextField("Formality", text: $formality)
                    TextField("Status", text: $status)
                    TextField("Notes", text: $notes, axis: .vertical)
                }

                Section("Backend") {
                    Text("This creates the item through the backend JSON API. Codex photo analysis can prefill the form before you save it.")
                        .foregroundStyle(.secondary)
                }
            }
            .navigationTitle("Create Item")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        Task {
                            let created = await itemsViewModel.createItem(
                                requestBody: CreateItemRequest(
                                    name: name,
                                    category: category,
                                    subcategory: subcategory,
                                    brand: brand,
                                    size: size,
                                    colorPrimary: colorPrimary,
                                    colorSecondary: colorSecondary,
                                    material: material,
                                    season: season,
                                    formality: formality,
                                    status: status,
                                    currentLocationID: nil,
                                    notes: notes
                                ),
                                baseURLString: baseURLString
                            )
                            if created {
                                dismiss()
                            }
                        }
                    } label: {
                        if itemsViewModel.isCreating {
                            ProgressView()
                        } else {
                            Text("Create")
                        }
                    }
                    .disabled(itemsViewModel.isCreating)
                }
            }
            .onChange(of: selectedAnalysisImage) { _, newValue in
                guard let newValue else {
                    return
                }

                Task {
                    await analyzePhoto(newValue)
                    selectedAnalysisImage = nil
                }
            }
        }
    }

    private func analyzePhoto(_ pickerItem: PhotosPickerItem) async {
        guard !baseURLString.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            analysisMessage = "Save a backend URL in the Connection tab first."
            analysisWarnings = []
            return
        }

        guard let data = try? await pickerItem.loadTransferable(type: Data.self) else {
            analysisMessage = "The selected photo could not be loaded."
            analysisWarnings = []
            return
        }

        let type = pickerItem.supportedContentTypes.first
        let mimeType = type?.preferredMIMEType ?? "image/jpeg"
        let fileExtension = type?.preferredFilenameExtension ?? "jpg"
        let upload = PendingMediaUpload(
            data: data,
            fileName: "analysis-\(UUID().uuidString).\(fileExtension)",
            mimeType: mimeType,
            caption: nil
        )

        isAnalyzing = true
        defer { isAnalyzing = false }

        do {
            let suggestion = try await itemsViewModel.analyzeItemPhoto(
                upload: upload,
                baseURLString: baseURLString
            )
            applySuggestion(suggestion)
            analysisMessage = suggestion.summary
            analysisWarnings = suggestion.warnings
        } catch {
            analysisMessage = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
            analysisWarnings = []
        }
    }

    private func applySuggestion(_ suggestion: ItemPhotoAnalysisSuggestion) {
        if let value = suggestion.name { name = value }
        if let value = suggestion.category { category = value }
        if let value = suggestion.subcategory { subcategory = value }
        if let value = suggestion.brand { brand = value }
        if let value = suggestion.size { size = value }
        if let value = suggestion.colorPrimary { colorPrimary = value }
        if let value = suggestion.colorSecondary { colorSecondary = value }
        if let value = suggestion.material { material = value }
        if let value = suggestion.season { season = value }
        if let value = suggestion.formality { formality = value }
        if let value = suggestion.status { status = value }
        if let value = suggestion.notes { notes = value }
    }
}
