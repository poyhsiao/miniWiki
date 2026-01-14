import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/presentation/dialogs/export_dialog.dart';
import 'package:miniwiki/presentation/providers/document_provider.dart';
import 'package:miniwiki/presentation/providers/export_provider.dart';
import 'package:miniwiki/presentation/widgets/file_list.dart';
import 'package:miniwiki/presentation/widgets/file_upload_widget.dart';
import 'package:miniwiki/services/export_service.dart';
import 'package:miniwiki/presentation/widgets/rich_text_editor.dart';

class DocumentEditorPage extends ConsumerStatefulWidget {
  final String spaceId;
  final String? documentId;
  final String? parentId;

  const DocumentEditorPage({
    required this.spaceId,
    super.key,
    this.documentId,
    this.parentId,
  });

  @override
  ConsumerState<DocumentEditorPage> createState() => _DocumentEditorPageState();

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('spaceId', spaceId));
    properties.add(StringProperty('documentId', documentId));
    properties.add(StringProperty('parentId', parentId));
  }
}

class _DocumentEditorPageState extends ConsumerState<DocumentEditorPage> {
  final TextEditingController _titleController = TextEditingController();
  final FocusNode _titleFocus = FocusNode();
  bool _isInitialized = false;

  @override
  void initState() {
    super.initState();
    if (widget.documentId != null) {
      Future.microtask(_loadDocument);
    }
  }

  @override
  void dispose() {
    _titleController.dispose();
    _titleFocus.dispose();
    super.dispose();
  }

