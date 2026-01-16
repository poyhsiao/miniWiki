import 'dart:convert';
import 'package:isar_community/isar.dart';

part 'document_entity.g.dart';

/// Document entity for data layer (Isar storage)
///
/// This entity is used for offline storage with Isar database.
/// It has mutable fields for easy modification during offline editing.
@collection
class DocumentEntity {
  /// Auto-increment primary key for Isar
  Id id = Isar.autoIncrement;

  /// Unique identifier for the document
  @Index(unique: true)
  String? uuid;

  /// ID of the space this document belongs to
  @Index()
  String? spaceId;

  /// Serialize to JSON with validation
  Map<String, dynamic> toJson() {
    if (uuid == null || uuid!.isEmpty || spaceId == null || spaceId!.isEmpty) {
      throw StateError(
          'DocumentEntity validation failed: uuid or spaceId cannot be empty');
    }
    return {
      'uuid': uuid,
      'spaceId': spaceId,
      'title': title,
      'parentId': parentId,
      'icon': icon,
      'content': content,
      'contentSize': contentSize,
      'isArchived': isArchived,
      'createdBy': createdBy,
      'lastEditedBy': lastEditedBy,
      'createdAt': createdAt?.toIso8601String(),
      'updatedAt': updatedAt?.toIso8601String(),
      'isSynced': isSynced,
      'isDirty': isDirty,
      'lastSyncedAt': lastSyncedAt?.toIso8601String(),
    };
  }

  /// ID of the parent document (for hierarchical organization)
  String? parentId;

  /// Title of the document
  String title = '';

  /// Icon for the document
  String? icon;

  /// Document content as JSON string (Yjs CRDT state)
  /// Stored as JSON string for Isar persistence
  String? contentJson;

  /// Document content as Map (computed property)
  @ignore
  Map<String, dynamic>? get content {
    if (contentJson == null || contentJson!.isEmpty) return null;
    try {
      final decoded = jsonDecode(contentJson!);
      if (decoded is Map<String, dynamic>) {
        return decoded;
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  /// Set document content from Map
  set content(Map<String, dynamic>? value) {
    if (value == null) {
      contentJson = null;
      contentSize = 0;
    } else {
      contentJson = jsonEncode(value);
      contentSize = utf8.encode(contentJson!).length;
    }
  }

  /// Size of the content in bytes
  int contentSize = 0;

  /// Whether the document is archived (soft deleted)
  bool isArchived = false;

  /// ID of the user who created the document
  String? createdBy;

  /// ID of the user who last edited the document
  String? lastEditedBy;

  /// Timestamp when the document was created
  DateTime? createdAt;

  /// Timestamp when the document was last updated
  DateTime? updatedAt;

  /// Whether the document is synced with the server
  /// This is used in offline-first mode
  bool isSynced = true;

  /// Whether the document has unsaved changes
  /// This is used in offline-first mode
  bool isDirty = false;

  /// Timestamp when the document was last synced
  DateTime? lastSyncedAt;

  /// Creates a copy of this entity
  DocumentEntity copyWith({
    int? id,
    String? uuid,
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
  }) {
    final entity = DocumentEntity()
      ..id = id ?? this.id
      ..uuid = uuid ?? this.uuid
      ..spaceId = spaceId ?? this.spaceId
      ..parentId = parentId ?? this.parentId
      ..title = title ?? this.title
      ..icon = icon ?? this.icon
      ..isArchived = isArchived ?? this.isArchived
      ..createdBy = createdBy ?? this.createdBy
      ..lastEditedBy = lastEditedBy ?? this.lastEditedBy
      ..createdAt = createdAt ?? this.createdAt
      ..updatedAt = updatedAt ?? this.updatedAt
      ..isSynced = isSynced ?? this.isSynced
      ..isDirty = isDirty ?? this.isDirty
      ..lastSyncedAt = lastSyncedAt ?? this.lastSyncedAt;

    final contentToUse = content ?? this.content;
    entity.content = contentToUse;
    if (contentSize != null && content == null) {
      entity.contentSize = contentSize;
    }

    return entity;
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other is! DocumentEntity) return false;
    if (uuid == null || other.uuid == null) return false;
    return other.uuid == uuid;
  }

  @override
  int get hashCode => uuid != null ? uuid.hashCode : identityHashCode(this);
}
