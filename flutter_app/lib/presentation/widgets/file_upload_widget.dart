import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/file.dart';
import 'package:miniwiki/presentation/providers/file_provider.dart';
import 'package:miniwiki/core/config/providers.dart';
import 'package:miniwiki/services/providers.dart';
import 'package:miniwiki/services/file_service.dart';

/// Widget for uploading files
class FileUploadWidget extends ConsumerWidget {
  final String spaceId;
  final String documentId;
  final bool showProgress;
  final VoidCallback? onUploadComplete;
  final VoidCallback? onDismiss;

  const FileUploadWidget({
    super.key,
    required this.spaceId,
    required this.documentId,
    this.showProgress = true,
    this.onUploadComplete,
    this.onDismiss,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final uploadKey = '$spaceId:$documentId';
    final uploadState = ref.watch(fileUploadNotifierProvider(uploadKey));
    final theme = Theme.of(context);

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (uploadState.isUploading && showProgress)
          _buildOverallProgress(context, uploadState, theme)
        else
          _buildUploadButton(context, ref, theme),
        if (uploadState.uploads.isNotEmpty)
          _buildUploadList(context, uploadState, theme, ref, uploadKey),
      ],
    );
  }

  Widget _buildUploadButton(
    BuildContext context,
    WidgetRef ref,
    ThemeData theme,
  ) {
    return ElevatedButton.icon(
      onPressed: () => _pickAndUpload(context, ref),
      icon: const Icon(Icons.attach_file),
      label: const Text('Attach File'),
      style: ElevatedButton.styleFrom(
        backgroundColor: theme.colorScheme.secondaryContainer,
        foregroundColor: theme.colorScheme.onSecondaryContainer,
      ),
    );
  }

  Widget _buildOverallProgress(
    BuildContext context,
    FileUploadState uploadState,
    ThemeData theme,
  ) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            const Icon(Icons.cloud_upload, size: 20),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                'Uploading ${uploadState.uploads.length} file(s)...',
                style: theme.textTheme.bodyMedium,
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        LinearProgressIndicator(
          value: uploadState.overallProgress,
          minHeight: 4,
          backgroundColor: theme.colorScheme.surfaceVariant,
          valueColor: AlwaysStoppedAnimation<Color>(
            theme.colorScheme.primary,
          ),
        ),
      ],
    );
  }

  Widget _buildUploadList(
    BuildContext context,
    FileUploadState uploadState,
    ThemeData theme,
    WidgetRef ref,
    String uploadKey,
  ) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: uploadState.uploads.entries.map((entry) {
        final progress = entry.value;
        return _buildUploadItem(context, progress, theme, ref, uploadKey);
      }).toList(),
    );
  }

  Widget _buildUploadItem(
    BuildContext context,
    FileUploadProgress progress,
    ThemeData theme,
    WidgetRef ref,
    String uploadKey,
  ) {
    final statusColor = switch (progress.status) {
      FileUploadStatus.pending => theme.colorScheme.outline,
      FileUploadStatus.uploading => theme.colorScheme.primary,
      FileUploadStatus.completed => theme.colorScheme.primary,
      FileUploadStatus.failed => theme.colorScheme.error,
      FileUploadStatus.cancelled => theme.colorScheme.outline,
    };

    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: Row(
        children: [
          Icon(
            _getStatusIcon(progress.status),
            size: 20,
            color: statusColor,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  progress.fileName,
                  style: theme.textTheme.bodyMedium,
                  overflow: TextOverflow.ellipsis,
                ),
                if (progress.status == FileUploadStatus.uploading)
                  LinearProgressIndicator(
                    value: progress.progress,
                    minHeight: 3,
                    backgroundColor: theme.colorScheme.surfaceVariant,
                  ),
                if (progress.error != null)
                  Text(
                    progress.error!,
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.error,
                    ),
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
              ],
            ),
          ),
          if (progress.status == FileUploadStatus.completed ||
              progress.status == FileUploadStatus.failed)
            IconButton(
              icon: const Icon(Icons.close, size: 18),
              onPressed: () {
                ref
                    .read(fileUploadNotifierProvider(uploadKey).notifier)
                    .removeUpload(progress.fileName);
                onDismiss?.call();
              },
            ),
        ],
      ),
    );
  }

  IconData _getStatusIcon(FileUploadStatus status) {
    return switch (status) {
      FileUploadStatus.pending => Icons.hourglass_empty,
      FileUploadStatus.uploading => Icons.cloud_upload,
      FileUploadStatus.completed => Icons.check_circle,
      FileUploadStatus.failed => Icons.error,
      FileUploadStatus.cancelled => Icons.cancel,
    };
  }

  Future<void> _pickAndUpload(BuildContext context, WidgetRef ref) async {
    try {
      final service = ref.read(fileServiceProvider);
      final uploadKey = '$spaceId:$documentId';
      final uploader = ref.read(fileUploadNotifierProvider(uploadKey).notifier);

      final files = await service.pickFiles(
        type: FileTypeFilter.all,
        allowMultiple: true,
      );

      if (files.isEmpty) return;

      await uploader.uploadFiles(files);
      onUploadComplete?.call();
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to pick files: ${e.toString()}'),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    }
  }
}

/// Compact file upload button for use in toolbars
class FileUploadButton extends ConsumerWidget {
  final String spaceId;
  final String documentId;
  final VoidCallback? onFilesUploaded;

  const FileUploadButton({
    super.key,
    required this.spaceId,
    required this.documentId,
    this.onFilesUploaded,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final uploadKey = '$spaceId:$documentId';
    final uploadState = ref.watch(fileUploadNotifierProvider(uploadKey));
    final hasActiveUploads = uploadState.isUploading;

    return IconButton(
      onPressed: hasActiveUploads ? null : () => _pickAndUpload(context, ref),
      icon: hasActiveUploads
          ? const SizedBox(
              width: 20,
              height: 20,
              child: CircularProgressIndicator(strokeWidth: 2),
            )
          : const Icon(Icons.attach_file),
      tooltip: 'Attach File',
    );
  }

  Future<void> _pickAndUpload(BuildContext context, WidgetRef ref) async {
    try {
      final service = ref.read(fileServiceProvider);
      final uploadKey = '$spaceId:$documentId';
      final uploader = ref.read(fileUploadNotifierProvider(uploadKey).notifier);

      final files = await service.pickFiles(
        type: FileTypeFilter.all,
        allowMultiple: true,
      );

      if (files.isEmpty) return;

      await uploader.uploadFiles(files);
      onFilesUploaded?.call();
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to pick files: ${e.toString()}'),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    }
  }
}