  Future<void> _loadDocument() async {
    try {
      await ref
          .read(documentEditProvider.notifier)
          .loadDocument(widget.documentId!);
      final document = ref.read(documentEditProvider).document;
      if (document != null) {
        _titleController.text = document.title;
      }
      setState(() => _isInitialized = true);
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Failed to load document: $e')),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final editState = ref.watch(documentEditProvider);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () async {
            final canPop = await _maybePop(context);
            if (canPop && mounted) {
              Navigator.pop(context);
            }
          },
        ),
        title: _buildTitleField(context, editState),
        actions: _buildActions(context, editState),
        bottom: editState.isSaving
            ? const PreferredSize(
                preferredSize: Size.fromHeight(2),
                child: LinearProgressIndicator(),
              )
            : null,
      ),
      body: Column(
        children: [
          if (editState.error != null)
            Container(
              color: Colors.red.shade100,
              padding: const EdgeInsets.all(8),
              child: Row(
                children: [
                  const Icon(Icons.error, color: Colors.red),
                  const SizedBox(width: 8),
                  Expanded(child: Text(editState.error!)),
                  TextButton(
                    onPressed: () =>
                        ref.read(documentEditProvider.notifier).clearError(),
                    child: const Text('Dismiss'),
                  ),
                ],
              ),
            ),
          Expanded(
            child: editState.isLoading && !_isInitialized
                ? const Center(child: CircularProgressIndicator())
                : _buildEditorContent(context, editState),
          ),
        ],
      ),
    );
  }

  Widget _buildTitleField(BuildContext context, DocumentEditState state) =>
      TextField(
        controller: _titleController,
        focusNode: _titleFocus,
        decoration: const InputDecoration(
          hintText: 'Untitled',
          border: InputBorder.none,
          filled: false,
        ),
        style: Theme.of(context).textTheme.titleLarge?.copyWith(
              fontWeight: FontWeight.bold,
            ),
        onChanged: (value) {
          if (widget.documentId != null) {
            ref.read(documentEditProvider.notifier).updateTitle(value);
          }
        },
      );

  List<Widget> _buildActions(BuildContext context, DocumentEditState state) => [
        if (widget.documentId != null)
          IconButton(
            icon: const Icon(Icons.attach_file),
            onPressed: () => _showFileAttachments(context),
          ),
        if (widget.documentId != null)
          IconButton(
            icon: const Icon(Icons.history),
            onPressed: () => _showVersionHistory(context),
          ),
        if (widget.documentId != null)
          IconButton(
            icon: const Icon(Icons.file_download),
            onPressed: () => _showExportDialog(context),
          ),
        if (widget.documentId != null)
          IconButton(
            icon: const Icon(Icons.more_vert),
            onPressed: () => _showMoreOptions(context),
          ),
        const SizedBox(width: 8),
        ElevatedButton.icon(
          onPressed: state.isSaving ? null : () => _saveDocument(context),
          icon: state.isSaving
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(Icons.save),
          label: const Text('Save'),
        ),
      ];

  Widget _buildEditorContent(BuildContext context, DocumentEditState state) =>
      Column(
        children: [
          Expanded(
            child: RichTextEditor(
              initialContent: state.content,
              onContentChanged: (content) {
                ref.read(documentEditProvider.notifier).updateContent(content);
              },
            ),
          ),
          if (widget.documentId != null)
            SizedBox(
              height: 120,
              child: FileListWidget(
                spaceId: widget.spaceId,
                documentId: widget.documentId,
                showActions: true,
              ),
            ),
          if (state.hasUnsavedChanges)
            Container(
              padding: const EdgeInsets.all(8),
              color: Theme.of(context).colorScheme.surfaceVariant,
              child: Row(
                children: [
                  const Icon(Icons.info_outline, size: 16),
                  const SizedBox(width: 8),
                  const Text('Unsaved changes'),
                  const Spacer(),
                  TextButton(
                    onPressed: () => _saveDocument(context),
                    child: const Text('Save now'),
                  ),
                ],
              ),
            ),
        ],
      );

  Future<void> _saveDocument(BuildContext context) async {
    try {
      if (widget.documentId == null) {
        await ref.read(documentServiceProvider).createDocument(
              spaceId: widget.spaceId,
              parentId: widget.parentId,
              title: _titleController.text.isEmpty
                  ? 'Untitled'
                  : _titleController.text,
              content: ref.read(documentEditProvider).content,
            );
        if (mounted) {
          Navigator.pop(context, true);
        }
      } else {
        await ref.read(documentEditProvider.notifier).saveDocument();
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('Saved successfully')),
          );
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Failed to save: $e')),
        );
      }
    }
  }

  Future<void> _showVersionHistory(BuildContext context) async {
    try {
      await ref.read(documentEditProvider.notifier).loadVersions();
    } catch (_) {
      // Silently ignore errors when preloading versions for history sheet
    }
    if (!mounted) return;
    showModalBottomSheet(
      context: context,
      builder: (context) =>
          _VersionHistorySheet(documentId: widget.documentId!),
    );
  }

  void _showExportDialog(BuildContext context) {
    final document = ref.read(documentEditProvider).document;
    final documentTitle = document?.title ??
        (_titleController.text.isEmpty ? 'Untitled' : _titleController.text);
    showDialog(
      context: context,
      builder: (context) => ExportDialog(
        documentId: widget.documentId!,
        documentTitle: documentTitle,
        onExportComplete: () {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('Document exported successfully')),
          );
        },
        onShare: () async {
          try {
            final path =
                await ref.read(exportNotifierProvider.notifier).shareExport(
                      documentId: widget.documentId!,
                      format: ExportFormat.markdown,
                    );
            if (path != null && context.mounted) {
              // TODO: Implement native share
            }
          } catch (e) {
            if (context.mounted) {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('Failed to share: $e')),
              );
            }
          }
        },
      ),
    );
  }

  void _showFileAttachments(BuildContext context) {
    showModalBottomSheet(
      context: context,
      builder: (context) => _FileAttachmentsSheet(
        spaceId: widget.spaceId,
        documentId: widget.documentId!,
      ),
    );
  }

  void _showMoreOptions(BuildContext context) {
    showModalBottomSheet(
      context: context,
      builder: (context) => _MoreOptionsSheet(
        documentId: widget.documentId!,
        onDeleted: () => Navigator.popUntil(context, (route) => route.isFirst),
      ),
    );
  }

  Future<bool> _maybePop(BuildContext context) async {
    final state = ref.read(documentEditProvider);
    if (state.hasUnsavedChanges) {
      final shouldDiscard = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('Unsaved Changes'),
          content: const Text('You have unsaved changes. Discard them?'),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Cancel'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context, true),
              child: const Text('Discard', style: TextStyle(color: Colors.red)),
            ),
          ],
        ),
      );
      return shouldDiscard ?? false;
    }
    return true;
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
            children: [
              const Text('Version History',
                  style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
              const Spacer(),
              IconButton(
                icon: const Icon(Icons.close),
                onPressed: () => Navigator.pop(context),
              ),
            ],
          ),
          const SizedBox(height: 8),
          if (state.isLoading)
            const Center(child: CircularProgressIndicator())
          else if (versions.isEmpty)
            const Center(
                child: Padding(
              padding: EdgeInsets.all(32),
              child: Text('No version history available'),
            ))
          else
            Expanded(
              child: ListView.builder(
                itemCount: versions.length,
                itemBuilder: (context, index) {
                  final version = versions[index];
                  return ListTile(
                    leading: const Icon(Icons.history),
                    title: Text('Version ${version.versionNumber}'),
                    subtitle: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        if (version.changeSummary != null)
                          Text(version.changeSummary!),
                        Text(
                          'by ${version.createdBy}',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      ],
                    ),
                    trailing: TextButton(
                      onPressed: () async {
                        Navigator.pop(context);
                        try {
                          await ref
                              .read(documentEditProvider.notifier)
                              .restoreVersion(version.versionNumber);
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                                content: Text(
                                    'Restored to version ${version.versionNumber}')),
                          );
                        } catch (e) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text('Failed to restore: $e')),
                          );
                        }
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

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('documentId', documentId));
  }
}

