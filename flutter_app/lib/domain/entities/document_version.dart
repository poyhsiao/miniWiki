/// DocumentVersion entity representing a version of a document in miniWiki
///
/// This is a pure domain entity that represents a specific version
/// of a document in the version history.
class DocumentVersion {
  /// Unique identifier for the version
  final String id;

  /// ID of the document this version belongs to
  final String documentId;

  /// Sequential version number (1, 2, 3, ...)
  final int versionNumber;

  /// Title of the document at this version
  final String title;

  /// Version content as JSON (Yjs CRDT state)
  final Map<String, dynamic> content;

  /// Optional summary of changes in this version
  final String? changeSummary;

  /// ID of the user who created this version
  final String createdBy;

  /// Timestamp when this version was created
  final DateTime? createdAt;

  /// Vector clock for CRDT conflict resolution
  final Map<String, dynamic> vectorClock;

  /// If this version was created by restoring, the original version number
  final int? restoredFromVersion;

  /// Size of the content in bytes
  final int contentSize;

  const DocumentVersion({
    required this.id,
    required this.documentId,
    required this.versionNumber,
    required this.title,
    required this.createdBy,
    required this.vectorClock,
    this.content = const {},
    this.changeSummary,
    this.createdAt,
    this.restoredFromVersion,
    this.contentSize = 0,
  });

  /// Creates a copy of the version with updated fields
  DocumentVersion copyWith({
    String? id,
    String? documentId,
    int? versionNumber,
    String? title,
    Map<String, dynamic>? content,
    String? changeSummary,
    String? createdBy,
    DateTime? createdAt,
    Map<String, dynamic>? vectorClock,
    int? restoredFromVersion,
    int? contentSize,
  }) =>
      DocumentVersion(
        id: id ?? this.id,
        documentId: documentId ?? this.documentId,
        versionNumber: versionNumber ?? this.versionNumber,
        title: title ?? this.title,
        content: content ?? this.content,
        changeSummary: changeSummary ?? this.changeSummary,
        createdBy: createdBy ?? this.createdBy,
        createdAt: createdAt ?? this.createdAt,
        vectorClock: vectorClock ?? this.vectorClock,
        restoredFromVersion: restoredFromVersion ?? this.restoredFromVersion,
        contentSize: contentSize ?? this.contentSize,
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

  /// Creates a DocumentVersion from JSON (for API responses)
  factory DocumentVersion.fromJson(Map<String, dynamic> json) {
    final contentJson = json['content'] as Map<String, dynamic>?;
    final vectorClockJson = json['vector_clock'] as Map<String, dynamic>?;

    return DocumentVersion(
      id: json['id'] as String,
      documentId: json['document_id'] as String,
      versionNumber: json['version_number'] as int,
      title: json['title'] as String,
      content: contentJson ?? {},
      changeSummary: json['change_summary'] as String?,
      createdBy: json['created_by'] as String,
      createdAt: _parseDateTime(json['created_at'] as String?),
      vectorClock: vectorClockJson ?? {},
      restoredFromVersion: json['restored_from_version'] as int?,
      contentSize: json['content_size'] as int? ?? 0,
    );
  }

  /// Converts DocumentVersion to JSON (for API requests)
  Map<String, dynamic> toJson() {
    final jsonMap = <String, dynamic>{
      'id': id,
      'document_id': documentId,
      'version_number': versionNumber,
      'title': title,
      'content': content,
      'created_by': createdBy,
      'vector_clock': vectorClock,
      'content_size': contentSize,
    };

    if (changeSummary != null) {
      jsonMap['change_summary'] = changeSummary;
    }

    if (createdAt != null) {
      jsonMap['created_at'] = createdAt!.toIso8601String();
    }

    if (restoredFromVersion != null) {
      jsonMap['restored_from_version'] = restoredFromVersion;
    }

    return jsonMap;
  }

  @override
  String toString() =>
      'DocumentVersion(id: $id, version: $versionNumber, title: $title)';

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is DocumentVersion && other.id == id;
  }

  @override
  int get hashCode => id.hashCode;
}
