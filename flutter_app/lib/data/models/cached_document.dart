import 'package:isar/isar.dart';

part 'cached_document.g.dart';

/// Cached document entity for offline access
/// Stores document content and metadata for offline-first functionality
@Collection()
class CachedDocument {
  /// Auto-increment ID
  Id get id => Isar.autoIncrement;

  /// Document ID (unique identifier)
  @Index()
  String documentId = '';

  /// Document title (for display in offline mode)
  String title = '';

  /// Space ID this document belongs to
  @Index()
  String spaceId = '';

  /// Parent document ID for hierarchical structure
  String? parentId;

  /// Cached document content (CRDT state bytes) - stored as List<int for Isar
  List<int>? content;

  /// Cached content as JSON string (for non-binary content)
  String? contentJson;

  /// Last synced state vector
  String? stateVector;

  /// Document version at cache time
  int version = 0;

  /// When the document was last modified
  DateTime? modifiedAt;

  /// When the document was cached
  DateTime cachedAt = DateTime.now();

  /// When the cache expires (for TTL-based invalidation)
  DateTime? expiresAt;

  /// Whether this document has un-synced changes
  bool isDirty = false;

  /// Cache priority (higher = more likely to be retained)
  int priority = 0;

  /// Create a cached document
  CachedDocument({
    required this.documentId,
    this.title = '',
    this.spaceId = '',
    this.parentId,
    this.content,
    this.contentJson,
    this.stateVector,
    this.version = 0,
    this.modifiedAt,
    this.expiresAt,
    this.isDirty = false,
    this.priority = 0,
  }) : cachedAt = DateTime.now();

  /// Check if cache is expired
  bool get isExpired {
    if (expiresAt == null) return false;
    return DateTime.now().isAfter(expiresAt!);
  }

  /// Check if cache is valid (not expired and has content)
  bool get isValid {
    return !isExpired && (content != null || contentJson != null);
  }

  /// Get cache age in milliseconds
  int get ageMs {
    return DateTime.now().difference(cachedAt).inMilliseconds;
  }

  /// Get time until expiry in milliseconds
  int? get ttlMs {
    if (expiresAt == null) return null;
    final remaining = expiresAt!.difference(DateTime.now());
    return remaining.inMilliseconds;
  }

  /// Update content and mark as dirty
  void updateContent(List<int> newContent, {int? newVersion}) {
    content = newContent;
    if (newVersion != null) {
      version = newVersion;
    }
    modifiedAt = DateTime.now();
    isDirty = true;
    cachedAt = DateTime.now();
  }

  /// Mark as synced
  void markSynced(String? newStateVector) {
    isDirty = false;
    stateVector = newStateVector;
    modifiedAt = DateTime.now();
  }

  /// Extend cache TTL
  void extendTtl(Duration duration) {
    expiresAt = DateTime.now().add(duration);
  }
}
