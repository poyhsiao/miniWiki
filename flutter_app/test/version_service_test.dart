/// Version service unit tests
///
/// Tests for document version listing, retrieval, and restore functionality.
/// These tests verify the version service works correctly.
///
/// Run with: flutter test test/version_service_test.dart

import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/document_version.dart';
import 'package:miniwiki/services/version_service.dart';
import 'package:miniwiki/domain/repositories/version_repository.dart';
import 'package:miniwiki/core/network/network_error.dart';
import 'package:mocktail/mocktail.dart';

class MockVersionRepository extends Mock implements VersionRepository {}

void main() {
  group('VersionService', () {
    late VersionService versionService;
    late MockVersionRepository mockRepository;

    setUp(() {
      mockRepository = MockVersionRepository();
      versionService = VersionService(versionRepository: mockRepository);
    });

    group('listVersions', () {
      test('returns list of versions for document', () async {
        final versions = [
          DocumentVersion(
            id: 'version-1',
            documentId: 'doc-1',
            versionNumber: 2,
            title: 'Second Version',
            content: {'type': 'Y.Doc', 'update': 'base64update2'},
            changeSummary: 'Added more content',
            createdBy: 'user-1',
            createdAt: DateTime.now().subtract(const Duration(hours: 1)),
            vectorClock: {'client_id': 'user-1', 'clock': 5},
          ),
          DocumentVersion(
            id: 'version-2',
            documentId: 'doc-1',
            versionNumber: 1,
            title: 'First Version',
            content: {'type': 'Y.Doc', 'update': 'base64update1'},
            changeSummary: 'Initial version',
            createdBy: 'user-1',
            createdAt: DateTime.now().subtract(const Duration(hours: 2)),
            vectorClock: {'client_id': 'user-1', 'clock': 3},
          ),
        ];

        when(() => mockRepository.listVersions('doc-1'))
            .thenAnswer((_) async => versions);

        final result = await versionService.listVersions('doc-1');

        expect(result.length, 2);
        expect(result[0].versionNumber, 2); // Most recent first
        expect(result[1].versionNumber, 1);
        verify(() => mockRepository.listVersions('doc-1')).called(1);
      });

      test('returns empty list when document has no versions', () async {
        when(() => mockRepository.listVersions('doc-empty'))
            .thenAnswer((_) async => []);

        final result = await versionService.listVersions('doc-empty');

        expect(result.isEmpty, true);
      });

      test('returns versions with pagination', () async {
        final versions = List.generate(
            10,
            (index) => DocumentVersion(
                  id: 'version-${index + 1}',
                  documentId: 'doc-1',
                  versionNumber: index + 1,
                  title: 'Version ${index + 1}',
                  content: {},
                  changeSummary: null,
                  createdBy: 'user-1',
                  createdAt: DateTime.now(),
                  vectorClock: {},
                ));

        when(() => mockRepository.listVersions('doc-1', limit: 10, offset: 0))
            .thenAnswer((_) async => versions);

        final result = await versionService.listVersions(
          'doc-1',
          limit: 10,
          offset: 0,
        );

        expect(result.length, 10);
      });

      test('throws exception when repository fails', () async {
        when(() => mockRepository.listVersions('doc-1'))
            .thenThrow(NetworkError('Failed to fetch versions', 500));

        expect(
          () => versionService.listVersions('doc-1'),
          throwsA(isA<NetworkError>()),
        );
      });
    });

    group('getVersion', () {
      test('returns version when it exists', () async {
        final version = DocumentVersion(
          id: 'version-1',
          documentId: 'doc-1',
          versionNumber: 1,
          title: 'First Version',
          content: {'type': 'Y.Doc', 'update': 'base64update'},
          changeSummary: 'Initial version',
          createdBy: 'user-1',
          createdAt: DateTime.now(),
          vectorClock: {},
        );

        when(() => mockRepository.getVersion('doc-1', 1))
            .thenAnswer((_) async => version);

        final result = await versionService.getVersion('doc-1', 1);

        expect(result.id, 'version-1');
        expect(result.versionNumber, 1);
        expect(result.title, 'First Version');
      });

      test('throws exception when version not found', () async {
        when(() => mockRepository.getVersion('doc-1', 999))
            .thenThrow(NetworkError('Version not found', 404));

        expect(
          () => versionService.getVersion('doc-1', 999),
          throwsA(isA<NetworkError>()),
        );
      });
    });

    group('createVersion', () {
      test('creates a new version successfully', () async {
        final newVersion = DocumentVersion(
          id: 'version-new',
          documentId: 'doc-1',
          versionNumber: 3,
          title: 'Third Version',
          content: {'type': 'Y.Doc', 'update': 'newcontent'},
          changeSummary: 'Updated content',
          createdBy: 'user-1',
          createdAt: DateTime.now(),
          vectorClock: {'client_id': 'user-1', 'clock': 10},
        );

        when(() => mockRepository.createVersion(
              documentId: 'doc-1',
              title: 'Third Version',
              content: {'type': 'Y.Doc', 'update': 'newcontent'},
              changeSummary: 'Updated content',
              vectorClock: {'client_id': 'user-1', 'clock': 10},
            )).thenAnswer((_) async => newVersion);

        final result = await versionService.createVersion(
          documentId: 'doc-1',
          title: 'Third Version',
          content: {'type': 'Y.Doc', 'update': 'newcontent'},
          changeSummary: 'Updated content',
          vectorClock: {'client_id': 'user-1', 'clock': 10},
        );

        expect(result.versionNumber, 3);
        expect(result.changeSummary, 'Updated content');
      });

      test('throws exception for empty content', () async {
        expect(
          () => versionService.createVersion(
            documentId: 'doc-1',
            title: 'New Version',
            content: {},
            changeSummary: null,
            vectorClock: {},
          ),
          throwsArgumentError,
        );
      });
    });

    group('restoreVersion', () {
      test('restores document to previous version', () async {
        final restoredVersion = DocumentVersion(
          id: 'version-restored',
          documentId: 'doc-1',
          versionNumber: 4,
          title: 'Restored Version',
          content: {'type': 'Y.Doc', 'update': 'oldcontent'},
          changeSummary: 'Restored to version 2',
          createdBy: 'user-1',
          createdAt: DateTime.now(),
          vectorClock: {},
          restoredFromVersion: 2,
        );

        when(() => mockRepository.restoreVersion('doc-1', 2))
            .thenAnswer((_) async => restoredVersion);

        final result = await versionService.restoreVersion('doc-1', 2);

        expect(result.versionNumber, 4);
        expect(result.restoredFromVersion, 2);
        expect(result.changeSummary, 'Restored to version 2');
      });

      test('throws exception when restoring non-existent version', () async {
        when(() => mockRepository.restoreVersion('doc-1', 999))
            .thenThrow(NetworkError('Version not found', 404));

        expect(
          () => versionService.restoreVersion('doc-1', 999),
          throwsA(isA<NetworkError>()),
        );
      });

      test('restored version creates new version number', () async {
        final currentVersion = DocumentVersion(
          id: 'version-current',
          documentId: 'doc-1',
          versionNumber: 5,
          title: 'Current Version',
          content: {},
          changeSummary: null,
          createdBy: 'user-1',
          createdAt: DateTime.now(),
          vectorClock: {},
        );

        when(() => mockRepository.getCurrentVersion('doc-1'))
            .thenAnswer((_) async => currentVersion);

        when(() => mockRepository.restoreVersion('doc-1', 3))
            .thenAnswer((_) async => DocumentVersion(
                  id: 'version-new',
                  documentId: 'doc-1',
                  versionNumber: 6,
                  title: 'Restored from v3',
                  content: {},
                  changeSummary: 'Restored to version 3',
                  createdBy: 'user-1',
                  createdAt: DateTime.now(),
                  vectorClock: {},
                  restoredFromVersion: 3,
                ));

        final result = await versionService.restoreVersion('doc-1', 3);

        // New version number should be current + 1
        expect(result.versionNumber, 6);
        expect(result.restoredFromVersion, 3);
      });
    });

    group('compareVersions', () {
      test('returns diff between two versions', () async {
        final diff = {
          'added': ['New paragraph'],
          'removed': [],
          'modified': [],
        };

        when(() => mockRepository.compareVersions('doc-1', 1, 2))
            .thenAnswer((_) async => diff);

        final result = await versionService.compareVersions('doc-1', 1, 2);

        expect(result['added'], ['New paragraph']);
        expect(result['removed'], isEmpty);
      });

      test('throws exception for invalid version range', () async {
        // Validation happens before repository call
        expect(
          () => versionService.compareVersions('doc-1', 5, 2),
          throwsArgumentError,
        );
      });
    });

    group('getVersionCount', () {
      test('returns total version count for document', () async {
        when(() => mockRepository.getVersionCount('doc-1'))
            .thenAnswer((_) async => 15);

        final result = await versionService.getVersionCount('doc-1');

        expect(result, 15);
      });

      test('returns zero for document with no versions', () async {
        when(() => mockRepository.getVersionCount('doc-empty'))
            .thenAnswer((_) async => 0);

        final result = await versionService.getVersionCount('doc-empty');

        expect(result, 0);
      });
    });
  });
}
