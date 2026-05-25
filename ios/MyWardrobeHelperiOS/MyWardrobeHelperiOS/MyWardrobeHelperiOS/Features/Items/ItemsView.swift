import SwiftUI

struct ItemsView: View {
    @ObservedObject var connectionViewModel: ConnectionViewModel
    @StateObject private var itemsViewModel = ItemsViewModel()
    @State private var isPresentingCreateSheet = false

    var body: some View {
        NavigationStack {
            List {
                Section("Backend Status") {
                    Text(itemsViewModel.statusMessage)
                        .foregroundStyle(.secondary)
                    Button("Reload Items") {
                        Task {
                            await itemsViewModel.loadItems(baseURLString: connectionViewModel.profile.baseURLString)
                        }
                    }
                    .disabled(itemsViewModel.isLoading)
                }

                Section("Items") {
                    if itemsViewModel.items.isEmpty {
                        Text("No items loaded yet.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(itemsViewModel.items) { item in
                            NavigationLink {
                                ItemDetailView(
                                    itemID: item.id,
                                    baseURLString: connectionViewModel.profile.baseURLString
                                )
                            } label: {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(item.name)
                                        .font(.headline)
                                    Text(summary(for: item))
                                        .font(.subheadline)
                                        .foregroundStyle(.secondary)
                                }
                            }
                        }
                    }
                }
            }
            .navigationTitle("Items")
            .toolbar {
                ToolbarItem {
                    Button("Add Item") {
                        isPresentingCreateSheet = true
                    }
                }
            }
            .sheet(isPresented: $isPresentingCreateSheet) {
                CreateItemView(
                    itemsViewModel: itemsViewModel,
                    baseURLString: connectionViewModel.profile.baseURLString
                )
            }
            .task(id: connectionViewModel.profile.baseURLString) {
                await itemsViewModel.loadItems(baseURLString: connectionViewModel.profile.baseURLString)
            }
        }
    }

    private func summary(for item: WardrobeItem) -> String {
        let parts = [
            item.category,
            item.brand,
            item.status
        ]
        .compactMap { $0 }
        .filter { !$0.isEmpty }

        return parts.isEmpty ? "No extra metadata yet" : parts.joined(separator: " · ")
    }
}
