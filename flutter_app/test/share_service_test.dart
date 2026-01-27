import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/share_link.dart';
import 'package:miniwiki/domain/repositories/share_repository.dart';
import 'package:miniwiki/services/share_service.dart';
import 'package:mocktail/mocktail.dart';

class MockShareRepository extends Mock implements ShareRepository {}

void main() {
  setUpAll(() {
    // Register fallback values for mocktail
    registerFallbackValue(
      const CreateShareLinkRequest(documentId: 'test-doc'),
    );
  });

  group('ShareService Tests', () {
    late ShareService shareService;
    late MockShareRepository mockRepository;

    setUp(() {
      mockRepository = MockShareRepository();
      shareService = ShareService(mockRepository, 'https://example.com');
    });

    group('createShareLink', () {
      test('creates share link with default permission', () async {
        final testLink = ShareLink(
          id: 'link-1',
          documentId: 'doc-1',
          documentTitle: 'Test Document',
          token: 'abc123',
          permission: 'view',
          isActive: true,
          createdAt: DateTime.now(),
          createdBy: 'user-1',
          requiresAccessCode: false,
          accessCount: 0,
        );

        when(() => mockRepository.createShareLink(any()))
            .thenAnswer((_) async => testLink);

        final result = await shareService.createShareLink(documentId: 'doc-1');

        expect(result.token, 'abc123');
        expect(result.permission, 'view');

        final captured = verify(() => mockRepository.createShareLink(captureAny()))
            .captured.single as CreateShareLinkRequest;
        expect(captured.documentId, 'doc-1');
        expect(captured.permission, 'view');
      });

      test('creates share link with custom permission', () async {
        final testLink = ShareLink(
          id: 'link-1',
          documentId: 'doc-1',
          documentTitle: 'Test Document',
          token: 'abc123',
          permission: 'comment',
          isActive: true,
          createdAt: DateTime.now(),
          createdBy: 'user-1',
          requiresAccessCode: false,
          accessCount: 0,
        );

        when(() => mockRepository.createShareLink(any()))
            .thenAnswer((_) async => testLink);

        await shareService.createShareLink(
          documentId: 'doc-1',
          permission: 'comment',
        );

        final captured = verify(() => mockRepository.createShareLink(captureAny()))
            .captured.single as CreateShareLinkRequest;
        expect(captured.permission, 'comment');
      });

      test('creates share link with access code', () async {
        final testLink = ShareLink(
          id: 'link-1',
          documentId: 'doc-1',
          documentTitle: 'Test Document',
          token: 'abc123',
          permission: 'view',
          requiresAccessCode: true,
          isActive: true,
          createdAt: DateTime.now(),
          createdBy: 'user-1',
          accessCount: 0,
        );

        when(() => mockRepository.createShareLink(any()))
            .thenAnswer((_) async => testLink);

        await shareService.createShareLink(
          documentId: 'doc-1',
          accessCode: 'secret123',
        );

        final captured = verify(() => mockRepository.createShareLink(captureAny()))
            .captured.single as CreateShareLinkRequest;
        expect(captured.accessCode, 'secret123');
      });

      test('creates share link with expiration', () async {
        final expiration = DateTime.now().add(const Duration(days: 7));
        final testLink = ShareLink(
          id: 'link-1',
          documentId: 'doc-1',
          documentTitle: 'Test Document',
          token: 'abc123',
          permission: 'view',
          expiresAt: expiration,
          isActive: true,
          createdAt: DateTime.now(),
          createdBy: 'user-1',
          requiresAccessCode: false,
          accessCount: 0,
        );

        when(() => mockRepository.createShareLink(any()))
            .thenAnswer((_) async => testLink);

        await shareService.createShareLink(
          documentId: 'doc-1',
          expiresAt: expiration,
        );

        final captured = verify(() => mockRepository.createShareLink(captureAny()))
            .captured.single as CreateShareLinkRequest;
        expect(captured.expiresAt, expiration);
      });

      test('creates share link with max access count', () async {
        final testLink = ShareLink(
          id: 'link-1',
          documentId: 'doc-1',
          documentTitle: 'Test Document',
          token: 'abc123',
          permission: 'view',
          maxAccessCount: 100,
          isActive: true,
          createdAt: DateTime.now(),
          createdBy: 'user-1',
          requiresAccessCode: false,
          accessCount: 0,
        );

        when(() => mockRepository.createShareLink(any()))
            .thenAnswer((_) async => testLink);

        await shareService.createShareLink(
          documentId: 'doc-1',
          maxAccessCount: 100,
        );

        final captured = verify(() => mockRepository.createShareLink(captureAny()))
            .captured.single as CreateShareLinkRequest;
        expect(captured.maxAccessCount, 100);
      });
    });

    group('getShareLinks', () {
      test('returns list of share links', () async {
        final testLinks = [
          ShareLink(
            id: 'link-1',
            documentId: 'doc-1',
            documentTitle: 'Test Document',
            token: 'abc123',
            permission: 'view',
            isActive: true,
            createdAt: DateTime.now(),
            createdBy: 'user-1',
            requiresAccessCode: false,
            accessCount: 0,
          ),
          ShareLink(
            id: 'link-2',
            documentId: 'doc-1',
            documentTitle: 'Test Document',
            token: 'def456',
            permission: 'comment',
            isActive: true,
            createdAt: DateTime.now(),
            createdBy: 'user-2',
            requiresAccessCode: false,
            accessCount: 0,
          ),
        ];

        when(() => mockRepository.getShareLinks('doc-1'))
            .thenAnswer((_) async => testLinks);

        final result = await shareService.getShareLinks('doc-1');

        expect(result.length, 2);
        expect(result[0].token, 'abc123');
        expect(result[1].token, 'def456');

        verify(() => mockRepository.getShareLinks('doc-1')).called(1);
      });

      test('returns empty list when no links exist', () async {
        when(() => mockRepository.getShareLinks('doc-1'))
            .thenAnswer((_) async => []);

        final result = await shareService.getShareLinks('doc-1');

        expect(result, isEmpty);
      });
    });

    group('deleteShareLink', () {
      test('deletes share link successfully', () async {
        when(() => mockRepository.deleteShareLink('doc-1', 'abc123'))
            .thenAnswer((_) async {});

        await shareService.deleteShareLink('doc-1', 'abc123');

        verify(() => mockRepository.deleteShareLink('doc-1', 'abc123')).called(1);
      });
    });

    group('copyShareLink', () {
      test('copies share link to clipboard', () async {
        final testLink = ShareLink(
          id: 'link-1',
          documentId: 'doc-1',
          documentTitle: 'Test Document',
          token: 'abc123',
          permission: 'view',
          isActive: true,
          createdAt: DateTime.now(),
          createdBy: 'user-1',
          requiresAccessCode: false,
          accessCount: 0,
        );

        when(() => mockRepository.copyShareLinkToClipboard(testLink, 'https://example.com'))
            .thenAnswer((_) async => true);

        final result = await shareService.copyShareLink(testLink);

        expect(result, true);
        verify(() => mockRepository.copyShareLinkToClipboard(testLink, 'https://example.com'))
            .called(1);
      });

      test('handles clipboard copy failure', () async {
        final testLink = ShareLink(
          id: 'link-1',
          documentId: 'doc-1',
          documentTitle: 'Test Document',
          token: 'abc123',
          permission: 'view',
          isActive: true,
          createdAt: DateTime.now(),
          createdBy: 'user-1',
          requiresAccessCode: false,
          accessCount: 0,
        );

        when(() => mockRepository.copyShareLinkToClipboard(testLink, 'https://example.com'))
            .thenAnswer((_) async => false);

        final result = await shareService.copyShareLink(testLink);

        expect(result, false);
      });
    });

    group('getShareLinkByToken', () {
      test('returns share link for valid token', () async {
        final testLink = ShareLink(
          id: 'link-1',
          documentId: 'doc-1',
          documentTitle: 'Test Document',
          token: 'abc123',
          permission: 'view',
          isActive: true,
          createdAt: DateTime.now(),
          createdBy: 'user-1',
          requiresAccessCode: false,
          accessCount: 0,
        );

        when(() => mockRepository.getShareLinkByToken('abc123'))
            .thenAnswer((_) async => testLink);

        final result = await shareService.getShareLinkByToken('abc123');

        expect(result, isNotNull);
        expect(result!.token, 'abc123');
      });

      test('returns null for non-existent token', () async {
        when(() => mockRepository.getShareLinkByToken('invalid'))
            .thenAnswer((_) async => null);

        final result = await shareService.getShareLinkByToken('invalid');

        expect(result, isNull);
      });
    });

    group('verifyAccessCode', () {
      test('verifies valid access code', () async {
        const verification = ShareLinkVerification(
          id: 'verify-1',
          documentId: 'doc-1',
          documentTitle: 'Test Document',
          documentContent: {'type': 'Y.Doc'},
          permission: 'view',
          verified: true,
        );

        when(() => mockRepository.verifyAccessCode('abc123', 'code123'))
            .thenAnswer((_) async => verification);

        final result = await shareService.verifyAccessCode('abc123', 'code123');

        expect(result.verified, true);
        expect(result.documentId, 'doc-1');
      });

      test('verifies invalid access code', () async {
        const verification = ShareLinkVerification(
          id: 'verify-1',
          documentId: 'doc-1',
          documentTitle: 'Test Document',
          documentContent: {'type': 'Y.Doc'},
          permission: 'view',
          verified: false,
        );

        when(() => mockRepository.verifyAccessCode('abc123', 'wrong'))
            .thenAnswer((_) async => verification);

        final result = await shareService.verifyAccessCode('abc123', 'wrong');

        expect(result.verified, false);
      });
    });

    group('formatExpiration', () {
      test('returns "Never" for null expiration', () {
        final result = shareService.formatExpiration(null);

        expect(result, 'Never');
      });

      test('returns "Expired" for past date', () {
        final pastDate = DateTime.now().subtract(const Duration(days: 1));

        final result = shareService.formatExpiration(pastDate);

        expect(result, 'Expired');
      });

      test('returns years left for >365 days', () {
        final futureDate = DateTime.now().add(const Duration(days: 400));

        final result = shareService.formatExpiration(futureDate);

        expect(result, contains('y left'));
      });

      test('returns months left for >30 days', () {
        final futureDate = DateTime.now().add(const Duration(days: 60));

        final result = shareService.formatExpiration(futureDate);

        expect(result, contains('mo left'));
      });

      test('returns days left for >0 days', () {
        final futureDate = DateTime.now().add(const Duration(days: 5));

        final result = shareService.formatExpiration(futureDate);

        expect(result, contains('d left'));
      });

      test('returns hours left for >0 hours', () {
        final futureDate = DateTime.now().add(const Duration(hours: 3));

        final result = shareService.formatExpiration(futureDate);

        expect(result, contains('h left'));
      });

      test('returns minutes left for <1 hour', () {
        final futureDate = DateTime.now().add(const Duration(minutes: 30));

        final result = shareService.formatExpiration(futureDate);

        expect(result, contains('m left'));
      });
    });

    group('formatAccessCount', () {
      test('returns accesses without max', () {
        final result = shareService.formatAccessCount(5, null);

        expect(result, '5 accesses');
      });

      test('returns accesses with max', () {
        final result = shareService.formatAccessCount(5, 10);

        expect(result, '5/10 accesses');
      });

      test('returns zero accesses', () {
        final result = shareService.formatAccessCount(0, 100);

        expect(result, '0/100 accesses');
      });
    });

    group('baseUrl', () {
      test('returns configured base URL', () {
        final service = ShareService(mockRepository, 'https://custom.example.com');

        expect(service.baseUrl, 'https://custom.example.com');
      });

      test('returns default base URL when not provided', () {
        final service = ShareService(mockRepository);

        expect(service.baseUrl, 'http://localhost:3000');
      });
    });
  });
}
