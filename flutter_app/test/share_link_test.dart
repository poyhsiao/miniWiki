import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/share_link.dart';

void main() {
  group('ShareLink Entity Tests', () {
    test('ShareLink can be created with all fields', () {
      // Arrange & Act
      final shareLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: true,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        expiresAt: DateTime(2025),
        accessCount: 5,
        isActive: true,
        maxAccessCount: 100,
      );

      // Assert
      expect(shareLink.id, 'share1');
      expect(shareLink.documentId, 'doc1');
      expect(shareLink.documentTitle, 'Test Document');
      expect(shareLink.token, 'abc123');
      expect(shareLink.permission, 'view');
      expect(shareLink.createdBy, 'user1');
      expect(shareLink.isActive, true);
      expect(shareLink.accessCount, 5);
      expect(shareLink.requiresAccessCode, true);
      expect(shareLink.maxAccessCount, 100);
    });

    test('ShareLink can be created with minimal fields', () {
      // Arrange & Act
      final shareLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'xyz789',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        isActive: true,
        accessCount: 0,
      );

      // Assert
      expect(shareLink.id, 'share1');
      expect(shareLink.expiresAt, isNull);
      expect(shareLink.isActive, true);
      expect(shareLink.accessCount, 0);
      expect(shareLink.maxAccessCount, isNull);
      expect(shareLink.requiresAccessCode, false);
    });

    test('ShareLink isExpired returns correct status', () {
      // Arrange
      final now = DateTime.now();
      final expiredLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: now,
        expiresAt: now.subtract(const Duration(days: 1)), // Expired yesterday
        isActive: true,
        accessCount: 0,
      );

      final activeLink = ShareLink(
        id: 'share2',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'xyz789',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: now,
        expiresAt: now.add(const Duration(days: 1)), // Expires tomorrow
        isActive: true,
        accessCount: 0,
      );

      final noExpiryLink = ShareLink(
        id: 'share3',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'nop123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: now,
        isActive: true,
        accessCount: 0,
      );

      // Assert
      expect(expiredLink.isExpired, true);
      expect(activeLink.isExpired, false);
      expect(noExpiryLink.isExpired, false);
    });

    test('ShareLink hasReachedMaxAccess returns correct status', () {
      // Arrange
      final reachedLimitLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        maxAccessCount: 10,
        accessCount: 10,
        isActive: true,
      );

      final underLimitLink = ShareLink(
        id: 'share2',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'xyz789',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        maxAccessCount: 100,
        accessCount: 25,
        isActive: true,
      );

      final noLimitLink = ShareLink(
        id: 'share3',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'nop123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        accessCount: 999,
        isActive: true,
      );

      // Assert
      expect(reachedLimitLink.hasReachedMaxAccess, true);
      expect(underLimitLink.hasReachedMaxAccess, false);
      expect(noLimitLink.hasReachedMaxAccess, false);
    });

    test('ShareLink isUsable returns correct status', () {
      // Arrange
      final now = DateTime.now();
      final usableLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: now,
        expiresAt: now.add(const Duration(days: 1)),
        maxAccessCount: 100,
        accessCount: 5,
        isActive: true,
      );

      final inactiveLink = ShareLink(
        id: 'share2',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'xyz789',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: now,
        expiresAt: now.add(const Duration(days: 1)),
        accessCount: 5,
        isActive: false,
      );

      final expiredLink = ShareLink(
        id: 'share3',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'nop123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: now,
        expiresAt: now.subtract(const Duration(days: 1)),
        accessCount: 5,
        isActive: true,
      );

      final reachedMaxAccessLink = ShareLink(
        id: 'share4',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'max123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: now,
        maxAccessCount: 10,
        accessCount: 10,
        isActive: true,
      );

      // Assert
      expect(usableLink.isUsable, true);
      expect(inactiveLink.isUsable, false);
      expect(expiredLink.isUsable, false);
      expect(reachedMaxAccessLink.isUsable, false);
    });

    test('ShareLink toJson creates correct JSON', () {
      // Arrange
      final shareLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: true,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        expiresAt: DateTime(2025),
        accessCount: 10,
        isActive: true,
        maxAccessCount: 100,
      );

      // Act
      final json = shareLink.toJson();

      // Assert
      expect(json['id'], 'share1');
      expect(json['document_id'], 'doc1');
      expect(json['document_title'], 'Test Document');
      expect(json['token'], 'abc123');
      expect(json['permission'], 'view');
      expect(json['created_by'], 'user1');
      expect(json['requires_access_code'], true);
      expect(json['expires_at'], isNotNull);
      expect(json['access_count'], 10);
      expect(json['max_access_count'], 100);
      expect(json['is_active'], true);
    });

    test('ShareLink fromJson creates instance correctly', () {
      // Arrange
      final json = {
        'id': 'share1',
        'document_id': 'doc1',
        'document_title': 'Test Document',
        'token': 'abc123',
        'permission': 'view',
        'created_by': 'user1',
        'created_at': '2024-01-01T00:00:00Z',
        'expires_at': '2025-01-01T00:00:00Z',
        'access_count': 15,
        'max_access_count': 100,
        'is_active': true,
        'requires_access_code': false,
      };

      // Act
      final shareLink = ShareLink.fromJson(json);

      // Assert
      expect(shareLink.id, 'share1');
      expect(shareLink.documentTitle, 'Test Document');
      expect(shareLink.token, 'abc123');
      expect(shareLink.accessCount, 15);
      expect(shareLink.maxAccessCount, 100);
      expect(shareLink.isActive, true);
      expect(shareLink.requiresAccessCode, false);
    });

    test('ShareLink fromJson handles null optional fields', () {
      // Arrange
      final json = {
        'id': 'share1',
        'document_id': 'doc1',
        'document_title': 'Test Document',
        'token': 'abc123',
        'permission': 'view',
        'created_by': 'user1',
        'created_at': '2024-01-01T00:00:00Z',
      };

      // Act
      final shareLink = ShareLink.fromJson(json);

      // Assert
      expect(shareLink.id, 'share1');
      expect(shareLink.expiresAt, isNull);
      expect(shareLink.maxAccessCount, isNull);
      expect(shareLink.isActive, true);
      expect(shareLink.requiresAccessCode, false);
      expect(shareLink.accessCount, 0);
    });

    test('ShareLink fromJson uses default createdBy when missing', () {
      // Arrange
      final json = {
        'id': 'share1',
        'document_id': 'doc1',
        'document_title': 'Test Document',
        'token': 'abc123',
        'permission': 'view',
        'created_at': '2024-01-01T00:00:00Z',
      };

      // Act
      final shareLink = ShareLink.fromJson(json);

      // Assert
      expect(shareLink.createdBy, 'Unknown');
    });

    test('ShareLink equality works correctly', () {
      // Arrange
      final link1 = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        isActive: true,
        accessCount: 0,
      );

      final link2 = ShareLink(
        id: 'share1',
        documentId: 'doc2',
        documentTitle: 'Different Document',
        token: 'xyz789',
        requiresAccessCode: true,
        permission: 'comment',
        createdBy: 'user2',
        createdAt: DateTime(2025),
        isActive: false,
        accessCount: 5,
      );

      final link3 = ShareLink(
        id: 'share2',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        isActive: true,
        accessCount: 0,
      );

      // Assert
      expect(link1, equals(link2)); // Same ID
      expect(link1, isNot(equals(link3))); // Different ID
      expect(link1.hashCode, equals(link2.hashCode));
    });

    test('ShareLink toString returns formatted string', () {
      // Arrange & Act
      final shareLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123456789',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        isActive: true,
        accessCount: 0,
      );

      // Assert
      expect(
        shareLink.toString(),
        'ShareLink(id: share1, documentId: doc1, token: abc12345...)',
      );
    });

    test('ShareLink toString with short token', () {
      // Arrange & Act
      final shareLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        isActive: true,
        accessCount: 0,
      );

      // Assert
      expect(
        shareLink.toString(),
        'ShareLink(id: share1, documentId: doc1, token: abc123)',
      );
    });

    test('ShareLink can be comment permission', () {
      // Arrange & Act
      final shareLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: false,
        permission: 'comment',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        isActive: true,
        accessCount: 0,
      );

      // Assert
      expect(shareLink.permission, 'comment');
    });

    test('ShareLink getShareUrl returns correct URL', () {
      // Arrange
      final shareLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: false,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        isActive: true,
        accessCount: 0,
      );

      // Act
      final url = shareLink.getShareUrl('https://example.com');

      // Assert
      expect(url, 'https://example.com/share/abc123');
    });

    test('ShareLink with requiresAccessCode true', () {
      // Arrange & Act
      final shareLink = ShareLink(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        token: 'abc123',
        requiresAccessCode: true,
        permission: 'view',
        createdBy: 'user1',
        createdAt: DateTime(2024),
        isActive: true,
        accessCount: 0,
      );

      // Assert
      expect(shareLink.requiresAccessCode, true);
    });
  });

  group('CreateShareLinkRequest Tests', () {
    test('CreateShareLinkRequest can be created with all fields', () {
      // Arrange & Act
      final request = CreateShareLinkRequest(
        documentId: 'doc1',
        accessCode: '1234',
        expiresAt: DateTime(2025),
        permission: 'comment',
        maxAccessCount: 50,
      );

      // Assert
      expect(request.documentId, 'doc1');
      expect(request.accessCode, '1234');
      expect(request.permission, 'comment');
      expect(request.maxAccessCount, 50);
    });

    test('CreateShareLinkRequest default permission is view', () {
      // Arrange & Act
      const request = CreateShareLinkRequest(
        documentId: 'doc1',
      );

      // Assert
      expect(request.permission, 'view');
    });

    test('CreateShareLinkRequest toJson creates correct JSON', () {
      // Arrange
      final request = CreateShareLinkRequest(
        documentId: 'doc1',
        accessCode: '1234',
        expiresAt: DateTime(2025),
        permission: 'comment',
        maxAccessCount: 50,
      );

      // Act
      final json = request.toJson();

      // Assert
      expect(json['document_id'], 'doc1');
      expect(json['access_code'], '1234');
      expect(json['permission'], 'comment');
      expect(json['max_access_count'], 50);
      expect(json['expires_at'], isNotNull);
    });
  });

  group('ShareLinkVerification Tests', () {
    test('ShareLinkVerification can be created with all fields', () {
      // Arrange & Act
      final verification = ShareLinkVerification(
        id: 'share1',
        documentId: 'doc1',
        documentTitle: 'Test Document',
        documentContent: <String, dynamic>{'delta': <dynamic>[]},
        permission: 'view',
        verified: true,
        expiresAt: DateTime(2025),
      );

      // Assert
      expect(verification.id, 'share1');
      expect(verification.documentId, 'doc1');
      expect(verification.documentTitle, 'Test Document');
      expect(verification.permission, 'view');
      expect(verification.verified, true);
    });

    test('ShareLinkVerification fromJson creates instance correctly', () {
      // Arrange
      final json = {
        'id': 'share1',
        'document_id': 'doc1',
        'document_title': 'Test Document',
        'document_content': <String, dynamic>{'delta': <dynamic>[]},
        'permission': 'view',
        'verified': true,
        'expires_at': '2025-01-01T00:00:00Z',
      };

      // Act
      final verification = ShareLinkVerification.fromJson(json);

      // Assert
      expect(verification.id, 'share1');
      expect(verification.documentId, 'doc1');
      expect(verification.documentTitle, 'Test Document');
      expect(verification.permission, 'view');
      expect(verification.verified, true);
    });

    test('ShareLinkVerification fromJson uses default verified when missing', () {
      // Arrange
      final json = {
        'id': 'share1',
        'document_id': 'doc1',
        'document_title': 'Test Document',
        'document_content': <String, dynamic>{'delta': <dynamic>[]},
        'permission': 'view',
      };

      // Act
      final verification = ShareLinkVerification.fromJson(json);

      // Assert
      expect(verification.verified, true);
    });
  });
}
