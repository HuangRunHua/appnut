import SwiftUI
import FluxSDK

// TODO: 由你来实现设置页面 UI
// 调用 store.client.setLocale("zh-CN") 切换语言
// 调用 store.emit("auth/logout") 退出登录

struct SettingsView: View {
    @EnvironmentObject var store: FluxStore

    var body: some View {
        NavigationStack {
            Text("Settings View - TODO")
                .navigationTitle("Settings")
        }
    }
}
