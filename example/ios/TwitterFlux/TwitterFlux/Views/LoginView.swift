import SwiftUI
import FluxSDK

// TODO: 由你来实现登录页面 UI
// 调用 store.emit("auth/login", payload: LoginPayload(username: ..., password: ...))
// 登录成功后 store.authState.isLoggedIn 会变为 true，自动跳转到主页面

struct LoginView: View {
    @EnvironmentObject var store: FluxStore

    var body: some View {
        Text("Login View - TODO")
    }
}
