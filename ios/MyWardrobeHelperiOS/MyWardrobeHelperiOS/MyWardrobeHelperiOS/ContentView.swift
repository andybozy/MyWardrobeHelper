//
//  ContentView.swift
//  MyWardrobeHelperiOS
//
//  Created by Andrea Bozzato on 25/05/2026.
//

import SwiftUI

struct ContentView: View {
    var body: some View {
        NavigationStack {
            List {
                Section("Status") {
                    Label("Native SwiftUI companion app", systemImage: "iphone")
                    Label("Backend remains the source of truth", systemImage: "externaldrive")
                    Label("Local-network API connection planned", systemImage: "network")
                }

                Section("Next milestones") {
                    Text("SEC-011: server profile and LAN connectivity")
                    Text("SEC-012: item browsing and editing")
                    Text("SEC-013: image and video upload")
                }
            }
            .navigationTitle("MyWardrobeHelper")
        }
    }
}

#Preview {
    ContentView()
}
