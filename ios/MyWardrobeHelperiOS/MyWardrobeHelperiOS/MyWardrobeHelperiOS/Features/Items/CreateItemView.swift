import SwiftUI

struct CreateItemView: View {
    @Environment(\.dismiss) private var dismiss

    @ObservedObject var itemsViewModel: ItemsViewModel
    let baseURLString: String

    @State private var name = ""
    @State private var category = ""
    @State private var brand = ""

    var body: some View {
        NavigationStack {
            Form {
                Section("New Item") {
                    TextField("Name", text: $name)
                    TextField("Category", text: $category)
                    TextField("Brand", text: $brand)
                }

                Section("Backend") {
                    Text("This creates the item through the backend JSON API.")
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
                                name: name,
                                category: category,
                                brand: brand,
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
        }
    }
}
