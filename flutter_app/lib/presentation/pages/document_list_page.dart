import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/document.dart';
import 'package:miniwiki/presentation/providers/document_provider.dart';
import 'package:miniwiki/presentation/pages/document_editor_page.dart';
import 'package:intl/intl.dart';

class DocumentListPage extends ConsumerStatefulWidget {
  final String spaceId;

  const DocumentListPage({
    required this.spaceId, super.key,
  });

  @override
  ConsumerState<DocumentListPage> createState() => _DocumentListPageState();

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('spaceId', spaceId));
  }
}

class _DocumentListPageState extends ConsumerState<DocumentListPage> {
  final TextEditingController _searchController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  String? _parentId;

  @override
  void initState() {
    super.initState();
    Future.microtask(
      () => ref.read(documentListProvider(widget.spaceId).notifier).loadDocuments(),
    );
    _scrollController.addListener(_onScroll);
  }

  void _onScroll() {
    if (_scrollController.position.pixels >=
        _scrollController.position.maxScrollExtent - 200) {
      final state = ref.read(documentListProvider(widget.spaceId));
      if (!state.isLoading && state.hasMore) {
        ref.read(documentListProvider(widget.spaceId).notifier).loadMore(
          state.documents.length,
          20,
        );
      }
    }
  }

  @override
  void dispose() {
    _searchController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final documentState = ref.watch(documentListProvider(widget.spaceId));

    return Scaffold(
      appBar: AppBar(
        title: const Text('Documents'),
        actions: [
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () => _showSearchBottomSheet(context),
          ),
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: () => _createNewDocument(context),
          ),
        ],
      ),
      body: Column(
        children: [
          if (_parentId != null)
            _buildBreadcrumb(context),
          Expanded(
            child: _buildDocumentList(context, documentState),
          ),
        ],
      ),
    );
  }

  Widget _buildBreadcrumb(BuildContext context) => Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: Theme.of(context).colorScheme.surfaceVariant,
      child: Row(
        children: [
          const Icon(Icons.folder, size: 16),
          const SizedBox(width: 8),
          Text(
            'Current folder',
            style: Theme.of(context).textTheme.bodyMedium,
          ),
          const Spacer(),
          TextButton(
            onPressed: () {
              setState(() => _parentId = null);
              ref.read(documentListProvider(widget.spaceId).notifier).loadDocuments();
            },
            child: const Text('Root'),
          ),
        ],
      ),
    );

  Widget _buildDocumentList(BuildContext context, DocumentListState state) {
    if (state.isLoading && state.documents.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }

    if (state.error != null && state.documents.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(state.error!, style: const TextStyle(color: Colors.red)),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () => ref
                  .read(documentListProvider(widget.spaceId).notifier)
                  .loadDocuments(),
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
            const Text('No documents yet'),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: () => _createNewDocument(context),
              icon: const Icon(Icons.add),
              label: const Text('Create Document'),
            ),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: () =>
          ref.read(documentListProvider(widget.spaceId).notifier).refresh(),
      child: ListView.builder(
        controller: _scrollController,
        padding: const EdgeInsets.all(8),
        itemCount: state.documents.length + (state.hasMore ? 1 : 0),
        itemBuilder: (context, index) {
          if (index >= state.documents.length) {
            return const Center(child: Padding(
              padding: EdgeInsets.all(16),
              child: CircularProgressIndicator(),
            ));
          }

          final document = state.documents[index];
          return _DocumentTile(
            document: document,
            onTap: () => _openDocument(context, document),
            onDelete: () => _deleteDocument(context, document),
          );
        },
      ),
    );
  }

  void _showSearchBottomSheet(BuildContext context) {
    showModalBottomSheet(
      context: context,
      builder: (context) => Padding(
        padding: EdgeInsets.only(
          bottom: MediaQuery.of(context).viewInsets.bottom,
        ),
        child: Container(
          padding: const EdgeInsets.all(16),
          child: TextField(
            controller: _searchController,
            autofocus: true,
            decoration: InputDecoration(
              hintText: 'Search documents...',
              prefixIcon: const Icon(Icons.search),
              suffixIcon: IconButton(
                icon: const Icon(Icons.clear),
                onPressed: _searchController.clear,
              ),
            ),
            onSubmitted: (query) {
              Navigator.pop(context);
              // Implement search
            },
          ),
        ),
      ),
    );
  }

  Future<void> _createNewDocument(BuildContext context) async {
    final result = await Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => DocumentEditorPage(
          spaceId: widget.spaceId,
          parentId: _parentId,
        ),
      ),
    );

    if (result == true) {
      ref.read(documentListProvider(widget.spaceId).notifier).refresh();
    }
  }

  Future<void> _openDocument(BuildContext context, Document document) async {
    final result = await Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => DocumentEditorPage(
          spaceId: widget.spaceId,
          documentId: document.id,
        ),
      ),
    );

    if (result == true) {
      ref.read(documentListProvider(widget.spaceId).notifier).refresh();
    }
  }

  Future<void> _deleteDocument(BuildContext context, Document document) async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete Document'),
        content: Text('Are you sure you want to delete "${document.title}"?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Delete', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );

    if (confirm ?? false) {
      try {
        await ref.read(documentEditProvider.notifier).loadDocument(document.id);
        await ref.read(documentEditProvider.notifier).deleteDocument();
        ref.read(documentListProvider(widget.spaceId).notifier).refresh();
      } catch (e) {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('Failed to delete: $e')),
          );
        }
      }
    }
  }
}

class _DocumentTile extends StatelessWidget {
  final Document document;
  final VoidCallback onTap;
  final VoidCallback onDelete;

  const _DocumentTile({
    required this.document,
    required this.onTap,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context) {
    final dateFormat = DateFormat('MMM d, yyyy');

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
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
          style: const TextStyle(fontWeight: FontWeight.w500),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        subtitle: Text(
          'Modified ${dateFormat.format(document.updatedAt ?? DateTime.now())}',
          style: Theme.of(context).textTheme.bodySmall,
        ),
        trailing: PopupMenuButton<String>(
          onSelected: (value) {
            if (value == 'delete') onDelete();
          },
          itemBuilder: (context) => [
            const PopupMenuItem(value: 'delete', child: Text('Delete')),
          ],
        ),
        onTap: onTap,
      ),
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<Document>('document', document));
    properties.add(ObjectFlagProperty<VoidCallback>.has('onTap', onTap));
    properties.add(ObjectFlagProperty<VoidCallback>.has('onDelete', onDelete));
  }
}
