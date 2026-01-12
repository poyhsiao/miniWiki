/// Space entity representing a workspace/space in miniWiki
///
/// This is a pure domain entity that represents a space
/// in the miniWiki Knowledge Management Platform.
class Space {
  /// Unique identifier for the space
  final String id;

  /// Name of the space
  final String name;

  /// Icon for the space
  final String? icon;

  /// Description of the space
  final String? description;

  /// Whether the space is public
  final bool isPublic;

  /// ID of the owner who created the space
  final String ownerId;

  /// Timestamp when the space was created
  final DateTime createdAt;

  /// Timestamp when the space was last updated
  final DateTime updatedAt;

  /// Role of the current user in this space
  final String? userRole;

  /// Whether the space is synced with the server
  /// This is used in offline-first mode
  final bool isSynced;

  /// Whether the space has unsaved changes
  /// This is used in offline-first mode
  final bool isDirty;

  /// Timestamp when the space was last synced
  /// This is used in offline-first mode
  final DateTime? lastSyncedAt;

  const Space({
    required this.id,
    required this.name,
    this.icon,
    this.description,
    this.isPublic = false,
    required this.ownerId,
    required this.createdAt,
    required this.updatedAt,
    this.userRole,
    this.isSynced = true,
    this.isDirty = false,
    this.lastSyncedAt,
  });

  /// Creates a copy of the space with updated fields
  Space copyWith({
    String? id,
    String? name,
    String? icon,
    String? description,
    bool? isPublic,
    String? ownerId,
    DateTime? createdAt,
    DateTime? updatedAt,
    String? userRole,
    bool? isSynced,
    bool? isDirty,
    DateTime? lastSyncedAt,
  }) {
    return Space(
      id: id ?? this.id,
      name: name ?? this.name,
      icon: icon ?? this.icon,
      description: description ?? this.description,
      isPublic: isPublic ?? this.isPublic,
      ownerId: ownerId ?? this.ownerId,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
      userRole: userRole ?? this.userRole,
      isSynced: isSynced ?? this.isSynced,
      isDirty: isDirty ?? this.isDirty,
      lastSyncedAt: lastSyncedAt ?? this.lastSyncedAt,
    );
  }

  /// Creates a Space from JSON (for API responses)
  factory Space.fromJson(Map<String, dynamic> json) {
    final createdAtStr = json['created_at'] as String?;
    final updatedAtStr = json['updated_at'] as String?;
    final lastSyncedAtStr = json['last_synced_at'] as String?;

    DateTime? createdAt;
    DateTime? updatedAt;
    DateTime? lastSyncedAt;

    if (createdAtStr != null && createdAtStr.isNotEmpty) {
      try {
        createdAt = DateTime.parse(createdAtStr);
      } catch (e) {
        createdAt = DateTime.now();
      }
    }

    if (updatedAtStr != null && updatedAtStr.isNotEmpty) {
      try {
        updatedAt = DateTime.parse(updatedAtStr);
      } catch (e) {
        updatedAt = DateTime.now();
      }
    }

    if (lastSyncedAtStr != null && lastSyncedAtStr.isNotEmpty) {
      try {
        lastSyncedAt = DateTime.parse(lastSyncedAtStr);
      } catch (e) {
        lastSyncedAt = null;
      }
    }

    return Space(
      id: json['id'] as String,
      name: json['name'] as String,
      icon: json['icon'] as String?,
      description: json['description'] as String?,
      isPublic: json['is_public'] as bool? ?? false,
      ownerId: json['owner_id'] as String,
      createdAt: createdAt ?? DateTime.now(),
      updatedAt: updatedAt ?? DateTime.now(),
      userRole: json['user_role'] as String?,
      isSynced: json['is_synced'] as bool? ?? true,
      isDirty: json['is_dirty'] as bool? ?? false,
      lastSyncedAt: lastSyncedAt,
    );
  }

  /// Converts Space to JSON (for API requests)
  Map<String, dynamic> toJson() {
    final jsonMap = <String, dynamic>{
      'id': id,
      'name': name,
      'is_public': isPublic,
      'owner_id': ownerId,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
      'is_synced': isSynced,
      'is_dirty': isDirty,
    };

    if (icon != null) {
      jsonMap['icon'] = icon;
    }

    if (description != null) {
      jsonMap['description'] = description;
    }

    if (userRole != null) {
      jsonMap['user_role'] = userRole;
    }

    if (lastSyncedAt != null) {
      jsonMap['last_synced_at'] = lastSyncedAt!.toIso8601String();
    }

    return jsonMap;
  }

  @override
  String toString() {
    return 'Space(id: $id, name: $name)';
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is Space && other.id == id;
  }

  @override
  int get hashCode => id.hashCode;
}
