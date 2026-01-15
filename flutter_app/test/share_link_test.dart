// ShareLink entity unit tests
// Testing ShareLink entity methods including toString, toJson, fromJson
// Run with: flutter test test/share_link_test.dart

import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/share_link.dart';

void main() {
  group('ShareLink - toString', () {
    test('toString handles short token correctly', () {
      final link = ShareLink(
        id: 'test-id',
        documentId: 'doc-id',
        documentTitle: 'Test Doc',
        token: 'short',
        requiresAccessCode: false,
        permission: 'view',
        isActive: true,
        createdAt: DateTime.now(),
        accessCount: 0,
        createdBy: 'Test User',
      );

      final result = link.toString();
      expect(result, contains('short'));
      expect(result, isNot(contains('...')));
    });

    test('toString handles long token correctly', () {
      final link = ShareLink(
        id: 'test-id',
        documentId: 'doc-id',
        documentTitle: 'Test Doc',
        token: 'verylongtoken123456789',
        requiresAccessCode: false,
        permission: 'view',
        isActive: true,
        createdAt: DateTime.now(),
        accessCount: 0,
        createdBy: 'Test User',
      );

      final result = link.toString();
      expect(result, contains('verylong...'));
    });

    test('toString handles exactly 8 character token', () {
      final link = ShareLink(
        id: 'test-id',
        documentId: 'doc-id',
        documentTitle: 'Test Doc',
        token: '12345678',
        requiresAccessCode: false,
        permission: 'view',
        isActive: true,
        createdAt: DateTime.now(),
        accessCount: 0,
        createdBy: 'Test User',
      );

      final result = link.toString();
      expect(result, contains('12345678'));
    });
  });

  group('ShareLink - toJson/fromJson', () {
    test('toJson produces snake_case keys', () {
      final link = ShareLink(
        id: 'test-id',
        documentId: 'doc-id',
        documentTitle: 'Test Doc',
        token: 'test-token',
        requiresAccessCode: true,
        permission: 'comment',
        isActive: true,
        createdAt: DateTime.parse('2024-01-01T00:00:00Z'),
        accessCount: 5,
        maxAccessCount: 10,
        createdBy: 'Test User',
      );

      final json = link.toJson();

      expect(json['document_id'], 'doc-id');
      expect(json['document_title'], 'Test Doc');
      expect(json['requires_access_code'], true);
      expect(json['is_active'], true);
      expect(json['created_at'], isNotNull);
      expect(json['access_count'], 5);
      expect(json['max_access_count'], 10);
      expect(json['created_by'], 'Test User');

      // Should not have camelCase keys
      expect(json.containsKey('documentId'), false);
      expect(json.containsKey('accessCode'), false);
      expect(json.containsKey('expiresAt'), false);
      expect(json.containsKey('maxAccessCount'), false);
    });

    test('fromJson and toJson are symmetric', () {
      final originalJson = {
        'id': 'test-id',
        'document_id': 'doc-id',
        'document_title': 'Test Doc',
        'token': 'test-token',
        'requires_access_code': true,
        'expires_at': '2024-12-31T23:59:59Z',
        'permission': 'view',
        'is_active': true,
        'created_at': '2024-01-01T00:00:00Z',
        'access_count': 3,
        'max_access_count': 5,
        'created_by': 'Test User',
      };

      final link = ShareLink.fromJson(originalJson);
      final resultJson = link.toJson();

      expect(resultJson['id'], originalJson['id']);
      expect(resultJson['document_id'], originalJson['document_id']);
      expect(resultJson['token'], originalJson['token']);
      expect(resultJson['permission'], originalJson['permission']);
      expect(resultJson['access_count'], originalJson['access_count']);
      expect(resultJson['max_access_count'], originalJson['max_access_count']);
    });
  });

  group('CreateShareLinkRequest - toJson', () {
    test('toJson produces snake_case keys', () {
      final request = CreateShareLinkRequest(
        documentId: 'doc-id',
        accessCode: 'secret',
        expiresAt: DateTime.parse('2024-12-31T23:59:59Z'),
        permission: 'comment',
        maxAccessCount: 10,
      );

      final json = request.toJson();

      expect(json['document_id'], 'doc-id');
      expect(json['access_code'], 'secret');
      expect(json['expires_at'], isNotNull);
      expect(json['permission'], 'comment');
      expect(json['max_access_count'], 10);

      // Should not have camelCase keys
      expect(json.containsKey('documentId'), false);
      expect(json.containsKey('accessCode'), false);
      expect(json.containsKey('expiresAt'), false);
      expect(json.containsKey('maxAccessCount'), false);
    });
  });
}
