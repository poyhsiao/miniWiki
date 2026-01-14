import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/domain/entities/comment.dart';
import 'package:miniwiki/services/comment_service.dart';
import 'package:miniwiki/core/config/providers.dart';

/// State for the comment list
class CommentListState {
  final List<Comment> comments;
  final int total;
  final bool isLoading;
  final String? error;
  final String? parentId;
  final String documentId;

  const CommentListState({
    required this.documentId,
    this.comments = const [],
    this.total = 0,
    this.isLoading = false,
    this.error,
    this.parentId,
  });

  CommentListState copyWith({
    List<Comment>? comments,
    int? total,
    bool? isLoading,
    String? error,
    String? parentId,
    String? documentId,
  }) =>
      CommentListState(
        documentId: documentId ?? this.documentId,
        comments: comments ?? this.comments,
        total: total ?? this.total,
        isLoading: isLoading ?? this.isLoading,
        error: error ?? this.error,
        parentId: parentId ?? this.parentId,
      );

  bool get hasMore => comments.length < total;

  /// Get top-level comments only
  List<Comment> get topLevelComments =>
      comments.where((c) => c.parentId == null).toList();

  /// Get replies to a specific comment
  List<Comment> getReplies(String commentId) =>
      comments.where((c) => c.parentId == commentId).toList();
}

/// State for creating/editing a comment
class CommentEditState {
  final String content;
  final bool isSubmitting;
  final String? error;

  const CommentEditState({
    this.content = '',
    this.isSubmitting = false,
    this.error,
  });

  CommentEditState copyWith({
    String? content,
    bool? isSubmitting,
    String? error,
  }) =>
      CommentEditState(
        content: content ?? this.content,
        isSubmitting: isSubmitting ?? this.isSubmitting,
        error: error ?? this.error,
      );

  bool get isValid => content.trim().isNotEmpty;
}

/// Provider for comment list state
class CommentListNotifier extends StateNotifier<CommentListState> {
  final CommentService _service;
  final String documentId;

  CommentListNotifier(this._service, this.documentId)
      : super(CommentListState(documentId: documentId));

  Future<void> loadComments({String? parentId, int limit = 50}) async {
    state = state.copyWith(isLoading: true, error: null, parentId: parentId);

    try {
      final result = await _service.listComments(
        documentId: documentId,
        parentId: parentId,
        limit: limit,
      );

      state = state.copyWith(
        comments: result.comments,
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

  Future<void> loadAllComments({int limit = 100}) async {
    await loadComments(limit: limit);
  }

  void addComment(Comment comment) {
    state = state.copyWith(
      comments: [...state.comments, comment],
      total: state.total + 1,
    );
  }

  void updateComment(Comment updatedComment) {
    state = state.copyWith(
      comments: state.comments
          .map((c) => c.id == updatedComment.id ? updatedComment : c)
          .toList(),
    );
  }

  void removeComment(String commentId) {
    final idsToRemove = {commentId};
    idsToRemove.addAll(
      state.comments.where((c) => c.parentId == commentId).map((c) => c.id),
    );
    final remaining =
        state.comments.where((c) => !idsToRemove.contains(c.id)).toList();
    state = state.copyWith(
      comments: remaining,
      total: state.total - idsToRemove.length,
    );
  }

  void clearComments() {
    state = CommentListState(documentId: documentId);
  }
}

/// Provider for comment edit state
class CommentEditNotifier extends StateNotifier<CommentEditState> {
  CommentEditNotifier() : super(const CommentEditState());

  void setContent(String content) {
    state = state.copyWith(content: content, error: null);
  }

  void clear() {
    state = const CommentEditState();
  }
}

/// Provider for comment list notifier
final commentListNotifierProvider =
    StateNotifierProvider.family<CommentListNotifier, CommentListState, String>(
  (ref, documentId) {
    final service = ref.watch(commentServiceProvider);
    return CommentListNotifier(service, documentId);
  },
);

/// Provider for comment edit notifier
final commentEditNotifierProvider =
    StateNotifierProvider<CommentEditNotifier, CommentEditState>(
  (ref) => CommentEditNotifier(),
);
