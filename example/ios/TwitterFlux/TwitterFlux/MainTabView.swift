import SwiftUI

struct MainTabView: View {
    var body: some View {
        TabView {
            TimelineView()
                .tabItem {
                    Label("Home", systemImage: "house")
                }

            SearchView()
                .tabItem {
                    Label("Search", systemImage: "magnifyingglass")
                }

            InboxView()
                .tabItem {
                    Label("Inbox", systemImage: "bell")
                }

            SettingsView()
                .tabItem {
                    Label("Settings", systemImage: "gear")
                }
        }
    }
}
