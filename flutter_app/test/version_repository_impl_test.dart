import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/repositories/version_repository_impl.dart';
import 'package:miniwiki/domain/entities/document_version.dart';
import 'package:mocktail/mocktail.dart';

class MockApiClient extends Mock implements ApiClient {}

void main() {
  late VersionRepositoryImpl repository;
  late MockApiClient mockApiClient;

  setUp(() {
    mockApiClient = MockApiClient();
    repository = VersionRepositoryImpl(mockApiClient);
  });

  group('VersionRepositoryImpl', () {
    const testDocumentId = 'test-doc-id';

    final testVersionData = {
      'id': 'version-1',
      'document_id': testDocumentId,
      'version_number': 1,
      'title': 'Test Document v1',
      'content': <String, dynamic>{'delta': []},
      'created_at': '2024-01-01T00:00:00.000Z',
      'created_by': 'user-1',
      'vector_clock': <String, dynamic>{'counter': 1, 'node_id': 'node-1'},
      'change_summary': 'Initial version',
      'content_size': 0,
    };

    final testVersionsListData = {
      'data': {
        'versions': [
          // API returns versions in descending order (latest first)
          {
            ...testVersionData,
            'id': 'version-2',
            'version_number': 2,
            'title': 'Test Document v2',
          } as Map<String, dynamic>,
          testVersionData,
        ],
      },
    };

    group('listVersions', () {
      test('should return list of versions', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: testVersionsListData,
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.get(
          any(),
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.listVersions(testDocumentId);

        // Assert
        expect(result.length, 2);
        expect(result[0].versionNumber, 2); // Latest version (first in descending order)
        expect(result[1].versionNumber, 1);

        verify(() => mockApiClient.get(
          '/documents/$testDocumentId/versions',
          queryParams: {
            'limit': 20,
            'offset': 0,
          },
        )).called(1);
      });

      test('should use custom pagination parameters', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: testVersionsListData,
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.get(
          any(),
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        await repository.listVersions(
          testDocumentId,
          limit: 50,
          offset: 10,
        );

        // Assert
        verify(() => mockApiClient.get(
          '/documents/$testDocumentId/versions',
          queryParams: {
            'limit': 50,
            'offset': 10,
          },
        )).called(1);
      });

      test('should throw exception on failure', () async {
        // Arrange
        when(() => mockApiClient.get(
          any(),
          queryParams: any(named: 'queryParams'),
        )).thenThrow(Exception('Network error'));

        // Act & Assert
        await expectLater(
          repository.listVersions(testDocumentId),
          throwsA(isA<Exception>()),
        );
      });
    });

    group('getVersion', () {
      const testVersionNumber = 1;

      test('should return specific version', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: {'data': testVersionData},
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.get(any()))
            .thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.getVersion(
          testDocumentId,
          testVersionNumber,
        );

        // Assert
        expect(result.versionNumber, testVersionNumber);
        expect(result.title, 'Test Document v1');

        verify(() => mockApiClient.get(
          '/documents/$testDocumentId/versions/$testVersionNumber',
        )).called(1);
      });

      test('should throw exception on failure', () async {
        // Arrange
        when(() => mockApiClient.get(any()))
            .thenThrow(Exception('Version not found'));

        // Act & Assert
        await expectLater(
          repository.getVersion(testDocumentId, 999),
          throwsA(isA<Exception>()),
        );
      });
    });

    group('createVersion', () {
      test('should create new version', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: {'data': testVersionData},
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.post(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => mockResponse);

        const content = <String, dynamic>{'delta': []};
        const vectorClock = <String, dynamic>{'counter': 1, 'node_id': 'node-1'};

        // Act
        final result = await repository.createVersion(
          documentId: testDocumentId,
          title: 'Test Document v1',
          content: content,
          changeSummary: 'Initial version',
          vectorClock: vectorClock,
        );

        // Assert
        expect(result.versionNumber, 1);

        verify(() => mockApiClient.post(
          '/documents/$testDocumentId/versions',
          data: {
            'title': 'Test Document v1',
            'content': content,
            'change_summary': 'Initial version',
            'vector_clock': vectorClock,
          },
        )).called(1);
      });

      test('should create version without optional parameters', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: {'data': testVersionData},
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.post(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => mockResponse);

        const content = <String, dynamic>{'delta': []};

        // Act
        final result = await repository.createVersion(
          documentId: testDocumentId,
          title: 'Test Document v1',
          content: content,
        );

        // Assert
        expect(result.versionNumber, 1);

        verify(() => mockApiClient.post(
          '/documents/$testDocumentId/versions',
          data: {
            'title': 'Test Document v1',
            'content': content,
            'change_summary': null,
            'vector_clock': null,
          },
        )).called(1);
      });
    });

    group('restoreVersion', () {
      const testVersionNumber = 1;

      test('should restore to specific version', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: {'data': testVersionData},
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.post(any()))
            .thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.restoreVersion(
          testDocumentId,
          testVersionNumber,
        );

        // Assert
        expect(result.versionNumber, testVersionNumber);

        verify(() => mockApiClient.post(
          '/documents/$testDocumentId/versions/$testVersionNumber/restore',
        )).called(1);
      });

      test('should throw exception on restore failure', () async {
        // Arrange
        when(() => mockApiClient.post(any()))
            .thenThrow(Exception('Restore failed'));

        // Act & Assert
        await expectLater(
          repository.restoreVersion(testDocumentId, 999),
          throwsA(isA<Exception>()),
        );
      });
    });

    group('compareVersions', () {
      const testFromVersion = 1;
      const testToVersion = 2;

      test('should return diff between versions', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: {
            'data': {
              'added': ['line 1', 'line 2'],
              'removed': ['line 3'],
              'modified': ['line 4'],
            },
          },
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.get(
          any(),
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.compareVersions(
          testDocumentId,
          testFromVersion,
          testToVersion,
        );

        // Assert
        expect(result['added'], ['line 1', 'line 2']);
        expect(result['removed'], ['line 3']);
        expect(result['modified'], ['line 4']);

        verify(() => mockApiClient.get(
          '/documents/$testDocumentId/versions/diff',
          queryParams: {
            'from': testFromVersion,
            'to': testToVersion,
          },
        )).called(1);
      });

      test('should handle empty diff', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: {'data': <String, dynamic>{}},
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.get(
          any(),
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.compareVersions(
          testDocumentId,
          testFromVersion,
          testToVersion,
        );

        // Assert
        expect(result['added'], []);
        expect(result['removed'], []);
        expect(result['modified'], []);
      });
    });

    group('getVersionCount', () {
      test('should return number of versions', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: testVersionsListData,
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.get(
          any(),
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.getVersionCount(testDocumentId);

        // Assert
        expect(result, 2);

        verify(() => mockApiClient.get(
          '/documents/$testDocumentId/versions',
          queryParams: {
            'limit': 1000,
            'offset': 0,
          },
        )).called(1);
      });

      test('should return 0 for document with no versions', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: {'data': {'versions': <dynamic>[]}},
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.get(
          any(),
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.getVersionCount(testDocumentId);

        // Assert
        expect(result, 0);
      });
    });

    group('getCurrentVersion', () {
      test('should return the latest version', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: testVersionsListData,
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.get(
          any(),
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.getCurrentVersion(testDocumentId);

        // Assert
        expect(result.versionNumber, 2); // Latest version (first in descending order)
      });

      test('should throw exception when no versions exist', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          data: {'data': {'versions': <dynamic>[]}},
          requestOptions: RequestOptions(path: ''),
        );
        when(() => mockApiClient.get(
          any(),
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => mockResponse);

        // Act & Assert
        await expectLater(
          repository.getCurrentVersion(testDocumentId),
          throwsA(isA<Exception>()),
        );
      });
    });
  });
}
