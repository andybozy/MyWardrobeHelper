//
//  MyWardrobeHelperiOSApp.swift
//  MyWardrobeHelperiOS
//
//  Created by Andrea Bozzato on 25/05/2026.
//

import SwiftUI

@main
struct MyWardrobeHelperiOSApp: App {
    @StateObject private var connectionViewModel = ConnectionViewModel()

    var body: some Scene {
        WindowGroup {
            ContentView(viewModel: connectionViewModel)
        }
    }
}
