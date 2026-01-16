import 'package:isar_community/isar.dart';

part 'user_entity.g.dart';

/// Sentinel value for copyWith to distinguish between "not provided" and "null"
const _sentinel = Object();

/// User entity for data layer (Isar storage)
///
/// This entity is used for offline storage with Isar database.
/// It has mutable fields for easy modification during offline editing.
@collection
class UserEntity {
  /// Auto-increment primary key for Isar
  Id id = Isar.autoIncrement;

  /// Unique identifier for the user
  @Index(unique: true)
  String uuid;

  /// User email address
  String email;

  /// Constructor
  UserEntity({
    required this.uuid,
    this.email = '',
    this.displayName = '',
    this.avatarUrl,
    this.timezone = 'UTC',
    this.language = 'en',
    this.isActive = true,
    this.isEmailVerified = false,
    this.emailVerifiedAt,
    this.lastLoginAt,
    this.createdAt,
    this.updatedAt,
  });

  /// Display name
  String displayName = '';

  /// Avatar image URL
  String? avatarUrl;

  /// User timezone
  String timezone = 'UTC';

  /// Language preference
  String language = 'en';

  /// Whether the account is active
  bool isActive = true;

  /// Whether the email is verified
  bool isEmailVerified = false;

  /// Timestamp when email was verified
  DateTime? emailVerifiedAt;

  /// Timestamp of last login
  DateTime? lastLoginAt;

  /// Timestamp when the user was created
  DateTime? createdAt;

  /// Timestamp when the user was last updated
  DateTime? updatedAt;

  /// Creates a copy of this entity
  UserEntity copyWith({
    int? id,
    String? uuid,
    String? email,
    String? displayName,
    Object? avatarUrl = _sentinel,
    String? timezone,
    String? language,
    bool? isActive,
    bool? isEmailVerified,
    Object? emailVerifiedAt = _sentinel,
    Object? lastLoginAt = _sentinel,
    Object? createdAt = _sentinel,
    Object? updatedAt = _sentinel,
  }) {
    return UserEntity(
      uuid: uuid ?? this.uuid,
      email: email ?? this.email,
      displayName: displayName ?? this.displayName,
      avatarUrl: avatarUrl == _sentinel ? this.avatarUrl : avatarUrl as String?,
      timezone: timezone ?? this.timezone,
      language: language ?? this.language,
      isActive: isActive ?? this.isActive,
      isEmailVerified: isEmailVerified ?? this.isEmailVerified,
      emailVerifiedAt: emailVerifiedAt == _sentinel
          ? this.emailVerifiedAt
          : emailVerifiedAt as DateTime?,
      lastLoginAt: lastLoginAt == _sentinel
          ? this.lastLoginAt
          : lastLoginAt as DateTime?,
      createdAt:
          createdAt == _sentinel ? this.createdAt : createdAt as DateTime?,
      updatedAt:
          updatedAt == _sentinel ? this.updatedAt : updatedAt as DateTime?,
    )..id = id ?? this.id;
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is UserEntity && other.uuid == uuid;
  }

  @override
  int get hashCode => uuid.hashCode;
}
