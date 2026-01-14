import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/presentation/providers/document_provider.dart';
import 'package:miniwiki/presentation/providers/presence_provider.dart';
import 'package:miniwiki/presentation/widgets/rich_text_editor.dart';
import 'package:miniwiki/presentation/widgets/cursor_overlay.dart';
import 'package:miniwiki/core/config/auth_provider.dart';

class DocumentEditorPage extends ConsumerStatefulWidget {
  final String documentId;
  final String spaceId;

  const DocumentEditorPage({
    required this.documentId, required this.spaceId, super.key,
  });

  @override
  ConsumerState<DocumentEditorPage> createState() => _DocumentEditorPageState();

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('documentId', documentId));
    properties.add(StringProperty('spaceId', spaceId));
  }
}

 class _DocumentEditorPageState extends ConsumerState<DocumentEditorPage> {
   Timer? _autoSaveTimer;
   Timer? _cursorDebounceTimer;
   final Duration _autoSaveInterval = const Duration(seconds: 30);
   bool _isAutoSaving = false;
   DateTime? _lastSavedAt;
   bool _isConnectedToWebSocket = false;

  @override
  void initState() {
    super.initState();
    _startAutoSave();
    _connectToWebSocket();
  }

  @override
  void dispose() {
    _autoSaveTimer?.cancel();
    _cursorDebounceTimer?.cancel();
    _disconnectFromWebSocket();
    super.dispose();
  }

  Future<void> _connectToWebSocket() async {
    final authState = ref.read(authProvider);

    if (authState is! Authenticated) {
      return;
    }

    try {
      final userId = (authState).userId;
      await ref.read(presenceProvider.notifier).connectToDocument(
        widget.documentId,
        userId,
      );
      _isConnectedToWebSocket = true;
    } catch (e) {
      // Log error but don't block the editor
      debugPrint('Failed to connect to WebSocket: $e');
    }
  }

  Future<void> _disconnectFromWebSocket() async {
    if (_isConnectedToWebSocket) {
      await ref.read(presenceProvider.notifier).disconnectFromDocument();
      _isConnectedToWebSocket = false;
    }
  }

  void _startAutoSave() {
    _autoSaveTimer = Timer.periodic(_autoSaveInterval, (_) {
      _autoSave();
    });
  }

  Future<void> _autoSave() async {
    final state = ref.read(documentEditProvider);
    if (!state.hasUnsavedChanges || state.isSaving) return;

    setState(() => _isAutoSaving = true);
    try {
      await ref.read(documentEditProvider.notifier).saveDocument();
      setState(() => _lastSavedAt = DateTime.now());
    } catch (e) {
      _showErrorSnackBar('Auto-save failed: $e');
    } finally {
      setState(() => _isAutoSaving = false);
    }
  }

  void _showErrorSnackBar(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: Theme.of(context).colorScheme.error,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(documentEditProvider);

    return Scaffold(
      appBar: _buildAppBar(context, state),
      body: Stack(
        children: [
          _buildBody(context, state),
          // Active users indicator overlay
          if (state.document != null)
            const Positioned(
              top: 16,
              right: 16,
              child: ActiveUsersIndicator(),
            ),
        ],
      ),
    );
  }

  PreferredSizeWidget _buildAppBar(BuildContext context, DocumentEditState state) => AppBar(
      title: state.document == null
          ? const Text('Loading...')
          : Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  state.document!.title,
                  style: const TextStyle(fontSize: 16),
                ),
                if (_isAutoSaving)
                  Text(
                    'Saving...',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Colors.grey,
                        ),
                  )
                else if (_lastSavedAt != null)
                  Text(
                    'Saved ${_formatTimeAgo(_lastSavedAt!)}',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Colors.grey,
                        ),
                  ),
              ],
            ),
      actions: [
        IconButton(
          icon: const Icon(Icons.history),
          tooltip: 'Version History',
          onPressed: () => _showVersionHistory(context),
        ),
        IconButton(
          icon: const Icon(Icons.more_vert),
          onPressed: () => _showMoreOptions(context),
        ),
      ],
    );

  Widget _buildBody(BuildContext context, DocumentEditState state) {
    if (state.isLoading && state.document == null) {
      return const Center(child: CircularProgressIndicator());
    }

    if (state.error != null && state.document == null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text('Error: ${state.error}'),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () {
                ref.read(documentEditProvider.notifier).loadDocument(widget.documentId);
              },
              child: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    return Column(
      children: [
        if (state.hasUnsavedChanges)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
            color: Theme.of(context).colorScheme.tertiaryContainer,
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    'Unsaved changes',
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.onTertiaryContainer,
                      fontSize: 12,
                    ),
                  ),
                ),
                TextButton(
                  onPressed: _autoSave,
                  child: Text(
                    'Save Now',
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.onTertiaryContainer,
                    ),
                  ),
                ),
              ],
            ),
          ),
        Expanded(
          child: CursorOverlay(
            child: RichTextEditor(
              initialContent: state.content,
              onContentChanged: (newContent) {
                ref.read(documentEditProvider.notifier).updateContent(newContent);
              },
            ),
          ),
        ),
      ],
    );
  }

  void _showVersionHistory(BuildContext context) {
    showModalBottomSheet(
      context: context,
      builder: (context) => _VersionHistorySheet(
        documentId: widget.documentId,
      ),
    );
  }

  void _showMoreOptions(BuildContext context) {
    showModalBottomSheet(
      context: context,
      builder: (context) => _MoreOptionsSheet(
        documentId: widget.documentId,
        spaceId: widget.spaceId,
      ),
    );
  }

  String _formatTimeAgo(DateTime dateTime) {
    final difference = DateTime.now().difference(dateTime);

    if (difference.inSeconds < 60) return 'just now';
    if (difference.inMinutes < 60) return '${difference.inMinutes}m ago';
    if (difference.inHours < 24) return '${difference.inHours}h ago';
    return '${difference.inDays}d ago';
  }
}

