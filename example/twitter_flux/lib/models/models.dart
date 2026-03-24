class AuthState {
  final String? phase;
  final String? username;
  final String? error;

  bool get isLoggedIn => phase == 'authenticated';

  AuthState({this.phase, this.username, this.error});

  factory AuthState.fromJson(Map<String, dynamic> json) => AuthState(
        phase: json['phase'] as String?,
        username: json['username'] as String?,
        error: json['error'] as String?,
      );
}

class Tweet {
  final String id;
  final String? author;
  final String content;
  final int likeCount;
  final int replyCount;
  final String? replyTo;
  final String? displayName;
  final String? createdAt;

  Tweet({
    required this.id,
    this.author,
    required this.content,
    this.likeCount = 0,
    this.replyCount = 0,
    this.replyTo,
    this.displayName,
    this.createdAt,
  });

  factory Tweet.fromJson(Map<String, dynamic> json) => Tweet(
        id: json['id'] as String,
        author: json['author'] as String?,
        content: json['content'] as String,
        likeCount: json['like_count'] as int? ?? 0,
        replyCount: json['reply_count'] as int? ?? 0,
        replyTo: json['reply_to'] as String?,
        displayName: json['display_name'] as String?,
        createdAt: json['created_at'] as String?,
      );
}

class UserProfile {
  final String id;
  final String username;
  final String? displayName;
  final String? bio;
  final String? avatar;
  final int followerCount;
  final int followingCount;
  final int tweetCount;

  UserProfile({
    required this.id,
    required this.username,
    this.displayName,
    this.bio,
    this.avatar,
    this.followerCount = 0,
    this.followingCount = 0,
    this.tweetCount = 0,
  });

  factory UserProfile.fromJson(Map<String, dynamic> json) => UserProfile(
        id: json['id'] as String,
        username: json['username'] as String,
        displayName: json['display_name'] as String?,
        bio: json['bio'] as String?,
        avatar: json['avatar'] as String?,
        followerCount: json['follower_count'] as int? ?? 0,
        followingCount: json['following_count'] as int? ?? 0,
        tweetCount: json['tweet_count'] as int? ?? 0,
      );
}

class Message {
  final String id;
  final String kind;
  final String title;
  final String body;
  final bool read;
  final String? createdAt;

  Message({
    required this.id,
    required this.kind,
    required this.title,
    required this.body,
    this.read = false,
    this.createdAt,
  });

  factory Message.fromJson(Map<String, dynamic> json) => Message(
        id: json['id'] as String,
        kind: json['kind'] as String,
        title: json['title'] as String,
        body: json['body'] as String,
        read: json['read'] as bool? ?? false,
        createdAt: json['created_at'] as String?,
      );
}
