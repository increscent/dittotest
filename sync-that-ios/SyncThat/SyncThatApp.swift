//
//  SyncThatApp.swift
//  SyncThat
//
//  Created by Robert Williams on 9/24/24.
//

import SwiftUI
import DittoSwift

protocol UpdateDelegate {
    func update()
}

@main
struct SyncThatApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
