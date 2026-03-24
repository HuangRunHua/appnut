import SwiftUI
import FluxSDK

// TODO: 由你来实现发推页面 UI
// 调用 store.emit("tweet/create", payload: CreateTweetPayload(content: ...))

struct ComposeView: View {
    @EnvironmentObject var store: FluxStore
    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationStack {
            Text("Compose View - TODO")
                .navigationTitle("New Tweet")
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button("Cancel") { dismiss() }
                    }
                }
        }
    }
}
