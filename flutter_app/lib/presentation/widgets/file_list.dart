import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/file.dart';
import 'package:miniwiki/presentation/providers/file_provider.dart';

/// Widget for displaying a list of files
class FileListWidget extends ConsumerWidget {
  final String spaceId;
  final String? documentId;
  final bool showActions;
  final VoidCallback? onFileTap;
  final VoidCallback? onFileDelete;

  const FileListWidget({
    super.key,
    required this.spaceId,
    this.documentId,
    this.showActions = true,
    this.onFileTap,
    this.onFileDelete,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final fileState = ref.watch(
      fileListNotifierProvider(spaceId),
    );
    final theme = Theme.of(context);

    if (fileState.isLoading && fileState.files.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }

    if (fileState.error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.error_outline,
              size: 48,
              color: theme.colorScheme.error,
            ),
            const SizedBox(height: 16),
            Text(
              'Failed to load files',
              style: theme.textTheme.titleMedium,
            ),
            Text(
              fileState.error!,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: () => ref
                  .read(fileListNotifierProvider(spaceId).notifier)
                  .loadFiles(),
              icon: const Icon(Icons.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    if (fileState.files.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.folder_open,
              size: 48,
              color: theme.colorScheme.onSurfaceVariant.withOpacity(0.5),
            ),
            const SizedBox(height: 16),
            Text(
              'No files attached',
              style: theme.textTheme.titleMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'Files you attach will appear here',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant.withOpacity(0.7),
              ),
            ),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: () =>
          ref.read(fileListNotifierProvider(spaceId).notifier).loadFiles(),
      child: ListView.builder(
        padding: const EdgeInsets.all(8),
        itemCount: fileState.files.length,
        itemBuilder: (context, index) {
          final file = fileState.files[index];
          return _FileListTile(
            file: file,
            showActions: showActions,
            onTap: onFileTap,
            onDelete: onFileDelete,
          );
        },
      ),
    );
  }
}

/// Individual file list tile
class _FileListTile extends StatelessWidget {
  final FileEntity file;
  final bool showActions;
  final VoidCallback? onTap;
  final VoidCallback? onDelete;

  const _FileListTile({
    required this.file,
    this.showActions = true,
    this.onTap,
    this.onDelete,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      child: ListTile(
        leading: Container(
          width: 40,
          height: 40,
          decoration: BoxDecoration(
            color: theme.colorScheme.secondaryContainer,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Center(
            child: Text(
              file.icon,
              style: const TextStyle(fontSize: 20),
            ),
          ),
        ),
        title: Text(
          file.fileName,
          style: theme.textTheme.bodyLarge,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        subtitle: Text(
          file.formattedSize,
          style: theme.textTheme.bodySmall?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
        trailing: showActions
            ? Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  IconButton(
                    icon: const Icon(Icons.download, size: 20),
                    onPressed: onTap,
                    tooltip: 'Download',
                  ),
                  IconButton(
                    icon: const Icon(Icons.delete, size: 20),
                    onPressed: onDelete,
                    tooltip: 'Delete',
                  ),
                ],
              )
            : null,
        onTap: onTap,
      ),
    );
  }
}

/// Compact file list for use in panels
class CompactFileList extends ConsumerWidget {
  final String spaceId;
  final String? documentId;
  final int maxItems;

  const CompactFileList({
    super.key,
    required this.spaceId,
    this.documentId,
    this.maxItems = 3,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final fileState = ref.watch(
      fileListNotifierProvider(spaceId),
    );
    final theme = Theme.of(context);

    final files = fileState.files.take(maxItems).toList();

    if (files.isEmpty) {
      return const SizedBox.shrink();
    }

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Text(
            'Attachments (${fileState.files.length})',
            style: theme.textTheme.labelLarge,
          ),
        ),
        ...files.map((file) => _CompactFileItem(file: file)),
        if (fileState.files.length > maxItems)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
            child: Text(
              '+${fileState.files.length - maxItems} more',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.primary,
              ),
            ),
          ),
      ],
    );
  }
}

class _CompactFileItem extends StatelessWidget {
  final FileEntity file;

  const _CompactFileItem({required this.file});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      child: ListTile(
        dense: true,
        leading: Text(file.icon, style: const TextStyle(fontSize: 16)),
        title: Text(
          file.fileName,
          style: theme.textTheme.bodyMedium,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        subtitle: Text(
          file.formattedSize,
          style: theme.textTheme.bodySmall?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
        trailing: const Icon(Icons.chevron_right, size: 16),
        onTap: () {},
      ),
    );
  }
}
