/// Document entity representing a document in miniWiki
///
/// This is a pure domain entity that represents a document
/// in the miniWiki Knowledge Management Platform.
class Document {
  /// Unique identifier for the document
  final String id;

  /// ID of the space this document belongs to
  final String spaceId;

  /// ID of the parent document (for hierarchical organization)
  final String? parentId;

  /// Title of the document
  final String title;

  /// Icon for the document
  final String? icon;

  /// Document content as JSON (Yjs CRDT state)
  final Map<String, dynamic> content;

  /// Size of the content in bytes
  final int contentSize;

  /// Whether the document is archived (soft deleted)
  final bool isArchived;

  /// ID of the user who created the document
  final String createdBy;

  /// ID of the user who last edited the document
  final String lastEditedBy;

  /// Timestamp when the document was created
  final DateTime? createdAt;

  /// Timestamp when the document was last updated
  final DateTime? updatedAt;

  /// Whether the document is synced with the server
  /// This is used in offline-first mode
  final bool isSynced;

  /// Whether the document has unsaved changes
  /// This is used in offline-first mode
  final bool isDirty;

  /// Timestamp when the document was last synced
  /// This is used in offline-first mode
  final DateTime? lastSyncedAt;

  const Document({
    required this.id,
    required this.spaceId,
    required this.title, required this.createdBy, required this.lastEditedBy, this.parentId,
    this.icon,
    this.content = const {},
    this.contentSize = 0,
    this.isArchived = false,
    this.createdAt,
    this.updatedAt,
    this.isSynced = true,
    this.isDirty = false,
    this.lastSyncedAt,
  });

  /// Creates a copy of the document with updated fields
  Document copyWith({
    String? id,
    String? spaceId,
    String? parentId,
    String? title,
    String? icon,
    Map<String, dynamic>? content,
    int? contentSize,
    bool? isArchived,
    String? createdBy,
    String? lastEditedBy,
    DateTime? createdAt,
    DateTime? updatedAt,
    bool? isSynced,
    bool? isDirty,
    DateTime? lastSyncedAt,
  }) => Document(
      id: id ?? this.id,
      spaceId: spaceId ?? this.spaceId,
      parentId: parentId ?? this.parentId,
      title: title ?? this.title,
      icon: icon ?? this.icon,
      content: content ?? this.content,
      contentSize: contentSize ?? this.contentSize,
      isArchived: isArchived ?? this.isArchived,
      createdBy: createdBy ?? this.createdBy,
      lastEditedBy: lastEditedBy ?? this.lastEditedBy,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
      isSynced: isSynced ?? this.isSynced,
      isDirty: isDirty ?? this.isDirty,
      lastSyncedAt: lastSyncedAt ?? this.lastSyncedAt,
    );

  /// Helper method to safely parse DateTime from API
  /// Returns null if input is null or empty
  static DateTime? _parseDateTime(String? dateTimeString) {
    if (dateTimeString == null || dateTimeString.isEmpty) {
      return null;
    }
    try {
      return DateTime.parse(dateTimeString);
    } catch (e) {
      return DateTime.now();
    }
  }

  /// Creates a Document from JSON (for API responses)
  factory Document.fromJson(Map<String, dynamic> json) => Document(
      id: json['id'] as String,
      spaceId: json['space_id'] as String,
      parentId: json['parent_id'] as String?,
      title: json['title'] as String,
      icon: json['icon'] as String?,
      content: json['content'] as Map<String, dynamic>? ?? {},
      contentSize: json['content_size'] as int? ?? 0,
      isArchived: json['is_archived'] as bool? ?? false,
      createdBy: json['created_by'] as String,
      lastEditedBy: json['last_edited_by'] as String,
      createdAt: _parseDateTime(json['created_at'] as String?),
      updatedAt: _parseDateTime(json['updated_at'] as String?),
      isSynced: json['is_synced'] as bool? ?? true,
      isDirty: json['is_dirty'] as bool? ?? false,
      lastSyncedAt: _parseDateTime(json['last_synced_at'] as String?),
    );

  /// Converts Document to JSON (for API requests)
  Map<String, dynamic> toJson() {
    final jsonMap = <String, dynamic>{
      'id': id,
      'space_id': spaceId,
      'title': title,
      'content': content,
      'content_size': contentSize,
      'is_archived': isArchived,
      'created_by': createdBy,
      'last_edited_by': lastEditedBy,
      'is_synced': isSynced,
      'is_dirty': isDirty,
    };

    if (parentId != null) {
      jsonMap['parent_id'] = parentId;
    }

    if (icon != null) {
      jsonMap['icon'] = icon;
    }

    if (createdAt != null) {
      jsonMap['created_at'] = createdAt!.toIso8601String();
    }

    if (updatedAt != null) {
      jsonMap['updated_at'] = updatedAt!.toIso8601String();
    }

    if (lastSyncedAt != null) {
      jsonMap['last_synced_at'] = lastSyncedAt!.toIso8601String();
    }

    return jsonMap;
  }

  @override
  String toString() => 'Document(id: $id, title: $title)';

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is Document && other.id == id;
  }

  @override
  int get hashCode => id.hashCode;
}
