import SwiftUI

struct ConnectionView: View {
    @ObservedObject var viewModel: ConnectionViewModel

    var body: some View {
        NavigationStack {
            Form {
                Section("Backend Profile") {
                    TextField("Profile name", text: $viewModel.profile.displayName)
                    TextField("http://192.168.1.10:8787", text: $viewModel.profile.baseURLString)
                        .autocorrectionDisabled()

                    Button("Save Profile") {
                        viewModel.saveProfile()
                    }

                    Text(viewModel.saveMessage)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }

                Section("Connection Test") {
                    Button {
                        Task {
                            await viewModel.testConnection()
                        }
                    } label: {
                        if viewModel.isTesting {
                            HStack {
                                ProgressView()
                                Text("Testing connection")
                            }
                        } else {
                            Text("Test Connection")
                        }
                    }
                    .disabled(viewModel.isTesting || !viewModel.canTestConnection)

                    LabeledContent("Status", value: viewModel.connectionMessage)

                    if let checkedAt = viewModel.lastCheckedAt {
                        LabeledContent("Last checked", value: checkedAt.formatted(date: .abbreviated, time: .shortened))
                    }
                }

                Section("Backend Health") {
                    if let health = viewModel.healthResponse {
                        LabeledContent("API status", value: health.status)
                        LabeledContent("Items", value: "\(health.itemCount)")
                        LabeledContent("Locations", value: "\(health.locationCount)")
                        LabeledContent("Trips", value: "\(health.tripCount)")
                    } else {
                        Text("Run a connection test to load health details.")
                            .foregroundStyle(.secondary)
                    }
                }

                Section("Server Info") {
                    if let serverInfo = viewModel.serverInfoResponse {
                        LabeledContent("Application", value: serverInfo.application)
                        LabeledContent("Version", value: serverInfo.version)
                        LabeledContent("Bind URL", value: serverInfo.bindURL)
                        LabeledContent("Local URL", value: serverInfo.localURL)
                        if let lanURL = serverInfo.lanURL {
                            LabeledContent("LAN URL", value: lanURL)
                        }
                        LabeledContent("Data directory", value: serverInfo.dataDirectory)
                    } else {
                        Text("The iOS app is a client. It reads details from the backend after a successful test.")
                            .foregroundStyle(.secondary)
                    }
                }

                Section("Guidance") {
                    Label("The Rust backend remains the source of truth.", systemImage: "externaldrive")
                    Label("Use a LAN URL such as http://192.168.1.10:8787 on a physical device.", systemImage: "network")
                    Label("Use 127.0.0.1 only when the app runs on the same Mac, not on an iPhone or iPad.", systemImage: "info.circle")
                }
            }
            .navigationTitle("MyWardrobeHelper")
        }
    }
}
