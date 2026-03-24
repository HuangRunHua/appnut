import SwiftUI
import FluxSDK

// TODO: 由你来实现个人主页 UI
// 调用 store.emit("profile/load", payload: UserActionPayload(userId: ...)) 加载用户信息
// 调用 store.emit("user/follow", payload: UserActionPayload(userId: ...)) 关注

struct ProfileView: View {
    let userId: String
    @EnvironmentObject var store: FluxStore

    var body: some View {
        Text("Profile View - TODO")
            .navigationTitle("Profile")
    }
}
