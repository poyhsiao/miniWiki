import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/models/document_entity.dart';
import 'package:miniwiki/data/repositories/document_repository_impl.dart';
import 'package:mocktail/mocktail.dart';

class MockApiClient extends Mock implements ApiClient {}

class MockResponse extends Mock implements Response<Object?> {}

class DocumentEntityFake extends Fake implements DocumentEntity {}

void main() {
  setUpAll(() {
    registerFallbackValue(DocumentEntityFake());
    registerFallbackValue('');
  });

  group('DocumentRepository Tests', () {
    late ApiClient apiClient;
    late DocumentRepositoryImpl documentRepository;

    setUp(() {
      apiClient = MockApiClient();
      documentRepository = DocumentRepositoryImpl(apiClient);
    });

    group('listDocuments', () {
      test('listDocuments returns list from API', () async {
        final response = MockResponse<dynamic>();
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
        when(() => apiClient.get('/spaces/space-uuid/documents',
                queryParams: any(named: 'queryParams')))
            .thenAnswer((_) async => response);

        final result =
            await documentRepository.listDocuments(spaceId: 'space-uuid');

        expect(result.documents.length, 1);
        expect(result.documents.first.id, 'doc-1');
        expect(result.documents.first.title, 'Document 1');
      });

      test('listDocuments throws an exception on API error', () async {
        when(() => apiClient.get('/spaces/space-uuid/documents',
                queryParams: any(named: 'queryParams')))
            .thenThrow(Exception('Network error'));

        expect(
          () => documentRepository.listDocuments(spaceId: 'space-uuid'),
          throwsA(isA<Exception>()),
        );
      });
    });

    group('getDocument', () {
      test('getDocument returns document from API', () async {
        final response = MockResponse<dynamic>();
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
        when(() => apiClient.get('/documents/nonexistent'))
            .thenThrow(Exception('Document not found'));

        expect(
          () => documentRepository.getDocument('nonexistent'),
          throwsA(isA<Exception>()),
        );
      });
    });

    group('createDocument', () {
      test('createDocument creates and returns document', () async {
        final response = MockResponse<dynamic>();
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
        when(() => apiClient.post('/spaces/space-uuid/documents',
            data: any(named: 'data'))).thenAnswer((_) async => response);

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
        final response = MockResponse<dynamic>();
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
        when(() => apiClient.patch('/documents/doc-uuid',
            data: any(named: 'data'))).thenAnswer((_) async => response);

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
            .thenAnswer((_) async => MockResponse<dynamic>()..statusCode = 200);

        await expectLater(
          documentRepository.deleteDocument('doc-uuid'),
          completes,
        );
      });
    });

    group('getDocumentChildren', () {
      test('getDocumentChildren returns child documents', () async {
        final response = MockResponse<dynamic>();
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

        final result =
            await documentRepository.getDocumentChildren('parent-doc');

        expect(result.documents.length, 1);
        expect(result.documents.first.id, 'child-doc');
      });
    });

    group('getDocumentPath', () {
      test('getDocumentPath returns path from root to document', () async {
        final response = MockResponse<dynamic>();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'data': {
            'path': [
              {
                'id': 'root-doc',
                'space_id': 'space-uuid',
                'title': 'Root',
                'content': <String, dynamic>{},
                'created_by': 'user',
                'last_edited_by': 'user',
              },
              {
                'id': 'child-doc',
                'space_id': 'space-uuid',
                'parent_id': 'root-doc',
                'title': 'Child',
                'content': <String, dynamic>{},
                'created_by': 'user',
                'last_edited_by': 'user',
              },
            ],
          },
        });
        when(() => apiClient.get(any(), queryParams: any(named: 'queryParams')))
            .thenAnswer((_) async => response);

        final result = await documentRepository.getDocumentPath('child-doc');

        expect(result.length, 2);
        expect(result.first.id, 'root-doc');
        expect(result.last.id, 'child-doc');
        verify(() => apiClient.get('/documents/child-doc/path')).called(1);
      });

      test('getDocumentPath throws exception when document not found',
          () async {
        when(() => apiClient.get(any(), queryParams: any(named: 'queryParams')))
            .thenThrow(Exception('Not found'));

        expect(
          () => documentRepository.getDocumentPath('unknown'),
          throwsA(isA<Exception>()),
        );
      });
    });
  });
}
