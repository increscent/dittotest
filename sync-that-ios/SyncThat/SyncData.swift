//
//  Data.swift
//  SyncThat
//
//  Created by Robert Williams on 9/24/24.
//

import Foundation
import DittoSwift

class SyncData: ObservableObject {
    private var ditto: Ditto = Ditto(identity: .onlinePlayground(
                appID: Constants.appID,
                token: Constants.playgroundAuthToken,
                enableDittoCloudSync: true))
        private var observer: DittoStoreObserver?
    
    @Published var wats: Int = 0
    
    func start() {
        DispatchQueue.main.async {
            do {
                self.ditto.deviceName = "Random Ditto Device"
                
                var config = DittoTransportConfig()
                config.enableAllPeerToPeer()
                self.ditto.transportConfig = config

                try self.ditto.disableSyncWithV3()
                try self.ditto.startSync()
                
                try self.ditto.sync.registerSubscription(query: "SELECT * FROM wats")
                
                self.observer = try self.ditto.store.registerObserver(
                    query: "SELECT * FROM wats"){ result in
                        self.wats = result.items.count
                    };
            } catch(let err) {
                print("Ditto error: \(err.localizedDescription)")
            }
        }
    }
    
    func stop() {
        self.observer?.cancel()
    }
    
    func update() {
        Task {
            do {
                try await self.ditto.store.execute(
                    query: "INSERT INTO wats DOCUMENTS (:newWat)",
                    arguments: ["newWat": ["color": "blue"]]);
            } catch (let err) {
                print("Ditto error: \(err.localizedDescription)")
            }
        }
    }
}
