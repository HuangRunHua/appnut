import Foundation
import Combine

@MainActor
public final class FluxStore: ObservableObject {

    public let client: FluxClient

    @Published public private(set) var authState: AuthState = .init()
    @Published public private(set) var timeline: [Tweet] = []
    @Published public private(set) var currentRoute: String = ""

    private var subscriptions: [SubscriptionID] = []

    public init(client: FluxClient = FluxClient()) {
        self.client = client
        setupSubscriptions()
    }

    deinit {
        for id in subscriptions {
            client.unsubscribe(id)
        }
    }

    private func setupSubscriptions() {
        let id = client.subscribe("#") { [weak self] path, json in
            Task { @MainActor in
                self?.handleChange(path: path, json: json)
            }
        }
        subscriptions.append(id)
    }

    private func handleChange(path: String, json: Data?) {
        switch path {
        case "auth/state":
            if let data = json {
                authState = (try? JSONDecoder().decode(AuthState.self, from: data)) ?? .init()
            }
        case "timeline/feed":
            if let data = json {
                timeline = (try? JSONDecoder().decode([Tweet].self, from: data)) ?? []
            }
        case "app/route":
            if let data = json, let route = String(data: data, encoding: .utf8) {
                currentRoute = route.trimmingCharacters(in: CharacterSet(charactersIn: "\""))
            }
        default:
            break
        }
    }

    // MARK: - Actions

    public func emit(_ path: String, payload: some Encodable) {
        client.emit(path, payload: payload)
    }

    public func emit(_ path: String) {
        client.emit(path)
    }

    public func refreshState() {
        if let auth: AuthState = client.get("auth/state") {
            authState = auth
        }
        if let feed: [Tweet] = client.get("timeline/feed") {
            timeline = feed
        }
    }
}

// MARK: - Data Models

public struct AuthState: Codable {
    public var phase: String?
    public var username: String?
    public var error: String?

    public var isLoggedIn: Bool { phase == "authenticated" }

    public init(phase: String? = nil, username: String? = nil, error: String? = nil) {
        self.phase = phase
        self.username = username
        self.error = error
    }
}

public struct Tweet: Codable, Identifiable {
    public let id: String
    public let author: String?
    public let content: String
    public let likeCount: Int
    public let replyCount: Int
    public let replyTo: String?
    public let displayName: String?
    public let createdAt: String?

    enum CodingKeys: String, CodingKey {
        case id, author, content
        case likeCount = "like_count"
        case replyCount = "reply_count"
        case replyTo = "reply_to"
        case displayName = "display_name"
        case createdAt = "created_at"
    }
}

public struct UserProfile: Codable, Identifiable {
    public let id: String
    public let username: String
    public let displayName: String?
    public let bio: String?
    public let avatar: String?
    public let followerCount: Int
    public let followingCount: Int
    public let tweetCount: Int

    enum CodingKeys: String, CodingKey {
        case id, username, bio, avatar
        case displayName = "display_name"
        case followerCount = "follower_count"
        case followingCount = "following_count"
        case tweetCount = "tweet_count"
    }
}

public struct Message: Codable, Identifiable {
    public let id: String
    public let kind: String
    public let title: String
    public let body: String
    public let read: Bool
    public let createdAt: String?

    enum CodingKeys: String, CodingKey {
        case id, kind, title, body, read
        case createdAt = "created_at"
    }
}

public struct LoginPayload: Codable {
    public let username: String
    public let password: String

    public init(username: String, password: String) {
        self.username = username
        self.password = password
    }
}

public struct CreateTweetPayload: Codable {
    public let content: String

    public init(content: String) {
        self.content = content
    }
}

public struct TweetActionPayload: Codable {
    public let tweetId: String

    public init(tweetId: String) {
        self.tweetId = tweetId
    }

    enum CodingKeys: String, CodingKey {
        case tweetId = "tweet_id"
    }
}

public struct UserActionPayload: Codable {
    public let userId: String

    public init(userId: String) {
        self.userId = userId
    }

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
    }
}

public struct SearchPayload: Codable {
    public let query: String

    public init(query: String) {
        self.query = query
    }
}