class _MoreOptionsSheet extends ConsumerWidget {
  final String documentId;
  final VoidCallback onDeleted;

  const _MoreOptionsSheet({required this.documentId, required this.onDeleted});

  @override
  Widget build(BuildContext context, WidgetRef ref) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.share),
              title: const Text('Share'),
              onTap: () {
                Navigator.pop(context);
                // TODO: Implement share
              },
            ),
            ListTile(
              leading: const Icon(Icons.content_copy),
              title: const Text('Duplicate'),
              onTap: () {
                Navigator.pop(context);
                // TODO: Implement duplicate
              },
            ),
            ListTile(
              leading: const Icon(Icons.move_to_inbox),
              title: const Text('Move to folder'),
              onTap: () {
                Navigator.pop(context);
                // TODO: Implement move
              },
            ),
            const Divider(),
            ListTile(
              leading: const Icon(Icons.delete, color: Colors.red),
              title: const Text('Delete', style: TextStyle(color: Colors.red)),
              onTap: () async {
                Navigator.pop(context);
                final confirm = await showDialog<bool>(
                  context: context,
                  builder: (context) => AlertDialog(
                    title: const Text('Delete Document'),
                    content: const Text(
                        'Are you sure you want to delete this document?'),
                    actions: [
                      TextButton(
                        onPressed: () => Navigator.pop(context, false),
                        child: const Text('Cancel'),
                      ),
                      TextButton(
                        onPressed: () => Navigator.pop(context, true),
                        child: const Text('Delete',
                            style: TextStyle(color: Colors.red)),
                      ),
                    ],
                  ),
                );

                if (confirm == true) {
                  try {
                    await ref
                        .read(documentEditProvider.notifier)
                        .deleteDocument();
                    onDeleted();
                  } catch (e) {
                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('Failed to delete: $e')),
                      );
                    }
                  }
                }
              },
            ),
          ],
        ),
      );

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('documentId', documentId));
    properties
        .add(ObjectFlagProperty<VoidCallback>.has('onDeleted', onDeleted));
  }
}

class _FileAttachmentsSheet extends ConsumerWidget {
  final String spaceId;
  final String documentId;

  const _FileAttachmentsSheet(
      {required this.spaceId, required this.documentId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Container(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Text('Attachments',
                  style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
              const Spacer(),
              IconButton(
                icon: const Icon(Icons.close),
                onPressed: () => Navigator.pop(context),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Expanded(
            child: FileListWidget(
              spaceId: spaceId,
              documentId: documentId,
              showActions: true,
            ),
          ),
          const SizedBox(height: 16),
          FileUploadWidget(
            spaceId: spaceId,
            documentId: documentId,
            showProgress: true,
          ),
        ],
      ),
    );
  }
}
