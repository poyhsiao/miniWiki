import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import 'package:miniwiki/domain/entities/document_version.dart';
import 'package:miniwiki/presentation/providers/version_provider.dart';

/// Page for viewing and managing document version history
class VersionHistoryPage extends ConsumerWidget {
  final String documentId;
  final String documentTitle;

  const VersionHistoryPage({
    required this.documentId, required this.documentTitle, super.key,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final versionListState = ref.watch(versionListNotifierProvider(documentId));
    final comparisonState = ref.watch(versionComparisonNotifierProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text('Version History - $documentTitle'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () {
              ref
                  .read(versionListNotifierProvider(documentId).notifier)
                  .refreshVersions();
            },
            tooltip: 'Refresh versions',
          ),
        ],
      ),
      body: Column(
        children: [
          // Comparison section (if comparing)
          if (comparisonState.canCompare)
            _buildComparisonSection(context, ref, comparisonState),

          // Version list
          Expanded(
            child: _buildVersionList(context, ref, versionListState),
          ),
        ],
      ),
    );
  }

  Widget _buildComparisonSection(
    BuildContext context,
    WidgetRef ref,
    VersionComparisonState state,
  ) => Container(
      padding: const EdgeInsets.all(16),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Comparing Versions',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              TextButton.icon(
                onPressed: () {
                  ref
                      .read(versionComparisonNotifierProvider.notifier)
                      .clearComparison();
                },
                icon: const Icon(Icons.close),
                label: const Text('Clear'),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Row(
            children: [
              Expanded(
                child: _buildVersionChip(
                  context,
                  'From: v${state.fromVersion?.versionNumber}',
                  state.fromVersion?.changeSummary ?? 'No summary',
                ),
              ),
              const Icon(Icons.arrow_forward),
              Expanded(
                child: _buildVersionChip(
                  context,
                  'To: v${state.toVersion?.versionNumber}',
                  state.toVersion?.changeSummary ?? 'No summary',
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          if (state.isComparing)
            const Center(child: CircularProgressIndicator())
          else if (state.diff != null)
            _buildDiffResults(state.diff!)
          else if (state.error != null)
            Text(
              'Error comparing: ${state.error}',
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
        ],
      ),
    );

  Widget _buildVersionChip(
      BuildContext context, String version, String summary) => Card(
      child: Padding(
        padding: const EdgeInsets.all(8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              version,
              style: Theme.of(context).textTheme.labelLarge,
            ),
            Text(
              summary,
              style: Theme.of(context).textTheme.bodySmall,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ],
        ),
      ),
    );

  Widget _buildDiffResults(Map<String, dynamic> diff) {
    final added = diff['added'] as List? ?? [];
    final removed = diff['removed'] as List? ?? [];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (added.isNotEmpty) Text('Added: ${added.length} items'),
        if (removed.isNotEmpty) Text('Removed: ${removed.length} items'),
        if (added.isEmpty && removed.isEmpty)
          const Text('No significant changes'),
      ],
    );
  }

  Widget _buildVersionList(
    BuildContext context,
    WidgetRef ref,
    VersionListState state,
  ) {
    if (state.isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (state.error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              'Error loading versions',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(
              state.error!,
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: () {
                ref
                    .read(versionListNotifierProvider(documentId).notifier)
                    .loadVersions();
              },
              icon: const Icon(Icons.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    if (state.versions.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.history, size: 64, color: Colors.grey),
            SizedBox(height: 16),
            Text('No versions yet'),
            SizedBox(height: 8),
            Text(
              'Document versions will appear here as you make changes',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.grey),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: state.versions.length + (state.hasMore ? 1 : 0),
      itemBuilder: (context, index) {
        if (index == state.versions.length) {
          // Load more indicator
          if (state.isLoadingMore) {
            return const Center(
                child: Padding(
              padding: EdgeInsets.all(16),
              child: CircularProgressIndicator(),
            ));
          }
          return Center(
            child: TextButton(
              onPressed: () {
                ref
                    .read(versionListNotifierProvider(documentId).notifier)
                    .loadMoreVersions();
              },
              child: const Text('Load more versions'),
            ),
          );
        }

        final version = state.versions[index];
        return _VersionListItem(
          version: version,
          documentId: documentId,
          isSelectedForComparison:
              ref.watch(versionComparisonNotifierProvider).fromVersion?.id ==
                      version.id ||
                  ref.watch(versionComparisonNotifierProvider).toVersion?.id ==
                      version.id,
          onCompare: (selectedVersion) {
            ref
                .read(versionComparisonNotifierProvider.notifier)
                .selectFromVersion(selectedVersion);
          },
        );
      },
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('documentId', documentId));
    properties.add(StringProperty('documentTitle', documentTitle));
  }
}

/// List item for a single version
class _VersionListItem extends ConsumerWidget {
  final DocumentVersion version;
  final String documentId;
  final bool isSelectedForComparison;
  final void Function(DocumentVersion) onCompare;

  const _VersionListItem({
    required this.version,
    required this.documentId,
    required this.isSelectedForComparison,
    required this.onCompare,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final dateFormat = DateFormat('MMM d, yyyy h:mm a');
    final formattedDate = version.createdAt != null
        ? dateFormat.format(version.createdAt!)
        : 'Unknown date';

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      color: isSelectedForComparison
          ? Theme.of(context).colorScheme.primaryContainer
          : null,
      child: InkWell(
        onTap: () => onCompare(version),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Row(
                    children: [
                      Chip(
                        label: Text('v${version.versionNumber}'),
                        backgroundColor:
                            Theme.of(context).colorScheme.primaryContainer,
                      ),
                      const SizedBox(width: 8),
                      if (version.restoredFromVersion != null)
                        Chip(
                          label: Text(
                              'Restored from v${version.restoredFromVersion}'),
                          backgroundColor:
                              Theme.of(context).colorScheme.secondaryContainer,
                          visualDensity: VisualDensity.compact,
                        ),
                    ],
                  ),
                  Text(
                    formattedDate,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                version.title,
                style: Theme.of(context).textTheme.titleMedium,
              ),
              if (version.changeSummary != null &&
                  version.changeSummary!.isNotEmpty)
                Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: Text(
                    version.changeSummary!,
                    style: Theme.of(context).textTheme.bodyMedium,
                  ),
                ),
              const SizedBox(height: 8),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  TextButton.icon(
                    onPressed: () => _showRestoreDialog(context, ref),
                    icon: const Icon(Icons.restore),
                    label: const Text('Restore'),
                  ),
                  const SizedBox(width: 8),
                  TextButton.icon(
                    onPressed: () => _viewVersionDetails(context, ref),
                    icon: const Icon(Icons.visibility),
                    label: const Text('View'),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _showRestoreDialog(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (BuildContext dialogContext) => AlertDialog(
        title: const Text('Restore Version?'),
        content: Text(
          'This will create a new version with the content from version '
          '${version.versionNumber}. The current content will not be '
          'overwritten.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () async {
              Navigator.pop(dialogContext);
              try {
                await ref
                    .read(versionListNotifierProvider(documentId).notifier)
                    .restoreVersion(version.versionNumber);
                if (context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                        content: Text('Version restored successfully')),
                  );
                }
              } on Exception catch (e) {
                if (context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text('Failed to restore: $e')),
                  );
                }
              }
            },
            child: const Text('Restore'),
          ),
        ],
      ),
    );
  }

  void _viewVersionDetails(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (BuildContext dialogContext) => AlertDialog(
        title: Text('Version ${version.versionNumber} Details'),
        content: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildDetailRow('Title', version.title),
              _buildDetailRow(
                  'Created', version.createdAt?.toString() ?? 'Unknown'),
              _buildDetailRow(
                  'Change Summary', version.changeSummary ?? 'None'),
              _buildDetailRow(
                  'Version Number', version.versionNumber.toString()),
              if (version.restoredFromVersion != null)
                _buildDetailRow(
                    'Restored From', 'Version ${version.restoredFromVersion}'),
              const SizedBox(height: 16),
              const Text('Content Preview:',
                  style: TextStyle(fontWeight: FontWeight.bold)),
              const SizedBox(height: 8),
              Text(
                version.content.toString(),
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext),
            child: const Text('Close'),
          ),
          ElevatedButton.icon(
            onPressed: () {
              Navigator.pop(dialogContext);
              _showRestoreDialog(context, ref);
            },
            icon: const Icon(Icons.restore),
            label: const Text('Restore This Version'),
          ),
        ],
      ),
    );
  }

  Widget _buildDetailRow(String label, String value) => Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(
              label,
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<DocumentVersion>('version', version));
    properties.add(StringProperty('documentId', documentId));
    properties.add(DiagnosticsProperty<bool>(
        'isSelectedForComparison', isSelectedForComparison));
    properties.add(ObjectFlagProperty<void Function(DocumentVersion)>.has(
        'onCompare', onCompare));
  }
}
