import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/document.dart';
import 'package:miniwiki/presentation/pages/documents/document_editor_page.dart';
import 'package:miniwiki/presentation/providers/document_provider.dart';

class DocumentListPage extends ConsumerWidget {
  final String spaceId;
  final String spaceName;
  final String? parentId;

  const DocumentListPage({
    required this.spaceId, required this.spaceName, super.key,
    this.parentId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(documentListProvider(spaceId));

    return Scaffold(
      appBar: AppBar(
        title: Text(spaceName),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () {
              ref.read(documentListProvider(spaceId).notifier).refresh();
            },
          ),
        ],
      ),
      body: _buildBody(context, ref, state),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _showCreateDocumentDialog(context, ref),
        label: const Text('New Document'),
        icon: const Icon(Icons.add),
      ),
    );
  }

  Widget _buildBody(BuildContext context, WidgetRef ref, DocumentListState state) {
    if (state.isLoading && state.documents.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }

    if (state.error != null && state.documents.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text('Error: ${state.error}'),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () {
                ref.read(documentListProvider(spaceId).notifier).loadDocuments();
              },
              child: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    if (state.documents.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.description_outlined, size: 64, color: Colors.grey),
            const SizedBox(height: 16),
            Text(
              parentId == null
                  ? 'No documents yet'
                  : 'No documents in this folder',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    color: Colors.grey,
                  ),
            ),
            const SizedBox(height: 8),
            const Text('Create your first document to get started'),
            const SizedBox(height: 24),
            FilledButton.icon(
              onPressed: () => _showCreateDocumentDialog(context, ref),
              icon: const Icon(Icons.add),
              label: const Text('Create Document'),
            ),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: () => ref.read(documentListProvider(spaceId).notifier).refresh(),
      child: ListView.builder(
        padding: const EdgeInsets.all(16),
        itemCount: state.documents.length + (state.hasMore ? 1 : 0),
        itemBuilder: (context, index) {
          if (index >= state.documents.length) {
            return Center(
              child: TextButton(
                onPressed: () {
                  ref
                      .read(documentListProvider(spaceId).notifier)
                      .loadMore(state.documents.length, 20);
                },
                child: const Text('Load More'),
              ),
            );
          }

          final document = state.documents[index];
          return _DocumentListTile(
            document: document,
            onTap: () => _openDocument(context, document),
            onDelete: () => _confirmDelete(context, ref, document),
          );
        },
      ),
    );
  }

  void _openDocument(BuildContext context, Document document) {
    Navigator.push(
      context,
      MaterialPageRoute<void>(
        builder: (BuildContext context) => DocumentEditorPage(
          documentId: document.id,
          spaceId: document.spaceId,
        ),
      ),
    );
  }

  void _showCreateDocumentDialog(BuildContext context, WidgetRef ref) {
    final titleController = TextEditingController();

    showDialog(
      context: context,
      builder: (BuildContext context) => AlertDialog(
        title: const Text('Create Document'),
        content: TextField(
          controller: titleController,
          autofocus: true,
          decoration: const InputDecoration(
            labelText: 'Document Title',
            hintText: 'Enter a title for your document',
          ),
          maxLength: 200,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () async {
              if (titleController.text.trim().isEmpty) {
                return;
              }

              Navigator.pop(context);

              final service = ref.read(documentServiceProvider);
              final document = await service.createDocument(
                spaceId: spaceId,
                title: titleController.text.trim(),
                parentId: parentId,
              );

              if (context.mounted) {
                Navigator.push(
                  context,
                  MaterialPageRoute<void>(
                    builder: (BuildContext context) => DocumentEditorPage(
                      documentId: document.id,
                      spaceId: spaceId,
                    ),
                  ),
                );
              }
            },
            child: const Text('Create'),
          ),
        ],
      ),
    );
  }

  void _confirmDelete(BuildContext context, WidgetRef ref, Document document) {
    showDialog(
      context: context,
      builder: (BuildContext context) => AlertDialog(
        title: const Text('Delete Document'),
        content: Text('Are you sure you want to delete "${document.title}"?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          FilledButton(
            style: FilledButton.styleFrom(
              backgroundColor: Theme.of(context).colorScheme.error,
            ),
            onPressed: () async {
              Navigator.pop(context);

              final service = ref.read(documentServiceProvider);
              await service.deleteDocument(document.id);

              if (context.mounted) {
                ref.read(documentListProvider(spaceId).notifier).refresh();
              }
            },
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('spaceId', spaceId));
    properties.add(StringProperty('spaceName', spaceName));
    properties.add(StringProperty('parentId', parentId));
  }
}

class _DocumentListTile extends StatelessWidget {
  final Document document;
  final VoidCallback onTap;
  final VoidCallback onDelete;

  const _DocumentListTile({
    required this.document,
    required this.onTap,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context) => Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: ListTile(
        leading: Container(
          width: 40,
          height: 40,
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.primaryContainer,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Center(
            child: Text(
              document.icon ?? '📄',
              style: const TextStyle(fontSize: 20),
            ),
          ),
        ),
        title: Text(
          document.title,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        subtitle: Text(
          _formatDate(document.updatedAt),
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: Colors.grey,
              ),
        ),
        trailing: PopupMenuButton<String>(
          onSelected: (value) {
            if (value == 'delete') {
              onDelete();
            }
          },
          itemBuilder: (context) => [
            const PopupMenuItem(
              value: 'delete',
              child: Text('Delete'),
            ),
          ],
        ),
        onTap: onTap,
      ),
    );

  String _formatDate(DateTime? date) {
    if (date == null) return '';

    final now = DateTime.now();
    final difference = now.difference(date);

    if (difference.inMinutes < 1) return 'Just now';
    if (difference.inMinutes < 60) return '${difference.inMinutes} min ago';
    if (difference.inHours < 24) return '${difference.inHours} hours ago';
    if (difference.inDays < 7) return '${difference.inDays} days ago';

    return '${date.month}/${date.day}/${date.year}';
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<Document>('document', document));
    properties.add(ObjectFlagProperty<VoidCallback>.has('onTap', onTap));
    properties.add(ObjectFlagProperty<VoidCallback>.has('onDelete', onDelete));
  }
}
