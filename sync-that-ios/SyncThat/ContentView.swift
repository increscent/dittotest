//
//  ContentView.swift
//  SyncThat
//
//  Created by Robert Williams on 9/24/24.
//

import SwiftUI

struct ContentView: View {
    @StateObject private var syncData = SyncData()
    
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("Hello, world! \(self.syncData.wats)")
            Button("Ok", action: {
                self.syncData.update()
            })
        }
        .padding()
        .onAppear(perform: syncData.start)
        .onDisappear(perform: syncData.stop)
    }
}

#Preview {
    ContentView()
}
