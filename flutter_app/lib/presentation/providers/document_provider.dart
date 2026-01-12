import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/domain/entities/document.dart';
import 'package:miniwiki/domain/repositories/document_repository.dart';
import 'package:miniwiki/services/document_service.dart';
import 'package:miniwiki/services/sync_service.dart';
import 'package:miniwiki/core/config/providers.dart';

/// State for the document list
class DocumentListState {
  final List<Document> documents;
  final int total;
  final bool isLoading;
  final String? error;
  final String? parentId;
  final String spaceId;

  const DocumentListState({
    this.documents = const [],
    this.total = 0,
    this.isLoading = false,
    this.error,
    this.parentId,
    required this.spaceId,
  });

  DocumentListState copyWith({
    List<Document>? documents,
    int? total,
    bool? isLoading,
    String? error,
    String? parentId,
    String? spaceId,
  }) {
    return DocumentListState(
      documents: documents ?? this.documents,
      total: total ?? this.total,
      isLoading: isLoading ?? this.isLoading,
      error: error ?? this.error,
      parentId: parentId ?? this.parentId,
      spaceId: spaceId ?? this.spaceId,
    );
  }

  bool get hasMore => documents.length < total;
}

/// State for the current document being edited
class DocumentEditState {
  final Document? document;
  final Map<String, dynamic> content;
  final bool isLoading;
  final bool isSaving;
  final bool hasUnsavedChanges;
  final String? error;
  final List<DocumentVersion> versions;
  final int? selectedVersion;

  const DocumentEditState({
    this.document,
    this.content = const {},
    this.isLoading = false,
    this.isSaving = false,
    this.hasUnsavedChanges = false,
    this.error,
    this.versions = const [],
    this.selectedVersion,
  });

  DocumentEditState copyWith({
    Document? document,
    Map<String, dynamic>? content,
    bool? isLoading,
    bool? isSaving,
    bool? hasUnsavedChanges,
    Object? error,
    List<DocumentVersion>? versions,
    int? selectedVersion,
  }) {
    return DocumentEditState(
      document: document ?? this.document,
      content: content ?? this.content,
      isLoading: isLoading ?? this.isLoading,
      isSaving: isSaving ?? this.isSaving,
      hasUnsavedChanges: hasUnsavedChanges ?? this.hasUnsavedChanges,
      error: error == null ? null : (error is String ? error as String : this.error),
      versions: versions ?? this.versions,
      selectedVersion: selectedVersion ?? this.selectedVersion,
    );
  }
}

/// Provider for document list state
class DocumentListNotifier extends StateNotifier<DocumentListState> {
  final DocumentService _service;
  final String spaceId;

  DocumentListNotifier(this._service, this.spaceId)
      : super(DocumentListState(spaceId: spaceId));

  Future<void> loadDocuments({String? parentId, int limit = 20}) async {
    state = state.copyWith(isLoading: true, error: null, parentId: parentId);

    try {
      final result = await _service.listDocuments(
        spaceId: spaceId,
        parentId: parentId,
        limit: limit,
      );

      state = state.copyWith(
        documents: result.documents,
        total: result.total,
        isLoading: false,
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.toString(),
      );
    }
  }

  Future<void> loadMore(int offset, int limit) async {
    if (state.isLoading) return;

    try {
      final result = await _service.listDocuments(
        spaceId: spaceId,
        parentId: state.parentId,
        limit: limit,
        offset: offset,
      );

      state = state.copyWith(
        documents: [...state.documents, ...result.documents],
        total: result.total,
      );
    } catch (e) {
      state = state.copyWith(error: e.toString());
    }
  }

  Future<void> refresh() async {
    await loadDocuments(parentId: state.parentId);
  }
}

/// Provider for document list
final documentListProvider =
    StateNotifierProvider.family<DocumentListNotifier, DocumentListState, String>(
  (ref, spaceId) {
    final service = ref.watch(documentServiceProvider);
    return DocumentListNotifier(service, spaceId);
  },
);

/// Provider for document service
final documentServiceProvider = Provider<DocumentService>((ref) {
  final repository = ref.watch(documentRepositoryProvider);
  final syncService = ref.watch(syncServiceProvider);
  return DocumentService(repository, syncService);
});

/// Notifier for document editing state
class DocumentEditNotifier extends StateNotifier<DocumentEditState> {
  final DocumentService _service;

  DocumentEditNotifier(this._service) : super(const DocumentEditState());

  Future<Document> loadDocument(String documentId) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final document = await _service.getDocument(documentId);
      state = state.copyWith(
        document: document,
        content: document.content,
        isLoading: false,
        hasUnsavedChanges: false,
      );
      return document;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.toString(),
      );
      rethrow;
    }
  }

  void updateContent(Map<String, dynamic> newContent) {
    state = state.copyWith(
      content: newContent,
      hasUnsavedChanges: true,
    );
  }

  Future<Document> saveDocument() async {
    final document = state.document;
    if (document == null) {
      throw Exception('No document to save');
    }

    state = state.copyWith(isSaving: true, error: null);

    try {
      final updated = await _service.updateDocument(
        id: document.id,
        title: document.title,
        icon: document.icon,
        content: state.content,
      );

      state = state.copyWith(
        document: updated,
        isSaving: false,
        hasUnsavedChanges: false,
      );

      return updated;
    } catch (e) {
      state = state.copyWith(
        isSaving: false,
        error: e.toString(),
      );
      rethrow;
    }
  }

  Future<Document> updateTitle(String title) async {
    final document = state.document;
    if (document == null) {
      throw Exception('No document to update');
    }

    try {
      final updated = await _service.updateTitle(document.id, title);
      state = state.copyWith(
        document: updated,
        hasUnsavedChanges: true,
      );
      return updated;
    } catch (e) {
      state = state.copyWith(error: e.toString());
      rethrow;
    }
  }

  Future<void> deleteDocument() async {
    final document = state.document;
    if (document == null) {
      throw Exception('No document to delete');
    }

    try {
      await _service.deleteDocument(document.id);
      state = const DocumentEditState();
    } catch (e) {
      state = state.copyWith(error: e.toString());
      rethrow;
    }
  }

  Future<List<DocumentVersion>> loadVersions() async {
    final document = state.document;
    if (document == null) {
      throw Exception('No document loaded');
    }

    try {
      final result = await _service.getVersions(document.id);
      state = state.copyWith(versions: result.versions);
      return result.versions;
    } catch (e) {
      state = state.copyWith(error: e.toString());
      rethrow;
    }
  }

  Future<Document> restoreVersion(int versionNumber) async {
    final document = state.document;
    if (document == null) {
      throw Exception('No document to restore');
    }

    state = state.copyWith(isSaving: true, error: null);

    try {
      final restored = await _service.restoreVersion(document.id, versionNumber);
      state = state.copyWith(
        document: restored,
        content: restored.content,
        isSaving: false,
        hasUnsavedChanges: true,
      );
      return restored;
    } catch (e) {
      state = state.copyWith(
        isSaving: false,
        error: e.toString(),
      );
      rethrow;
    }
  }

  void selectVersion(int? versionNumber) {
    state = state.copyWith(selectedVersion: versionNumber);
  }

  void clearError() {
    state = state.copyWith(error: null);
  }

  void reset() {
    state = const DocumentEditState();
  }
}

/// Provider for document editing state
final documentEditProvider =
    StateNotifierProvider<DocumentEditNotifier, DocumentEditState>((ref) {
  final service = ref.watch(documentServiceProvider);
  return DocumentEditNotifier(service);
});
