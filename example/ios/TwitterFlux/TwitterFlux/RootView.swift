import SwiftUI
import FluxSDK

struct RootView: View {
    @EnvironmentObject var store: FluxStore

    var body: some View {
        Group {
            if store.authState.isLoggedIn {
                MainTabView()
            } else {
                LoginView()
            }
        }
        .animation(.easeInOut, value: store.authState.isLoggedIn)
    }
}
