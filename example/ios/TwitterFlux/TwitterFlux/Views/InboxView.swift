import SwiftUI
import FluxSDK

// TODO: 由你来实现站内信页面 UI
// 调用 store.emit("inbox/load") 加载消息列表
// 多语言内容会根据当前 locale 自动切换

struct InboxView: View {
    @EnvironmentObject var store: FluxStore

    var body: some View {
        NavigationStack {
            Text("Inbox View - TODO")
                .navigationTitle("Inbox")
        }
    }
}
