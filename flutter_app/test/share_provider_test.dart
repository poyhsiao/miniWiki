// ShareProvider unit tests
// Testing ShareLinksState and ShareLinkCreateState copyWith methods
// Run with: flutter test test/share_provider_test.dart

import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/share_link.dart';
import 'package:miniwiki/presentation/providers/share_provider.dart';

void main() {
  group('ShareLinksState - copyWith', () {
    test('copyWith can explicitly clear error field', () {
      final state = ShareLinksState(
        documentId: 'doc-id',
        error: 'Some error',
      );

      final newState = state.copyWith(error: null);

      expect(newState.error, isNull);
    });

    test('copyWith preserves error when not specified', () {
      final state = ShareLinksState(
        documentId: 'doc-id',
        error: 'Some error',
      );

      final newState = state.copyWith(isLoading: true);

      expect(newState.error, 'Some error');
    });

    test('copyWith can set new error', () {
      final state = ShareLinksState(
        documentId: 'doc-id',
      );

      final newState = state.copyWith(error: 'New error');

      expect(newState.error, 'New error');
    });
  });

  group('ShareLinkCreateState - copyWith', () {
    test('copyWith can explicitly clear error field', () {
      final state = ShareLinkCreateState(
        error: 'Some error',
      );

      final newState = state.copyWith(error: null);

      expect(newState.error, isNull);
    });

    test('copyWith can explicitly clear createdLink field', () {
      final link = ShareLink(
        id: 'test-id',
        documentId: 'doc-id',
        documentTitle: 'Test Doc',
        token: 'test-token',
        requiresAccessCode: false,
        permission: 'view',
        isActive: true,
        createdAt: DateTime.now(),
        accessCount: 0,
        createdBy: 'Test User',
      );

      final state = ShareLinkCreateState(
        createdLink: link,
      );

      final newState = state.copyWith(createdLink: null);

      expect(newState.createdLink, isNull);
    });

    test('copyWith can explicitly clear expiresAt field', () {
      final state = ShareLinkCreateState(
        expiresAt: DateTime.now(),
      );

      final newState = state.copyWith(expiresAt: null);

      expect(newState.expiresAt, isNull);
    });

    test('copyWith can explicitly clear maxAccessCount field', () {
      final state = ShareLinkCreateState(
        maxAccessCount: 10,
      );

      final newState = state.copyWith(maxAccessCount: null);

      expect(newState.maxAccessCount, isNull);
    });

    test('copyWith preserves fields when not specified', () {
      final link = ShareLink(
        id: 'test-id',
        documentId: 'doc-id',
        documentTitle: 'Test Doc',
        token: 'test-token',
        requiresAccessCode: false,
        permission: 'view',
        isActive: true,
        createdAt: DateTime.now(),
        accessCount: 0,
        createdBy: 'Test User',
      );

      final state = ShareLinkCreateState(
        error: 'Some error',
        createdLink: link,
        expiresAt: DateTime.now(),
        maxAccessCount: 10,
      );

      final newState = state.copyWith(permission: 'comment');

      expect(newState.error, 'Some error');
      expect(newState.createdLink, link);
      expect(newState.expiresAt, isNotNull);
      expect(newState.maxAccessCount, 10);
    });
  });
}
