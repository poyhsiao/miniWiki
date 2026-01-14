import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:miniwiki/domain/entities/document_version.dart';

class VersionComparisonWidget extends StatelessWidget {
  final DocumentVersion fromVersion;
  final DocumentVersion toVersion;
  final Map<String, dynamic> diff;
  final VoidCallback? onRestoreFromVersion;
  final VoidCallback? onRestoreToVersion;

  const VersionComparisonWidget({
    super.key,
    required this.fromVersion,
    required this.toVersion,
    required this.diff,
    this.onRestoreFromVersion,
    this.onRestoreToVersion,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final addedLines = (diff['added'] as List<dynamic>?)?.cast<String>() ?? [];
    final removedLines =
        (diff['removed'] as List<dynamic>?)?.cast<String>() ?? [];
    final modifiedLines =
        (diff['modified'] as List<dynamic>?)?.cast<String>() ?? [];

    return Card(
      margin: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildHeader(theme),
          const Divider(),
          _buildVersionComparison(theme),
          if (addedLines.isNotEmpty ||
              removedLines.isNotEmpty ||
              modifiedLines.isNotEmpty)
            _buildDiffSection(theme, addedLines, removedLines, modifiedLines),
          _buildActions(context, theme),
        ],
      ),
    );
  }

  Widget _buildHeader(ThemeData theme) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Row(
        children: [
          Expanded(
            child: _VersionBadge(
              version: fromVersion,
              label: 'From',
              isNew: false,
            ),
          ),
          const Icon(Icons.arrow_forward, color: Colors.grey),
          Expanded(
            child: _VersionBadge(
              version: toVersion,
              label: 'To',
              isNew: true,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildVersionComparison(ThemeData theme) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        children: [
          Expanded(
            child: _buildVersionInfo(
              theme,
              fromVersion,
              'Version ${fromVersion.versionNumber}',
              Colors.red.shade100,
            ),
          ),
          Expanded(
            child: _buildVersionInfo(
              theme,
              toVersion,
              'Version ${toVersion.versionNumber}',
              Colors.green.shade100,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildVersionInfo(
    ThemeData theme,
    DocumentVersion version,
    String versionLabel,
    Color backgroundColor,
  ) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: backgroundColor,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            versionLabel,
            style: theme.textTheme.titleMedium?.copyWith(
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            version.title,
            style: theme.textTheme.bodyMedium,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 4),
          Text(
            _formatDate(version.createdAt),
            style: theme.textTheme.bodySmall?.copyWith(
              color: Colors.grey[600],
            ),
          ),
          if (version.changeSummary != null &&
              version.changeSummary!.isNotEmpty) ...[
            const SizedBox(height: 8),
            Text(
              version.changeSummary!,
              style: theme.textTheme.bodySmall?.copyWith(
                fontStyle: FontStyle.italic,
              ),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildDiffSection(
    ThemeData theme,
    List<String> added,
    List<String> removed,
    List<String> modified,
  ) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Changes',
            style: theme.textTheme.titleMedium?.copyWith(
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(height: 12),
          if (added.isNotEmpty) ...[
            _buildDiffGroup('Added', added, Colors.green, theme),
            const SizedBox(height: 8),
          ],
          if (removed.isNotEmpty) ...[
            _buildDiffGroup('Removed', removed, Colors.red, theme),
            const SizedBox(height: 8),
          ],
          if (modified.isNotEmpty) ...[
            _buildDiffGroup('Modified', modified, Colors.orange, theme),
          ],
        ],
      ),
    );
  }

  Widget _buildDiffGroup(
    String label,
    List<String> lines,
    Color color,
    ThemeData theme,
  ) {
    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: color.withOpacity(0.5)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: color.withOpacity(0.2),
              borderRadius:
                  const BorderRadius.vertical(top: Radius.circular(7)),
            ),
            child: Text(
              label,
              style: theme.textTheme.labelMedium?.copyWith(
                color: color,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: lines.map((line) {
                return Padding(
                  padding: const EdgeInsets.symmetric(vertical: 2),
                  child: Text(
                    line,
                    style: theme.textTheme.bodySmall?.copyWith(
                      fontFamily: 'monospace',
                    ),
                  ),
                );
              }).toList(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActions(BuildContext context, ThemeData theme) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          TextButton.icon(
            onPressed: onRestoreFromVersion,
            icon: const Icon(Icons.restore, size: 18),
            label: Text('Restore v${fromVersion.versionNumber}'),
          ),
          const SizedBox(width: 16),
          ElevatedButton.icon(
            onPressed: onRestoreToVersion,
            icon: const Icon(Icons.restore, size: 18),
            label: Text('Restore v${toVersion.versionNumber}'),
          ),
        ],
      ),
    );
  }

  String _formatDate(DateTime? date) {
    if (date == null) return 'Unknown';
    return DateFormat('M/d/yyyy HH:mm').format(date);
  }
}

class _VersionBadge extends StatelessWidget {
  final DocumentVersion version;
  final String label;
  final bool isNew;

  const _VersionBadge({
    required this.version,
    required this.label,
    required this.isNew,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: isNew ? Colors.green : Colors.blue,
            borderRadius: BorderRadius.circular(4),
          ),
          child: Text(
            label,
            style: const TextStyle(
              color: Colors.white,
              fontSize: 12,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
        const SizedBox(height: 8),
        Text(
          'v${version.versionNumber}',
          style: TextStyle(
            fontSize: 24,
            fontWeight: FontWeight.bold,
            color: isNew ? Colors.green.shade700 : Colors.blue.shade700,
          ),
        ),
      ],
    );
  }
}
