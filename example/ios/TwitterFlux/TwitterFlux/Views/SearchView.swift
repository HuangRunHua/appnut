import SwiftUI
import FluxSDK

// TODO: 由你来实现搜索页面 UI
// 调用 store.emit("search/query", payload: SearchPayload(query: ...))

struct SearchView: View {
    @EnvironmentObject var store: FluxStore

    var body: some View {
        NavigationStack {
            Text("Search View - TODO")
                .navigationTitle("Search")
        }
    }
}