class _VersionHistorySheet extends ConsumerWidget {
  final String documentId;

  const _VersionHistorySheet({required this.documentId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(documentEditProvider);
    final versions = state.versions;

    return Container(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'Version History',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
              ),
              IconButton(
                icon: const Icon(Icons.close),
                onPressed: () => Navigator.pop(context),
              ),
            ],
          ),
          const SizedBox(height: 16),
          if (versions.isEmpty)
            Center(
              child: Column(
                children: [
                  const Icon(Icons.history, size: 48, color: Colors.grey),
                  const SizedBox(height: 8),
                  Text(
                    'No versions yet',
                    style: TextStyle(color: Colors.grey[600]),
                  ),
                  const SizedBox(height: 16),
                  FilledButton.icon(
                    onPressed: () async {
                      await ref.read(documentEditProvider.notifier).loadVersions();
                    },
                    icon: const Icon(Icons.refresh),
                    label: const Text('Refresh'),
                  ),
                ],
              ),
            )
        else
          Expanded(
            child: ListView.builder(
              itemCount: versions.length,
              itemBuilder: (context, index) {
                final version = versions[index];
                return ListTile(
                  leading: CircleAvatar(
                    child: Text('${version.versionNumber}'),
                  ),
                  title: Text(version.title),
                  subtitle: Text(
                    '${version.createdAt.month}/${version.createdAt.day}/${version.createdAt.year} - ${version.changeSummary ?? 'No description'}',
                  ),
                  trailing: TextButton(
                    onPressed: () async {
                      Navigator.pop(context);
                      await Future.microtask(
                        () => _showRestoreConfirmDialog(context, ref, version.versionNumber),
                      );
                    },
                    child: const Text('Restore'),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _showRestoreConfirmDialog(
    BuildContext context,
    WidgetRef ref,
    int versionNumber,
  ) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Restore Version'),
        content: Text('Are you sure you want to restore to version $versionNumber?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Restore'),
          ),
        ],
      ),
    ) ?? false;

    if (confirmed && context.mounted) {
      try {
        await ref.read(documentEditProvider.notifier).restoreVersion(versionNumber);
        if (context.mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('Version restored successfully')),
          );
        }
      } catch (e) {
        if (context.mounted) {
          _showErrorSnackBar(context, 'Failed to restore: $e');
        }
      }
    }
  }

  void _showErrorSnackBar(BuildContext context, String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: Theme.of(context).colorScheme.error,
      ),
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('documentId', documentId));
  }
}

class _MoreOptionsSheet extends StatelessWidget {
  final String documentId;
  final String spaceId;

  const _MoreOptionsSheet({
    required this.documentId,
    required this.spaceId,
  });

  @override
  Widget build(BuildContext context) => SafeArea(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          ListTile(
            leading: const Icon(Icons.share),
            title: const Text('Share'),
            onTap: () {
              Navigator.pop(context);
            },
          ),
          ListTile(
            leading: const Icon(Icons.file_download),
            title: const Text('Export'),
            onTap: () {
              Navigator.pop(context);
            },
          ),
          const Divider(),
          ListTile(
            leading: Icon(
              Icons.delete,
              color: Theme.of(context).colorScheme.error,
            ),
            title: Text(
              'Delete',
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
            onTap: () {
              Navigator.pop(context);
            },
          ),
        ],
      ),
    );

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('documentId', documentId));
    properties.add(StringProperty('spaceId', spaceId));
  }
}
