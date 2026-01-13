import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:dio/dio.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/repositories/document_repository_impl.dart';
import 'package:miniwiki/data/datasources/isar_datasource.dart';
import 'package:miniwiki/data/models/document_entity.dart';

class MockApiClient extends Mock implements ApiClient {}

class MockIsarDatabase extends Mock implements IsarDatabase {}

class MockResponse extends Mock implements Response {}

class DocumentEntityFake extends Fake implements DocumentEntity {}

void main() {
  setUpAll(() {
    registerFallbackValue(DocumentEntityFake());
    registerFallbackValue('');
  });

  group('DocumentRepository Tests', () {
    late ApiClient apiClient;
    late IsarDatabase isarDatabase;
    late DocumentRepositoryImpl documentRepository;

    setUp(() {
      apiClient = MockApiClient();
      isarDatabase = MockIsarDatabase();
      documentRepository = DocumentRepositoryImpl(apiClient, isarDatabase);

      when(() => isarDatabase.saveDocument(any())).thenAnswer((_) async => {});
      when(() => isarDatabase.getDocumentById(any())).thenAnswer((_) async => null);
      when(() => isarDatabase.getDocumentsBySpace(any())).thenAnswer((_) async => []);
      when(() => isarDatabase.getDocumentsByParent(any())).thenAnswer((_) async => []);
      when(() => isarDatabase.deleteDocument(any())).thenAnswer((_) async => {});
    });

    group('listDocuments', () {
      test('listDocuments returns list from API', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'data': {
            'documents': [
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
            ],
            'total': 1,
          },
        });
        when(() => apiClient.get('/spaces/space-uuid/documents', queryParams: any(named: 'queryParams')))
            .thenAnswer((_) async => response);

        final result = await documentRepository.listDocuments(spaceId: 'space-uuid');

        expect(result.documents.length, 1);
        expect(result.documents.first.id, 'doc-1');
        expect(result.documents.first.title, 'Document 1');
      });

      test('listDocuments falls back to Isar on error', () async {
        when(() => apiClient.get('/spaces/space-uuid/documents', queryParams: any(named: 'queryParams')))
            .thenThrow(Exception('Network error'));

        final result = await documentRepository.listDocuments(spaceId: 'space-uuid');

        expect(result.documents, isEmpty);
      });
    });

    group('getDocument', () {
      test('getDocument returns document from API', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'data': {
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
          },
        });
        when(() => apiClient.get('/documents/doc-uuid'))
            .thenAnswer((_) async => response);

        final result = await documentRepository.getDocument('doc-uuid');

        expect(result.id, 'doc-uuid');
        expect(result.title, 'Test Document');
      });

      test('getDocument throws when not found', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(404);
        when(() => response.data).thenReturn({
          'error': 'DOC_NOT_FOUND',
          'message': 'Document not found',
        });
        when(() => apiClient.get('/documents/nonexistent'))
            .thenAnswer((_) async => response);

        expect(
          () => documentRepository.getDocument('nonexistent'),
          throwsA(isA<Exception>()),
        );
      });
    });

    group('createDocument', () {
      test('createDocument creates and returns document', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(201);
        when(() => response.data).thenReturn({
          'data': {
            'document': {
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
            },
          },
        });
        when(() => apiClient.post('/spaces/space-uuid/documents', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        final result = await documentRepository.createDocument(
          spaceId: 'space-uuid',
          parentId: null,
          title: 'New Document',
          icon: '📝',
          content: {'type': 'Y.Doc'},
        );

        expect(result.id, 'new-doc-uuid');
        expect(result.title, 'New Document');
      });
    });

    group('updateDocument', () {
      test('updateDocument updates and returns document', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'data': {
            'document': {
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
            },
          },
        });
        when(() => apiClient.patch('/documents/doc-uuid', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        final result = await documentRepository.updateDocument(
          id: 'doc-uuid',
          title: 'Updated Title',
        );

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

    group('getDocumentChildren', () {
      test('getDocumentChildren returns child documents', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'data': {
            'documents': [
              {
                'id': 'child-doc',
                'space_id': 'space-uuid',
                'parent_id': 'parent-doc',
                'title': 'Child Document',
                'icon': '📄',
                'content': {'type': 'Y.Doc'},
                'content_size': 50,
                'is_archived': false,
                'created_by': 'user-uuid',
                'last_edited_by': 'user-uuid',
                'created_at': '2026-01-12T10:00:00Z',
                'updated_at': '2026-01-12T10:00:00Z',
              },
            ],
            'total': 1,
          },
        });
        when(() => apiClient.get('/documents/parent-doc/children'))
            .thenAnswer((_) async => response);

        final result = await documentRepository.getDocumentChildren('parent-doc');

        expect(result.documents.length, 1);
        expect(result.documents.first.id, 'child-doc');
      });
    });

    group('getDocumentPath', () {
      test('getDocumentPath returns path from root to document', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'data': {
            'path': [
              {
                'id': 'root-doc',
                'space_id': 'space-uuid',
                'title': 'Root',
                'content': {},
                'created_by': 'user',
                'last_edited_by': 'user',
              },
              {
                'id': 'child-doc',
                'space_id': 'space-uuid',
                'parent_id': 'root-doc',
                'title': 'Child',
                'content': {},
                'created_by': 'user',
                'last_edited_by': 'user',
              },
            ],
          },
        });
        when(() => apiClient.get(any(), queryParams: any(named: 'queryParams')))
            .thenAnswer((invocation) {
          final path = invocation.positionalArguments[0] as String;
          if (path == '/documents/child-doc/path') {
            return Future.value(response);
          }
          throw Exception('Unexpected path: $path');
        });

        final result = await documentRepository.getDocumentPath('child-doc');

        expect(result.length, 2);
        expect(result.first.id, 'root-doc');
        expect(result.last.id, 'child-doc');
      });

      test('getDocumentPath returns empty list when document not found', () async {
        when(() => apiClient.get(any(), queryParams: any(named: 'queryParams')))
            .thenThrow(Exception('Not found'));

        final result = await documentRepository.getDocumentPath('unknown');

        expect(result, isEmpty);
      });
    });
  });
}
