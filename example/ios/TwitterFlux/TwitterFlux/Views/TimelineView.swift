import SwiftUI
import FluxSDK

// TODO: 由你来实现时间线页面 UI
// 读取 store.timeline 获取推文列表
// 调用 store.emit("timeline/load") 加载/刷新时间线

struct TimelineView: View {
    @EnvironmentObject var store: FluxStore

    var body: some View {
        NavigationStack {
            Text("Timeline View - TODO")
                .navigationTitle("Home")
        }
    }
}
