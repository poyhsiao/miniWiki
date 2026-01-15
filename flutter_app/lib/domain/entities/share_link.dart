/// Share link entity for document sharing functionality.
///
/// Represents a shareable link that allows external users to access
/// a document without authentication. Share links can be password-protected
/// and have optional expiration dates and access limits.
class ShareLink {
  /// Unique identifier for the share link
  final String id;

  /// ID of the document being shared
  final String documentId;

  /// Title of the shared document (for display purposes)
  final String documentTitle;

  /// The share token used in URLs
  final String token;

  /// Whether an access code is required to view this share link
  final bool requiresAccessCode;

  /// When the share link expires (null means never expires)
  final DateTime? expiresAt;

  /// Permission level: 'view' or 'comment'
  final String permission;

  /// Whether the share link is currently active
  final bool isActive;

  /// When the share link was created
  final DateTime createdAt;

  /// Number of times this share link has been accessed
  final int accessCount;

  /// Maximum number of times this share link can be accessed (null means unlimited)
  final int? maxAccessCount;

  /// Who created the share link (display name)
  final String createdBy;

  const ShareLink({
    required this.id,
    required this.documentId,
    required this.documentTitle,
    required this.token,
    required this.requiresAccessCode,
    this.expiresAt,
    required this.permission,
    required this.isActive,
    required this.createdAt,
    required this.accessCount,
    this.maxAccessCount,
    required this.createdBy,
  });

  /// Create a ShareLink from JSON data
  factory ShareLink.fromJson(Map<String, dynamic> json) {
    return ShareLink(
      id: json['id'] as String,
      documentId: json['document_id'] as String,
      documentTitle: json['document_title'] as String,
      token: json['token'] as String,
      requiresAccessCode: json['requires_access_code'] as bool? ?? false,
      expiresAt: json['expires_at'] != null
          ? DateTime.parse(json['expires_at'] as String)
          : null,
      permission: json['permission'] as String,
      isActive: json['is_active'] as bool? ?? true,
      createdAt: DateTime.parse(json['created_at'] as String),
      accessCount: json['access_count'] as int? ?? 0,
      maxAccessCount: json['max_access_count'] as int?,
      createdBy: json['created_by'] as String? ?? 'Unknown',
    );
  }

  /// Convert ShareLink to JSON
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'document_id': documentId,
      'document_title': documentTitle,
      'token': token,
      'requires_access_code': requiresAccessCode,
      'expires_at': expiresAt?.toIso8601String(),
      'permission': permission,
      'is_active': isActive,
      'created_at': createdAt.toIso8601String(),
      'access_count': accessCount,
      'max_access_count': maxAccessCount,
      'created_by': createdBy,
    };
  }

  /// Get the full share URL for this link
  String getShareUrl(String baseUrl) {
    return '$baseUrl/share/$token';
  }

  /// Check if the share link has expired
  bool get isExpired {
    if (expiresAt == null) return false;
    return DateTime.now().isAfter(expiresAt!);
  }

  /// Check if the share link has reached its maximum access count
  bool get hasReachedMaxAccess {
    final max = maxAccessCount;
    if (max == null) return false;
    return accessCount >= max;
  }

  /// Check if this share link can be used
  bool get isUsable {
    return isActive && !isExpired && !hasReachedMaxAccess;
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is ShareLink && other.id == id;
  }

  @override
  int get hashCode => id.hashCode;

  @override
  String toString() {
    final tokenPreview = token.length <= 8 ? token : '${token.substring(0, 8)}...';
    return 'ShareLink(id: $id, documentId: $documentId, token: $tokenPreview)';
  }
}

/// Request to create a new share link
class CreateShareLinkRequest {
  /// ID of the document to share
  final String documentId;

  /// Optional access code for password protection (4-10 characters)
  final String? accessCode;

  /// Optional expiration date
  final DateTime? expiresAt;

  /// Permission level: 'view' or 'comment'
  final String permission;

  /// Optional maximum number of accesses
  final int? maxAccessCount;

  const CreateShareLinkRequest({
    required this.documentId,
    this.accessCode,
    this.expiresAt,
    this.permission = 'view',
    this.maxAccessCount,
  });

  Map<String, dynamic> toJson() {
    return {
      'document_id': documentId,
      'access_code': accessCode,
      'expires_at': expiresAt?.toIso8601String(),
      'permission': permission,
      'max_access_count': maxAccessCount,
    };
  }
}

/// Response when verifying an access code
class ShareLinkVerification {
  /// The share link ID
  final String id;

  /// The document ID
  final String documentId;

  /// The document title
  final String documentTitle;

  /// The document content (Yjs CRDT state)
  final Map<String, dynamic> documentContent;

  /// Permission level
  final String permission;

  /// Expiration date
  final DateTime? expiresAt;

  /// Whether verification was successful
  final bool verified;

  const ShareLinkVerification({
    required this.id,
    required this.documentId,
    required this.documentTitle,
    required this.documentContent,
    required this.permission,
    this.expiresAt,
    required this.verified,
  });

  factory ShareLinkVerification.fromJson(Map<String, dynamic> json) {
    return ShareLinkVerification(
      id: json['id'] as String,
      documentId: json['document_id'] as String,
      documentTitle: json['document_title'] as String,
      documentContent:
          Map<String, dynamic>.from(json['document_content'] as Map),
      permission: json['permission'] as String,
      expiresAt: json['expires_at'] != null
          ? DateTime.parse(json['expires_at'] as String)
          : null,
      verified: json['verified'] as bool? ?? true,
    );
  }
}
