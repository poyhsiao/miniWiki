import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:miniwiki/domain/entities/comment.dart';

/// Widget to display a list of comments for a document
class CommentList extends StatelessWidget {
  final List<Comment> comments;
  final String currentUserId;
  final Function(Comment) onReply;
  final Function(Comment) onResolve;
  final Function(Comment) onUnresolve;
  final Function(Comment) onDelete;
  final bool isLoading;

  const CommentList({
    super.key,
    required this.comments,
    required this.currentUserId,
    required this.onReply,
    required this.onResolve,
    required this.onUnresolve,
    required this.onDelete,
    this.isLoading = false,
  });

  @override
  Widget build(BuildContext context) {
    if (isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (comments.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(Icons.chat_bubble_outline,
                  size: 48, color: Colors.grey[400]),
              const SizedBox(height: 16),
              Text(
                'No comments yet',
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      color: Colors.grey[600],
                    ),
              ),
              const SizedBox(height: 8),
              Text(
                'Be the first to add a comment!',
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: Colors.grey[500],
                    ),
              ),
            ],
          ),
        ),
      );
    }

    // Group comments by thread (parent -> replies)
    final topLevelComments = comments.where((c) => c.parentId == null).toList();

    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: topLevelComments.length,
      itemBuilder: (context, index) {
        final comment = topLevelComments[index];
        final replies = comments.where((c) => c.parentId == comment.id).toList()
          ..sort((a, b) => (a.createdAt ?? DateTime(0))
              .compareTo(b.createdAt ?? DateTime(0)));

        return CommentThread(
          comment: comment,
          replies: replies,
          currentUserId: currentUserId,
          onReply: onReply,
          onResolve: onResolve,
          onUnresolve: onUnresolve,
          onDelete: onDelete,
        );
      },
    );
  }
}

/// A comment thread (parent + replies)
class CommentThread extends StatelessWidget {
  final Comment comment;
  final List<Comment> replies;
  final String currentUserId;
  final Function(Comment) onReply;
  final Function(Comment) onResolve;
  final Function(Comment) onUnresolve;
  final Function(Comment) onDelete;

  const CommentThread({
    super.key,
    required this.comment,
    required this.replies,
    required this.currentUserId,
    required this.onReply,
    required this.onResolve,
    required this.onUnresolve,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final dateFormat = DateFormat('MMM d, yyyy \'at\' h:mm a');

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Parent comment
        Card(
          margin: const EdgeInsets.only(bottom: 8),
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Header: avatar, name, time
                Row(
                  children: [
                    CircleAvatar(
                      radius: 16,
                      backgroundImage: comment.authorAvatar != null
                          ? NetworkImage(comment.authorAvatar!)
                          : null,
                      backgroundColor: theme.colorScheme.primaryContainer,
                      child: comment.authorAvatar == null
                          ? Text(
                              comment.authorName.isNotEmpty
                                  ? comment.authorName[0].toUpperCase()
                                  : '?',
                              style: TextStyle(
                                color: theme.colorScheme.onPrimaryContainer,
                              ),
                            )
                          : null,
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            comment.authorName,
                            style: theme.textTheme.titleSmall,
                          ),
                          Text(
                            comment.createdAt != null
                                ? dateFormat.format(comment.createdAt!)
                                : 'Unknown time',
                            style: theme.textTheme.bodySmall?.copyWith(
                              color: Colors.grey[600],
                            ),
                          ),
                        ],
                      ),
                    ),
                    if (comment.isResolved)
                      Container(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 8,
                          vertical: 4,
                        ),
                        decoration: BoxDecoration(
                          color: Colors.green[100],
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(Icons.check_circle,
                                size: 14, color: Colors.green[700]),
                            const SizedBox(width: 4),
                            Text(
                              'Resolved',
                              style: TextStyle(
                                color: Colors.green[700],
                                fontSize: 12,
                              ),
                            ),
                          ],
                        ),
                      ),
                  ],
                ),
                const SizedBox(height: 12),
                // Content
                Text(
                  comment.content,
                  style: theme.textTheme.bodyMedium,
                ),
                const SizedBox(height: 12),
                // Actions
                _buildActions(context, theme),
              ],
            ),
          ),
        ),

        // Replies (indented)
        if (replies.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(left: 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: replies.map((reply) {
                return _buildReplyCard(reply, context, theme, dateFormat);
              }).toList(),
            ),
          ),

        const SizedBox(height: 16),
      ],
    );
  }

  Widget _buildActions(BuildContext context, ThemeData theme) {
    final isAuthor = comment.authorId == currentUserId;

    return Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        TextButton.icon(
          onPressed: () => onReply(comment),
          icon: const Icon(Icons.reply, size: 18),
          label: const Text('Reply'),
        ),
        if (!comment.isResolved)
          TextButton.icon(
            onPressed: () => onResolve(comment),
            icon: const Icon(Icons.check_circle_outline, size: 18),
            label: const Text('Resolve'),
          ),
        if (comment.isResolved &&
            (isAuthor || comment.resolvedBy == currentUserId))
          TextButton.icon(
            onPressed: () => onUnresolve(comment),
            icon: const Icon(Icons.unpublished, size: 18),
            label: const Text('Unresolve'),
          ),
        if (isAuthor)
          TextButton.icon(
            onPressed: () => onDelete(comment),
            icon: const Icon(Icons.delete_outline, size: 18),
            label: const Text('Delete'),
            style: TextButton.styleFrom(
              foregroundColor: theme.colorScheme.error,
            ),
          ),
      ],
    );
  }

  Widget _buildReplyCard(
    Comment reply,
    BuildContext context,
    ThemeData theme,
    DateFormat dateFormat,
  ) {
    final isAuthor = reply.authorId == currentUserId;

    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                CircleAvatar(
                  radius: 12,
                  backgroundImage: reply.authorAvatar != null
                      ? NetworkImage(reply.authorAvatar!)
                      : null,
                  backgroundColor: theme.colorScheme.secondaryContainer,
                  child: reply.authorAvatar == null
                      ? Text(
                          reply.authorName.isNotEmpty
                              ? reply.authorName[0].toUpperCase()
                              : '?',
                          style: TextStyle(
                            color: theme.colorScheme.onSecondaryContainer,
                            fontSize: 12,
                          ),
                        )
                      : null,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        reply.authorName,
                        style: theme.textTheme.labelMedium,
                      ),
                      Text(
                        reply.createdAt != null
                            ? dateFormat.format(reply.createdAt!)
                            : '',
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: Colors.grey[600],
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              reply.content,
              style: theme.textTheme.bodyMedium,
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                if (isAuthor)
                  TextButton.icon(
                    onPressed: () => onDelete(reply),
                    icon: const Icon(Icons.delete_outline, size: 16),
                    label: const Text('Delete'),
                    style: TextButton.styleFrom(
                      foregroundColor: theme.colorScheme.error,
                    ),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
