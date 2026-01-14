/// Comment entity representing a comment on a document
///
/// This is a pure domain entity that represents a comment
/// in the miniWiki Knowledge Management Platform.
class Comment {
  /// Unique identifier for the comment
  final String id;

  /// ID of the document this comment belongs to
  final String documentId;

  /// ID of the parent comment (for threaded replies)
  final String? parentId;

  /// ID of the user who created the comment
  final String authorId;

  /// Display name of the comment author
  final String authorName;

  /// Avatar URL of the comment author
  final String? authorAvatar;

  /// Content of the comment
  final String content;

  /// Whether the comment is resolved
  final bool isResolved;

  /// ID of the user who resolved the comment
  final String? resolvedBy;

  /// Timestamp when the comment was resolved
  final DateTime? resolvedAt;

  /// Timestamp when the comment was created
  final DateTime? createdAt;

  /// Timestamp when the comment was last updated
  final DateTime? updatedAt;

  const Comment({
    required this.id,
    required this.documentId,
    this.parentId,
    required this.authorId,
    required this.authorName,
    this.authorAvatar,
    required this.content,
    this.isResolved = false,
    this.resolvedBy,
    this.resolvedAt,
    this.createdAt,
    this.updatedAt,
  });

  /// Creates a copy of the comment with updated fields
  Comment copyWith({
    String? id,
    String? documentId,
    String? parentId,
    String? authorId,
    String? authorName,
    String? authorAvatar,
    String? content,
    bool? isResolved,
    String? resolvedBy,
    DateTime? resolvedAt,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) =>
      Comment(
        id: id ?? this.id,
        documentId: documentId ?? this.documentId,
        parentId: parentId ?? this.parentId,
        authorId: authorId ?? this.authorId,
        authorName: authorName ?? this.authorName,
        authorAvatar: authorAvatar ?? this.authorAvatar,
        content: content ?? this.content,
        isResolved: isResolved ?? this.isResolved,
        resolvedBy: resolvedBy ?? this.resolvedBy,
        resolvedAt: resolvedAt ?? this.resolvedAt,
        createdAt: createdAt ?? this.createdAt,
        updatedAt: updatedAt ?? this.updatedAt,
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
      return null;
    }
  }

  /// Creates a Comment from JSON (for API responses)
  factory Comment.fromJson(Map<String, dynamic> json) => Comment(
        id: json['id'] as String,
        documentId: json['document_id'] as String,
        parentId: json['parent_id'] as String?,
        authorId: json['author_id'] as String,
        authorName: json['author_name'] as String,
        authorAvatar: json['author_avatar'] as String?,
        content: json['content'] as String,
        isResolved: json['is_resolved'] as bool? ?? false,
        resolvedBy: json['resolved_by'] as String?,
        resolvedAt: _parseDateTime(json['resolved_at'] as String?),
        createdAt: _parseDateTime(json['created_at'] as String?),
        updatedAt: _parseDateTime(json['updated_at'] as String?),
      );

  /// Converts Comment to JSON (for API requests)
  Map<String, dynamic> toJson() {
    final jsonMap = <String, dynamic>{
      'id': id,
      'document_id': documentId,
      'author_id': authorId,
      'author_name': authorName,
      'content': content,
      'is_resolved': isResolved,
    };

    if (parentId != null) {
      jsonMap['parent_id'] = parentId;
    }

    if (authorAvatar != null) {
      jsonMap['author_avatar'] = authorAvatar;
    }

    if (resolvedBy != null) {
      jsonMap['resolved_by'] = resolvedBy;
    }

    if (resolvedAt != null) {
      jsonMap['resolved_at'] = resolvedAt!.toIso8601String();
    }

    if (createdAt != null) {
      jsonMap['created_at'] = createdAt!.toIso8601String();
    }

    if (updatedAt != null) {
      jsonMap['updated_at'] = updatedAt!.toIso8601String();
    }

    return jsonMap;
  }

  @override
  String toString() => 'Comment(id: $id, author: $authorName)';

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is Comment && other.id == id;
  }

  @override
  int get hashCode => id.hashCode;
}
