import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/core/config/providers.dart';
import 'package:miniwiki/presentation/providers/comment_provider.dart';
import 'package:miniwiki/services/providers.dart';

/// Widget for inputting a new comment
class CommentInput extends ConsumerStatefulWidget {
  final String documentId;
  final String? parentCommentId;
  final VoidCallback? onSubmitted;
  final VoidCallback? onCancelled;

  const CommentInput({
    required this.documentId, super.key,
    this.parentCommentId,
    this.onSubmitted,
    this.onCancelled,
  });

  @override
  ConsumerState<CommentInput> createState() => _CommentInputState();

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('documentId', documentId));
    properties.add(StringProperty('parentCommentId', parentCommentId));
    properties.add(ObjectFlagProperty<VoidCallback?>.has('onSubmitted', onSubmitted));
    properties.add(ObjectFlagProperty<VoidCallback?>.has('onCancelled', onCancelled));
  }
}

class _CommentInputState extends ConsumerState<CommentInput> {
  final TextEditingController _controller = TextEditingController();
  final FocusNode _focusNode = FocusNode();
  bool _isExpanded = false;

  @override
  void dispose() {
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final editState = ref.watch(commentEditNotifierProvider);
    final theme = Theme.of(context);

    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      curve: Curves.easeInOut,
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        borderRadius: BorderRadius.circular(12),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.1),
            blurRadius: 8,
            offset: const Offset(0, -2),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Collapsed state - show button to expand
          if (!_isExpanded)
            Padding(
              padding: const EdgeInsets.all(12),
              child: InkWell(
                onTap: () {
                  setState(() => _isExpanded = true);
                  Future.delayed(const Duration(milliseconds: 100), () {
                    if (mounted) {
                      _focusNode.requestFocus();
                    }
                  });
                },
                borderRadius: BorderRadius.circular(8),
                child: Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 12,
                  ),
                  decoration: BoxDecoration(
                    border: Border.all(
                      color: theme.colorScheme.outline.withValues(alpha: 0.5),
                    ),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Row(
                    children: [
                      Icon(
                        Icons.add_comment_outlined,
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                      const SizedBox(width: 12),
                      Text(
                        widget.parentCommentId != null
                            ? 'Write a reply...'
                            : 'Add a comment...',
                        style: TextStyle(
                          color: theme.colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            )

          // Expanded state - show input field
          else
            Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Reply to indicator
                  if (widget.parentCommentId != null)
                    Container(
                      margin: const EdgeInsets.only(bottom: 8),
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 4,
                      ),
                      decoration: BoxDecoration(
                        color: theme.colorScheme.secondaryContainer,
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(
                            Icons.reply,
                            size: 14,
                            color: theme.colorScheme.onSecondaryContainer,
                          ),
                          const SizedBox(width: 4),
                          Text(
                            'Replying to comment',
                            style: TextStyle(
                              color: theme.colorScheme.onSecondaryContainer,
                              fontSize: 12,
                            ),
                          ),
                          const SizedBox(width: 8),
                          GestureDetector(
                            onTap: widget.onCancelled,
                            child: Icon(
                              Icons.close,
                              size: 14,
                              color: theme.colorScheme.onSecondaryContainer,
                            ),
                          ),
                        ],
                      ),
                    ),

                  // Text field
                  TextField(
                    controller: _controller,
                    focusNode: _focusNode,
                    maxLines: 4,
                    maxLength: 5000,
                    decoration: InputDecoration(
                      hintText: 'Write your comment...',
                      hintStyle: TextStyle(
                        color:
                            theme.colorScheme.onSurfaceVariant.withValues(alpha: 0.7),
                      ),
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(8),
                        borderSide: BorderSide(
                          color: theme.colorScheme.outline,
                        ),
                      ),
                      enabledBorder: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(8),
                        borderSide: BorderSide(
                          color: theme.colorScheme.outline.withValues(alpha: 0.5),
                        ),
                      ),
                      focusedBorder: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(8),
                        borderSide: BorderSide(
                          color: theme.colorScheme.primary,
                          width: 2,
                        ),
                      ),
                      contentPadding: const EdgeInsets.all(12),
                      counterStyle: TextStyle(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                    onChanged: (String value) {
                      ref
                          .read(commentEditNotifierProvider.notifier)
                          .setContent(value);
                    },
                  ),

                  // Error message
                  if (editState.error != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(
                        editState.error!,
                        style: TextStyle(
                          color: theme.colorScheme.error,
                          fontSize: 12,
                        ),
                      ),
                    ),

                  const SizedBox(height: 12),

                  // Actions
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      TextButton.icon(
                        onPressed: _clearAndCollapse,
                        icon: const Icon(Icons.close, size: 18),
                        label: const Text('Cancel'),
                      ),
                      const SizedBox(width: 8),
                      ElevatedButton.icon(
                        onPressed: editState.isValid && !editState.isSubmitting
                            ? _submitComment
                            : null,
                        icon: editState.isSubmitting
                            ? const SizedBox(
                                width: 18,
                                height: 18,
                                child:
                                    CircularProgressIndicator(strokeWidth: 2),
                              )
                            : const Icon(Icons.send, size: 18),
                        label: const Text('Post Comment'),
                      ),
                    ],
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }

  void _clearAndCollapse() {
    _controller.clear();
    ref.read(commentEditNotifierProvider.notifier).clear();
    setState(() => _isExpanded = false);
    widget.onCancelled?.call();
  }

  Future<void> _submitComment() async {
    final editState = ref.read(commentEditNotifierProvider);

    if (!editState.isValid) return;

    ref
        .read(commentEditNotifierProvider.notifier)
        .setContent(editState.content);

    try {
      final service = ref.read(commentServiceProvider);
      final comment = await service.createComment(
        documentId: widget.documentId,
        content: editState.content,
        parentId: widget.parentCommentId,
      );

      // Add comment to the list
      ref
          .read(commentListNotifierProvider(widget.documentId).notifier)
          .addComment(comment);

      _clearAndCollapse();
      widget.onSubmitted?.call();
    } catch (e) {
      ref
          .read(commentEditNotifierProvider.notifier)
          .setContent(editState.content);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to post comment: ${e.toString()}'),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    }
  }
}

/// Floating action button to open comment input
class CommentFab extends StatelessWidget {
  final VoidCallback onPressed;

  const CommentFab({required this.onPressed, super.key});

  @override
  Widget build(BuildContext context) => FloatingActionButton.small(
      onPressed: onPressed,
      child: const Icon(Icons.comment_outlined),
    );

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(ObjectFlagProperty<VoidCallback>.has('onPressed', onPressed));
  }
}
