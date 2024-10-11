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
                
                self.ditto.delegate = self

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

extension SyncData: DittoDelegate {
    public func dittoTransportConditionDidChange(
        ditto: DittoSwift.Ditto,
        condition: DittoSwift.DittoTransportCondition,
        subsystem: DittoSwift.DittoConditionSource)
    {
        switch condition {
        case .Unknown:
            // TODO: What is this??
            break
        case .Ok:
            // TODO
            break
        case .GenericFailure:
            // TODO: What is this??
            break
        case .AppInBackground:
            // TODO: How is this useful?
            break
        case .CannotEstablishConnection:
            // TODO: How is this useful?
            break
        case .NoBleHardware:
            // TODO: Does this actually work??
            break
        case .TemporarilyUnavailable:
            // TODO: What is this??
            break
        case .TcpListenFailure:
            // TODO: Is this a permissions issue or something else??
            break
            
        case .NoBleCentralPermission:
            fallthrough
        case .NoBlePeripheralPermission:
            // This case and .NoBleCentralPermission go together; on Apple devices they are the same permission
            // Prompt the user to go to Settings and allow this app to use Bluetooth
//            bluetoothPermissionDeniedAlert()
            break
        case .BleDisabled:
            // Prompt the user to re-enable Bluetooth
//            bluetoothDisabledAlert()
            break
        case .MdnsFailure:
            // Prompt the user to go to Settings and allow this app to use "Local Network"
//            localNetworkPermissionDeniedAlert()
            break
        case .WifiDisabled:
            // Prompt the user to re-enable WiFi
//            wifiDisabledAlert()
            break
            
        @unknown default:
            fatalError()
        }
        
        print("dittoTransportConditionDidChange \(subsystem) \(condition)")
    }
}
