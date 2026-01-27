import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/document.dart';
import 'package:miniwiki/domain/repositories/document_repository.dart';
import 'package:miniwiki/presentation/providers/document_provider.dart';
import 'package:miniwiki/services/document_service.dart';
import 'package:mocktail/mocktail.dart';

class MockDocumentService extends Mock implements DocumentService {}

void main() {
  group('DocumentListState', () {
    test('creates with default values', () {
      const state = DocumentListState(spaceId: 'space-1');

      expect(state.spaceId, 'space-1');
      expect(state.documents, isEmpty);
      expect(state.total, 0);
      expect(state.isLoading, false);
      expect(state.error, isNull);
      expect(state.parentId, isNull);
    });

    test('creates with custom values', () {
      final documents = [
        Document(
          id: 'doc-1',
          spaceId: 'space-1',
          title: 'Test Doc',
          icon: '📄',
          content: const {},
          createdBy: 'user-1',
          lastEditedBy: 'user-1',
        ),
      ];

      final state = DocumentListState(
        spaceId: 'space-1',
        documents: documents,
        total: 10,
        isLoading: true,
        error: 'Test error',
        parentId: 'parent-1',
      );

      expect(state.documents, documents);
      expect(state.total, 10);
      expect(state.isLoading, true);
      expect(state.error, 'Test error');
      expect(state.parentId, 'parent-1');
    });

    test('copyWith updates specified fields', () {
      const state = DocumentListState(spaceId: 'space-1');
      final documents = [
        Document(
          id: 'doc-1',
          spaceId: 'space-1',
          title: 'Test Doc',
          icon: '📄',
          content: const {},
          createdBy: 'user-1',
          lastEditedBy: 'user-1',
        ),
      ];

      final updated = state.copyWith(
        documents: documents,
        total: 5,
        isLoading: true,
      );

      expect(updated.documents, documents);
      expect(updated.total, 5);
      expect(updated.isLoading, true);
      expect(updated.spaceId, 'space-1'); // unchanged
    });

    test('hasMore returns true when more documents exist', () {
      final state = DocumentListState(
        spaceId: 'space-1',
        documents: List.generate(5, (_) => Document(
          id: 'doc',
          spaceId: 'space-1',
          title: 'Doc',
          icon: '📄',
          content: const {},
          createdBy: 'user',
          lastEditedBy: 'user',
        )),
        total: 10,
      );

      expect(state.hasMore, true);
    });

    test('hasMore returns false when all documents loaded', () {
      final state = DocumentListState(
        spaceId: 'space-1',
        documents: List.generate(10, (_) => Document(
          id: 'doc',
          spaceId: 'space-1',
          title: 'Doc',
          icon: '📄',
          content: const {},
          createdBy: 'user',
          lastEditedBy: 'user',
        )),
        total: 10,
      );

      expect(state.hasMore, false);
    });
  });

  group('DocumentEditState', () {
    test('creates with default values', () {
      const state = DocumentEditState();

      expect(state.document, isNull);
      expect(state.content, isEmpty);
      expect(state.isLoading, false);
      expect(state.isSaving, false);
      expect(state.hasUnsavedChanges, false);
      expect(state.error, isNull);
      expect(state.versions, isEmpty);
      expect(state.selectedVersion, isNull);
    });

    test('creates with custom values', () {
      final document = Document(
        id: 'doc-1',
        spaceId: 'space-1',
        title: 'Test Doc',
        icon: '📄',
        content: const {'type': 'Y.Doc'},
        createdBy: 'user-1',
        lastEditedBy: 'user-1',
      );

      final state = DocumentEditState(
        document: document,
        content: const {'type': 'Y.Doc'},
        isLoading: true,
        isSaving: true,
        hasUnsavedChanges: true,
        error: 'Test error',
        selectedVersion: 2,
      );

      expect(state.document, document);
      expect(state.content, {'type': 'Y.Doc'});
      expect(state.isLoading, true);
      expect(state.isSaving, true);
      expect(state.hasUnsavedChanges, true);
      expect(state.error, 'Test error');
      expect(state.selectedVersion, 2);
    });

    test('copyWith updates specified fields', () {
      const state = DocumentEditState();
      final document = Document(
        id: 'doc-1',
        spaceId: 'space-1',
        title: 'Test Doc',
        icon: '📄',
        content: const {},
        createdBy: 'user-1',
        lastEditedBy: 'user-1',
      );

      final updated = state.copyWith(
        document: document,
        isLoading: true,
        hasUnsavedChanges: true,
      );

      expect(updated.document, document);
      expect(updated.isLoading, true);
      expect(updated.hasUnsavedChanges, true);
      expect(updated.isSaving, false); // unchanged
    });

    test('copyWith can clear error', () {
      const state = DocumentEditState(error: 'Error');

      final updated = state.copyWith(error: null);

      expect(updated.error, isNull);
    });
  });

  group('DocumentListNotifier', () {
    late MockDocumentService mockService;
    late DocumentListNotifier notifier;

    setUp(() {
      mockService = MockDocumentService();
      notifier = DocumentListNotifier(mockService, 'space-1');
    });

    test('loadDocuments updates state with loaded documents', () async {
      final documents = [
        Document(
          id: 'doc-1',
          spaceId: 'space-1',
          title: 'Doc 1',
          icon: '📄',
          content: const {},
          createdBy: 'user-1',
          lastEditedBy: 'user-1',
        ),
      ];

      when(() => mockService.listDocuments(
        spaceId: any(named: 'spaceId'),
        parentId: any(named: 'parentId'),
        limit: any(named: 'limit'),
        offset: any(named: 'offset'),
      )).thenAnswer((_) async => DocumentListResult(
        documents: documents,
        total: 1,
        limit: 20,
        offset: 0,
      ));

      await notifier.loadDocuments();

      expect(notifier.state.documents, documents);
      expect(notifier.state.total, 1);
      expect(notifier.state.isLoading, false);
      expect(notifier.state.error, isNull);
    });

    test('loadDocuments handles errors', () async {
      when(() => mockService.listDocuments(
        spaceId: any(named: 'spaceId'),
        parentId: any(named: 'parentId'),
        limit: any(named: 'limit'),
        offset: any(named: 'offset'),
      )).thenThrow(Exception('Network error'));

      await notifier.loadDocuments();

      expect(notifier.state.isLoading, false);
      expect(notifier.state.error, 'Exception: Network error');
    });

    test('loadMore appends documents to existing list', () async {
      final initialDocs = [
        Document(
          id: 'doc-1',
          spaceId: 'space-1',
          title: 'Doc 1',
          icon: '📄',
          content: const {},
          createdBy: 'user-1',
          lastEditedBy: 'user-1',
        ),
      ];

      final moreDocs = [
        Document(
          id: 'doc-2',
          spaceId: 'space-1',
          title: 'Doc 2',
          icon: '📄',
          content: const {},
          createdBy: 'user-1',
          lastEditedBy: 'user-1',
        ),
      ];

      when(() => mockService.listDocuments(
        spaceId: any(named: 'spaceId'),
        parentId: any(named: 'parentId'),
        limit: any(named: 'limit'),
        offset: any(named: 'offset'),
      )).thenAnswer((_) async => DocumentListResult(
        documents: moreDocs,
        total: 2,
        limit: 20,
        offset: 1,
      ));

      // Load initial documents
      notifier.state = notifier.state.copyWith(documents: initialDocs);

      await notifier.loadMore(1, 20);

      expect(notifier.state.documents.length, 2);
      expect(notifier.state.documents[0].id, 'doc-1');
      expect(notifier.state.documents[1].id, 'doc-2');
      expect(notifier.state.total, 2);
    });

    test('refresh reloads documents', () async {
      final documents = [
        Document(
          id: 'doc-1',
          spaceId: 'space-1',
          title: 'Doc 1',
          icon: '📄',
          content: const {},
          createdBy: 'user-1',
          lastEditedBy: 'user-1',
        ),
      ];

      when(() => mockService.listDocuments(
        spaceId: any(named: 'spaceId'),
        parentId: any(named: 'parentId'),
        limit: any(named: 'limit'),
        offset: any(named: 'offset'),
      )).thenAnswer((_) async => DocumentListResult(
        documents: documents,
        total: 1,
        limit: 20,
        offset: 0,
      ));

      notifier.state = notifier.state.copyWith(parentId: 'parent-1');

      await notifier.refresh();

      expect(notifier.state.documents, documents);
    });
  });

  group('DocumentEditNotifier', () {
    late MockDocumentService mockService;
    late DocumentEditNotifier notifier;

    setUp(() {
      mockService = MockDocumentService();
      notifier = DocumentEditNotifier(mockService);
    });

    test('loadDocument updates state with loaded document', () async {
      final document = Document(
        id: 'doc-1',
        spaceId: 'space-1',
        title: 'Test Doc',
        icon: '📄',
        content: const {'type': 'Y.Doc'},
        createdBy: 'user-1',
        lastEditedBy: 'user-1',
      );

      when(() => mockService.getDocument('doc-1'))
          .thenAnswer((_) async => document);

      final result = await notifier.loadDocument('doc-1');

      expect(result, document);
      expect(notifier.state.document, document);
      expect(notifier.state.content, {'type': 'Y.Doc'});
      expect(notifier.state.isLoading, false);
      expect(notifier.state.hasUnsavedChanges, false);
    });

    test('loadDocument handles errors', () async {
      when(() => mockService.getDocument('doc-1'))
          .thenThrow(Exception('Not found'));

      expect(
        () => notifier.loadDocument('doc-1'),
        throwsException,
      );

      expect(notifier.state.isLoading, false);
      expect(notifier.state.error, isNotNull);
    });

    test('updateContent updates state and marks as unsaved', () {
      const newContent = {'type': 'Y.Doc', 'delta': []};

      notifier.updateContent(newContent);

      expect(notifier.state.content, newContent);
      expect(notifier.state.hasUnsavedChanges, true);
    });

    test('saveDocument updates document and clears unsaved flag', () async {
      final document = Document(
        id: 'doc-1',
        spaceId: 'space-1',
        title: 'Test Doc',
        icon: '📄',
        content: const {},
        createdBy: 'user-1',
        lastEditedBy: 'user-1',
      );

      final updated = Document(
        id: 'doc-1',
        spaceId: 'space-1',
        title: 'Test Doc',
        icon: '📄',
        content: const {'saved': true},
        createdBy: 'user-1',
        lastEditedBy: 'user-1',
      );

      notifier.state = notifier.state.copyWith(
        document: document,
        content: const {'saved': true},
      );

      when(() => mockService.updateDocument(
        id: any(named: 'id'),
        title: any(named: 'title'),
        icon: any(named: 'icon'),
        content: any(named: 'content'),
      )).thenAnswer((_) async => updated);

      final result = await notifier.saveDocument();

      expect(result, updated);
      expect(notifier.state.document, updated);
      expect(notifier.state.isSaving, false);
      expect(notifier.state.hasUnsavedChanges, false);
    });

    test('saveDocument throws when no document loaded', () {
      expect(
        () => notifier.saveDocument(),
        throwsException,
      );
    });

    test('updateTitle updates document title', () async {
      final document = Document(
        id: 'doc-1',
        spaceId: 'space-1',
        title: 'Old Title',
        icon: '📄',
        content: const {},
        createdBy: 'user-1',
        lastEditedBy: 'user-1',
      );

      final updated = Document(
        id: 'doc-1',
        spaceId: 'space-1',
        title: 'New Title',
        icon: '📄',
        content: const {},
        createdBy: 'user-1',
        lastEditedBy: 'user-1',
      );

      notifier.state = notifier.state.copyWith(document: document);

      when(() => mockService.updateTitle('doc-1', 'New Title'))
          .thenAnswer((_) async => updated);

      final result = await notifier.updateTitle('New Title');

      expect(result, updated);
      expect(notifier.state.document, updated);
      expect(notifier.state.hasUnsavedChanges, true);
    });

    test('deleteDocument clears state', () async {
      final document = Document(
        id: 'doc-1',
        spaceId: 'space-1',
        title: 'Test',
        icon: '📄',
        content: const {},
        createdBy: 'user-1',
        lastEditedBy: 'user-1',
      );

      notifier.state = notifier.state.copyWith(document: document);

      when(() => mockService.deleteDocument('doc-1'))
          .thenAnswer((_) async {});

      await notifier.deleteDocument();

      expect(notifier.state.document, isNull);
    });

    test('clearError clears error state', () {
      notifier.state = notifier.state.copyWith(error: 'Error');
      notifier.clearError();

      expect(notifier.state.error, isNull);
    });

    test('reset clears all state', () {
      final document = Document(
        id: 'doc-1',
        spaceId: 'space-1',
        title: 'Test',
        icon: '📄',
        content: const {},
        createdBy: 'user-1',
        lastEditedBy: 'user-1',
      );

      notifier.state = notifier.state.copyWith(
        document: document,
        content: const {'test': true},
        hasUnsavedChanges: true,
      );

      notifier.reset();

      expect(notifier.state.document, isNull);
      expect(notifier.state.content, isEmpty);
      expect(notifier.state.hasUnsavedChanges, false);
    });
  });
}
