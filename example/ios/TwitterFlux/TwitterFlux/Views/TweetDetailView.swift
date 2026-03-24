import SwiftUI
import FluxSDK

// TODO: 由你来实现推文详情页 UI
// 调用 store.emit("tweet/detail", payload: TweetActionPayload(tweetId: ...)) 加载详情
// 调用 store.emit("tweet/like", payload: TweetActionPayload(tweetId: ...)) 点赞

struct TweetDetailView: View {
    let tweet: Tweet
    @EnvironmentObject var store: FluxStore

    var body: some View {
        Text("Tweet Detail - TODO")
            .navigationTitle("Tweet")
    }
}
