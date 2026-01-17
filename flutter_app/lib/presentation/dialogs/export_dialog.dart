import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/presentation/providers/export_provider.dart';
import 'package:miniwiki/services/export_service.dart';

/// Dialog for exporting a document in various formats
class ExportDialog extends ConsumerStatefulWidget {
  final String documentId;
  final String documentTitle;
  final VoidCallback? onExportComplete;
  final VoidCallback? onShare;

  const ExportDialog({
    required this.documentId,
    required this.documentTitle,
    this.onExportComplete,
    this.onShare,
    super.key,
  });

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('documentId', documentId));
    properties.add(StringProperty('documentTitle', documentTitle));
    properties.add(ObjectFlagProperty<VoidCallback?>.has(
        'onExportComplete', onExportComplete));
    properties.add(ObjectFlagProperty<VoidCallback?>.has('onShare', onShare));
  }

  @override
  ConsumerState<ExportDialog> createState() => _ExportDialogState();
}

class _ExportDialogState extends ConsumerState<ExportDialog> {
  ExportFormat? _selectedFormat;

  @override
  void initState() {
    super.initState();
    _selectedFormat = ref.read(exportNotifierProvider).selectedFormat;
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;
    final exportState = ref.watch(exportNotifierProvider);

    return AlertDialog(
      backgroundColor: colorScheme.surface,
      title: Text(
        'Export Document',
        style: textTheme.titleLarge,
      ),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            'Export "${widget.documentTitle}" as:',
            style: textTheme.bodyMedium,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 20),
          // Error message
          if (exportState.error != null)
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: colorScheme.errorContainer,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  Icon(
                    Icons.error_outline,
                    color: colorScheme.error,
                    size: 20,
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      exportState.error!,
                      style: textTheme.bodySmall?.copyWith(
                        color: colorScheme.error,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          if (exportState.error != null) const SizedBox(height: 16),
          // Export format options
          RadioGroup<ExportFormat>(
            groupValue: _selectedFormat,
            onChanged: exportState.isExporting
                ? null
                : (value) {
                    if (value != null) {
                      setState(() => _selectedFormat = value);
                    }
                  },
            child: Column(
              children: ExportFormat.availableFormats.map(
                (format) => _buildFormatOption(
                  context,
                  format,
                  exportState.isExporting,
                ),
              ).toList(),
            ),
          ),
          if (exportState.isExporting)
            Padding(
              padding: const EdgeInsets.only(top: 16),
              child: Column(
                children: [
                  LinearProgressIndicator(
                    value: exportState.downloadProgress ?? 0.0,
                    color: colorScheme.primary,
                    backgroundColor: colorScheme.surfaceContainerHighest,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Exporting... ${((exportState.downloadProgress ?? 0.0) * 100).toInt()}%',
                    style: textTheme.bodySmall,
                  ),
                ],
              ),
            ),
          // Success message
          if (exportState.lastExport != null && !exportState.isExporting)
            Container(
              margin: const EdgeInsets.only(top: 16),
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: colorScheme.primaryContainer,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  Icon(
                    Icons.check_circle,
                    color: colorScheme.primary,
                    size: 20,
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Exported successfully!',
                      style: textTheme.bodySmall?.copyWith(
                        color: colorScheme.onPrimaryContainer,
                      ),
                    ),
                  ),
                ],
              ),
            ),
        ],
      ),
      actions: [
        // Cancel button
        TextButton(
          onPressed: exportState.isExporting
              ? null
              : () => Navigator.of(context).pop(),
          child: Text(
            'Cancel',
            style: textTheme.labelLarge?.copyWith(
              color: exportState.isExporting
                  ? colorScheme.onSurfaceVariant
                  : colorScheme.onSurfaceVariant,
            ),
          ),
        ),
        // Share button
        if (exportState.lastExport != null && !exportState.isExporting)
          TextButton.icon(
            onPressed: () {
              widget.onShare?.call();
            },
            icon: const Icon(Icons.share, size: 18),
            label: Text(
              'Share',
              style: textTheme.labelLarge,
            ),
          ),
        // Export button
        ElevatedButton(
          onPressed: exportState.isExporting || _selectedFormat == null
              ? null
              : () => _handleExport(context),
          child: Text(
            exportState.isExporting ? 'Exporting...' : 'Export',
            style: textTheme.labelLarge?.copyWith(
              color: exportState.isExporting || _selectedFormat == null
                  ? colorScheme.onSurfaceVariant
                  : colorScheme.onPrimary,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildFormatOption(
    BuildContext context,
    ExportFormat format,
    bool isExporting,
  ) {
    final colorScheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;
    final isSelected = _selectedFormat == format;

    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap:
            isExporting ? null : () => setState(() => _selectedFormat = format),
        borderRadius: BorderRadius.circular(8),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          decoration: BoxDecoration(
            border: Border.all(
              color: isSelected ? colorScheme.primary : colorScheme.outline,
              width: isSelected ? 2 : 1,
            ),
            borderRadius: BorderRadius.circular(8),
            color: isSelected
                ? colorScheme.primaryContainer.withValues(alpha: 0.3)
                : Colors.transparent,
          ),
          child: Row(
            children: [
              // Icon
              Container(
                width: 40,
                height: 40,
                decoration: BoxDecoration(
                  color: colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Center(
                  child: Text(
                    format.icon,
                    style: const TextStyle(fontSize: 20),
                  ),
                ),
              ),
              const SizedBox(width: 12),
              // Format info
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      format.displayName,
                      style: textTheme.titleSmall,
                    ),
                    Text(
                      format.description,
                      style: textTheme.bodySmall?.copyWith(
                        color: colorScheme.onSurfaceVariant,
                      ),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ),
              ),
              // Radio button
              Radio<ExportFormat>(
                value: format,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _handleExport(BuildContext context) async {
    if (_selectedFormat == null) return;

    try {
      await ref.read(exportNotifierProvider.notifier).exportDocument(
            documentId: widget.documentId,
            format: _selectedFormat!,
          );
      widget.onExportComplete?.call();
    } on Exception {
      // Error is handled by the provider
    }
  }
}

/// Show export dialog helper function
Future<void> showExportDialog({
  required BuildContext context,
  required String documentId,
  required String documentTitle,
  VoidCallback? onExportComplete,
  VoidCallback? onShare,
}) =>
    showDialog<void>(
      context: context,
      builder: (context) => ExportDialog(
        documentId: documentId,
        documentTitle: documentTitle,
        onExportComplete: onExportComplete,
        onShare: onShare,
      ),
    );
