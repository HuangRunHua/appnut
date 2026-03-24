import SwiftUI
import FluxSDK

@main
struct TwitterFluxApp: App {
    @StateObject private var store = FluxStore()

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(store)
        }
    }
}
