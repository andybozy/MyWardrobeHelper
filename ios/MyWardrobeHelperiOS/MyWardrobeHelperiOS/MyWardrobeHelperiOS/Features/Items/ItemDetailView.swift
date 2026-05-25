import SwiftUI

struct ItemDetailView: View {
    let itemID: String
    let baseURLString: String

    @StateObject private var viewModel = ItemDetailViewModel()

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
    }
}
