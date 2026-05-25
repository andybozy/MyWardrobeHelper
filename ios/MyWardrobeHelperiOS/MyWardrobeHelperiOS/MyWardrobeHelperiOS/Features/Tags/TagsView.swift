import SwiftUI

struct TagsView: View {
    @ObservedObject var connectionViewModel: ConnectionViewModel
    @StateObject private var viewModel = TagsViewModel()

    var body: some View {
        NavigationStack {
            Form {
                Section("Resolve Tag") {
                    Picker("Tag Type", selection: $viewModel.tagType) {
                        Text("NFC").tag("nfc")
                        Text("QR").tag("qr")
                        Text("Barcode").tag("barcode")
                        Text("Other").tag("other")
                    }

                    TextField("External Identifier", text: $viewModel.externalIdentifier)
                        .autocorrectionDisabled()

                    Button {
                        Task {
                            await viewModel.resolveTag(baseURLString: connectionViewModel.profile.baseURLString)
                        }
                    } label: {
                        if viewModel.isResolving {
                            ProgressView()
                        } else {
                            Text("Resolve from Backend")
                        }
                    }
                    .disabled(viewModel.isResolving)

                    Button("Try Scanner Placeholder") {
                        Task {
                            await viewModel.beginScannerPlaceholder()
                        }
                    }
                }

                Section("Status") {
                    Text(viewModel.statusMessage)
                        .foregroundStyle(.secondary)
                    Text("Scanner boundary: \(viewModel.scannerDisplayName)")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }

                Section("Resolved Result") {
                    if let resolved = viewModel.resolvedTag {
                        LabeledContent("Tag type", value: resolved.tag.tagType)
                        LabeledContent("Identifier", value: resolved.tag.externalIdentifier)
                        LabeledContent("Label", value: resolved.tag.label ?? "Not set")
                        LabeledContent("Bound type", value: resolved.tag.boundEntityType)
                        LabeledContent("Bound id", value: resolved.tag.boundEntityID)
                        LabeledContent("Entity name", value: resolved.entityName ?? "Unknown")
                    } else {
                        Text("No tag is resolved yet.")
                            .foregroundStyle(.secondary)
                    }
                }

                Section("Honest Status") {
                    Label("The backend can already register and resolve tags.", systemImage: "tag")
                    Label("Live iPhone scanning is still future work.", systemImage: "wave.3.right.circle")
                    Label("This screen establishes the scanner abstraction without faking NFC support.", systemImage: "checkmark.seal")
                }
            }
            .navigationTitle("Tags")
        }
    }
}
