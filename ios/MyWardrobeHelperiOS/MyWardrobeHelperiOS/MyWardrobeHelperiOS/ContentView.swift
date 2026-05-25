//
//  ContentView.swift
//  MyWardrobeHelperiOS
//
//  Created by Andrea Bozzato on 25/05/2026.
//

import SwiftUI

struct ContentView: View {
    @ObservedObject var viewModel: ConnectionViewModel

    var body: some View {
        TabView {
            ConnectionView(viewModel: viewModel)
                .tabItem {
                    Label("Connection", systemImage: "network")
                }

            ItemsView(connectionViewModel: viewModel)
                .tabItem {
                    Label("Items", systemImage: "tshirt")
                }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView(viewModel: ConnectionViewModel())
    }
}
