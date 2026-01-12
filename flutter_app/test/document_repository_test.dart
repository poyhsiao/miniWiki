import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:dio/dio.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/repositories/document_repository_impl.dart';
import 'package:miniwiki/data/models/document_entity.dart';
import 'package:miniwiki/data/datasources/isar_datasource.dart';

class MockApiClient extends Mock implements ApiClient {}

class MockIsarDatabase extends Mock implements IsarDatabase {}

class MockResponse extends Mock implements Response {}

void main() {
  group('DocumentRepository Tests', () {
    late ApiClient apiClient;
    late IsarDatabase isarDatabase;
    late DocumentRepository documentRepository;

    setUp(() {
      apiClient = MockApiClient();
      isarDatabase = MockIsarDatabase();
      documentRepository = DocumentRepositoryImpl(apiClient, isarDatabase);
    });

    group('getDocuments', () {
      test('getDocuments returns list from API', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn([
          {
            'id': 'doc-1',
            'space_id': 'space-uuid',
            'parent_id': null,
            'title': 'Document 1',
            'icon': '📄',
            'content': {'type': 'Y.Doc'},
            'content_size': 100,
            'is_archived': false,
            'created_by': 'user-uuid',
            'last_edited_by': 'user-uuid',
            'created_at': '2026-01-12T10:00:00Z',
            'updated_at': '2026-01-12T10:00:00Z',
          },
        ]);
        when(() => apiClient.get('/spaces/space-uuid/documents'))
            .thenAnswer((_) async => response);

        final result = await documentRepository.getDocuments('space-uuid');

        expect(result.length, 1);
        expect(result.first.uuid, 'doc-1');
        expect(result.first.title, 'Document 1');
      });

      test('getDocuments falls back to Isar on error', () async {
        when(() => apiClient.get('/spaces/space-uuid/documents'))
            .thenThrow(Exception('Network error'));
        when(() => isarDatabase.getDocumentsBySpace('space-uuid'))
            .thenAnswer((_) async => []);

        final result = await documentRepository.getDocuments('space-uuid');

        expect(result, isEmpty);
      });
    });

    group('getDocument', () {
      test('getDocument returns document from API', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'id': 'doc-uuid',
          'space_id': 'space-uuid',
          'parent_id': null,
          'title': 'Test Document',
          'icon': '📄',
          'content': {'type': 'Y.Doc'},
          'content_size': 100,
          'is_archived': false,
          'created_by': 'user-uuid',
          'last_edited_by': 'user-uuid',
          'created_at': '2026-01-12T10:00:00Z',
          'updated_at': '2026-01-12T10:00:00Z',
        });
        when(() => apiClient.get('/documents/doc-uuid'))
            .thenAnswer((_) async => response);

        final result = await documentRepository.getDocument('doc-uuid');

        expect(result!.uuid, 'doc-uuid');
        expect(result.title, 'Test Document');
      });

      test('getDocument returns null when not found', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(404);
        when(() => response.data).thenReturn({
          'error': 'DOC_NOT_FOUND',
          'message': 'Document not found',
        });
        when(() => apiClient.get('/documents/nonexistent'))
            .thenAnswer((_) async => response);

        final result = await documentRepository.getDocument('nonexistent');

        expect(result, isNull);
      });
    });

    group('createDocument', () {
      test('createDocument creates and returns document', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(201);
        when(() => response.data).thenReturn({
          'id': 'new-doc-uuid',
          'space_id': 'space-uuid',
          'parent_id': null,
          'title': 'New Document',
          'icon': '📝',
          'content': {'type': 'Y.Doc'},
          'content_size': 50,
          'is_archived': false,
          'created_by': 'user-uuid',
          'last_edited_by': 'user-uuid',
          'created_at': '2026-01-12T10:00:00Z',
          'updated_at': '2026-01-12T10:00:00Z',
        });
        when(() => apiClient.post('/spaces/space-uuid/documents', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        final newDoc = DocumentEntity()
          ..spaceId = 'space-uuid'
          ..title = 'New Document'
          ..icon = '📝'
          ..content = {'type': 'Y.Doc'};

        final result = await documentRepository.createDocument(newDoc);

        expect(result.uuid, 'new-doc-uuid');
        expect(result.title, 'New Document');
      });
    });

    group('updateDocument', () {
      test('updateDocument updates and returns document', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'id': 'doc-uuid',
          'space_id': 'space-uuid',
          'parent_id': null,
          'title': 'Updated Title',
          'icon': '📄',
          'content': {'type': 'Y.Doc'},
          'content_size': 100,
          'is_archived': false,
          'created_by': 'user-uuid',
          'last_edited_by': 'user-uuid',
          'created_at': '2026-01-12T10:00:00Z',
          'updated_at': '2026-01-12T11:00:00Z',
        });
        when(() => apiClient.patch('/documents/doc-uuid', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        final updateDoc = DocumentEntity()
          ..uuid = 'doc-uuid'
          ..spaceId = 'space-uuid'
          ..title = 'Updated Title';

        final result = await documentRepository.updateDocument(updateDoc);

        expect(result.title, 'Updated Title');
      });
    });

    group('deleteDocument', () {
      test('deleteDocument calls API', () async {
        when(() => apiClient.delete('/documents/doc-uuid'))
            .thenAnswer((_) async => MockResponse()..statusCode = 200);

        await expectLater(
          documentRepository.deleteDocument('doc-uuid'),
          completes,
        );
      });
    });

    group('getDirtyDocuments', () {
      test('getDirtyDocuments returns dirty documents from Isar', () async {
        final dirtyDoc = DocumentEntity()..uuid = 'dirty-doc';
        when(() => isarDatabase.getDirtyDocuments())
            .thenAnswer((_) async => [dirtyDoc]);

        final result = await documentRepository.getDirtyDocuments();

        expect(result.length, 1);
        expect(result.first.uuid, 'dirty-doc');
      });
    });

    group('markDocumentSynced', () {
      test('markDocumentSynced calls Isar', () async {
        when(() => isarDatabase.markDocumentSynced('doc-uuid'))
            .thenAnswer((_) async => {});

        await expectLater(
          documentRepository.markDocumentSynced('doc-uuid'),
          completes,
        );
      });
    });
  });
}
