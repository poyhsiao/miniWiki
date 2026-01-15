import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/share_link.dart';
import 'package:miniwiki/presentation/providers/share_provider.dart';
import 'package:miniwiki/services/providers.dart';
import 'package:miniwiki/services/share_service.dart';

/// Expiration options for share links
enum ExpirationOption {
  never,
  oneDay,
  oneWeek,
  oneMonth,
  oneYear;

  String get label {
    switch (this) {
      case ExpirationOption.never:
        return 'Never';
      case ExpirationOption.oneDay:
        return '1 day';
      case ExpirationOption.oneWeek:
        return '1 week';
      case ExpirationOption.oneMonth:
        return '1 month';
      case ExpirationOption.oneYear:
        return '1 year';
    }
  }

  DateTime? toDateTime() {
    final now = DateTime.now();
    switch (this) {
      case ExpirationOption.never:
        return null;
      case ExpirationOption.oneDay:
        return now.add(const Duration(days: 1));
      case ExpirationOption.oneWeek:
        return now.add(const Duration(days: 7));
      case ExpirationOption.oneMonth:
        return now.add(const Duration(days: 30));
      case ExpirationOption.oneYear:
        return now.add(const Duration(days: 365));
    }
  }
}

/// Dialog for creating and managing share links.
///
/// This dialog allows users to:
/// - Create new share links with customizable permissions
/// - View existing share links
/// - Copy share links to clipboard
/// - Delete share links
/// - Set optional access codes and expiration dates
class ShareLinkDialog extends ConsumerStatefulWidget {
  final String documentId;
  final String documentTitle;

  const ShareLinkDialog({
    required this.documentId,
    required this.documentTitle,
    super.key,
  });

  @override
  ConsumerState<ShareLinkDialog> createState() => _ShareLinkDialogState();
}

class _ShareLinkDialogState extends ConsumerState<ShareLinkDialog> {
  int _selectedTab = 0;
  ExpirationOption _expirationOption = ExpirationOption.never;

  @override
  void initState() {
    super.initState();
    // Load existing share links
    ref
        .read(shareLinksNotifierProvider(widget.documentId).notifier)
        .loadShareLinks();
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;

    return Dialog(
      backgroundColor: colorScheme.surface,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      child: SizedBox(
        width: 500,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Header
            Container(
              padding: const EdgeInsets.all(20),
              decoration: BoxDecoration(
                border: Border(
                  bottom: BorderSide(color: colorScheme.outlineVariant),
                ),
              ),
              child: Row(
                children: [
                  Icon(
                    Icons.link,
                    color: colorScheme.primary,
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Share Document',
                          style: textTheme.titleLarge,
                        ),
                        Text(
                          widget.documentTitle,
                          style: textTheme.bodyMedium?.copyWith(
                            color: colorScheme.onSurfaceVariant,
                          ),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                    ),
                  ),
                  IconButton(
                    onPressed: () => Navigator.pop(context),
                    icon: const Icon(Icons.close),
                  ),
                ],
              ),
            ),
            // Tabs
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              decoration: BoxDecoration(
                border: Border(
                  bottom: BorderSide(color: colorScheme.outlineVariant),
                ),
              ),
              child: Row(
                children: [
                  _buildTab(context, 'Create', 0),
                  const SizedBox(width: 8),
                  _buildTab(context, 'Manage', 1),
                ],
              ),
            ),
            // Content
            Flexible(
              child: _selectedTab == 0
                  ? _buildCreateTab(context)
                  : _buildManageTab(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTab(BuildContext context, String label, int index) {
    final colorScheme = Theme.of(context).colorScheme;
    final isSelected = _selectedTab == index;

    return Expanded(
      child: GestureDetector(
        onTap: () => setState(() => _selectedTab = index),
        child: Container(
          padding: const EdgeInsets.symmetric(vertical: 12),
          decoration: BoxDecoration(
            color:
                isSelected ? colorScheme.primaryContainer : Colors.transparent,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Text(
            label,
            textAlign: TextAlign.center,
            style: TextStyle(
              color: isSelected
                  ? colorScheme.onPrimaryContainer
                  : colorScheme.onSurfaceVariant,
              fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildCreateTab(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;
    final createState =
        ref.watch(shareLinkCreateNotifierProvider(widget.documentId));
    final createNotifier =
        ref.read(shareLinkCreateNotifierProvider(widget.documentId).notifier);
    final shareService = ref.read(shareServiceProvider);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Permission selection
          Text(
            'Permission',
            style: textTheme.labelLarge,
          ),
          const SizedBox(height: 8),
          SegmentedButton<String>(
            segments: const [
              ButtonSegment(value: 'view', label: Text('View Only')),
              ButtonSegment(value: 'comment', label: Text('Comment')),
            ],
            selected: {createState.permission},
            onSelectionChanged: (Set<String> newValue) {
              createNotifier.setPermission(newValue.first);
            },
          ),
          const SizedBox(height: 20),
          // Access code toggle
          Row(
            children: [
              Text(
                'Require Access Code',
                style: textTheme.labelLarge,
              ),
              const Spacer(),
              Switch(
                value: createState.requireAccessCode,
                onChanged: (value) =>
                    createNotifier.setRequireAccessCode(value),
              ),
            ],
          ),
          if (createState.requireAccessCode) ...[
            const SizedBox(height: 12),
            TextFormField(
              decoration: const InputDecoration(
                labelText: 'Access Code (4+ characters)',
                border: OutlineInputBorder(),
              ),
              obscureText: true,
              onChanged: (value) => createNotifier.setAccessCode(value),
            ),
          ],
          const SizedBox(height: 20),
          // Expiration date
          Row(
            children: [
              Text(
                'Expiration',
                style: textTheme.labelLarge,
              ),
              const Spacer(),
              DropdownButton<ExpirationOption>(
                value: _expirationOption,
                items: ExpirationOption.values.map((option) {
                  return DropdownMenuItem(
                    value: option,
                    child: Text(option.label),
                  );
                }).toList(),
                onChanged: (value) {
                  if (value != null) {
                    setState(() {
                      _expirationOption = value;
                    });
                    createNotifier.setExpiresAt(value.toDateTime());
                  }
                },
              ),
            ],
          ),
          const SizedBox(height: 24),
          // Error message
          if (createState.error != null)
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
                      createState.error!,
                      style: TextStyle(
                        color: colorScheme.error,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          if (createState.error != null) const SizedBox(height: 16),
          // Success message with link
          if (createState.createdLink != null)
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: colorScheme.primaryContainer,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(
                        Icons.check_circle,
                        color: colorScheme.primary,
                        size: 20,
                      ),
                      const SizedBox(width: 8),
                      Text(
                        'Share link created!',
                        style: TextStyle(
                          color: colorScheme.onPrimaryContainer,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 12,
                      vertical: 8,
                    ),
                    decoration: BoxDecoration(
                      color: colorScheme.surface,
                      borderRadius: BorderRadius.circular(8),
                      border: Border.all(color: colorScheme.outline),
                    ),
                    child: Row(
                      children: [
                        Expanded(
                          child: SelectableText(
                            createState.createdLink!
                                .getShareUrl(shareService.baseUrl),
                            style: textTheme.bodySmall,
                          ),
                        ),
                        IconButton(
                          icon: const Icon(Icons.copy),
                          onPressed: () async {
                            final success = await shareService
                                .copyShareLink(createState.createdLink!);
                            if (success && mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                const SnackBar(
                                    content: Text('Link copied to clipboard')),
                              );
                            }
                          },
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          if (createState.createdLink != null) const SizedBox(height: 16),
          // Create button
          SizedBox(
            width: double.infinity,
            child: FilledButton.icon(
              onPressed: createState.isCreating
                  ? null
                  : () => createNotifier.createShareLink(),
              icon: createState.isCreating
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.add_link),
              label: Text(
                  createState.isCreating ? 'Creating...' : 'Create Share Link'),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildManageTab(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;
    final linksState = ref.watch(shareLinksNotifierProvider(widget.documentId));
    final shareService = ref.read(shareServiceProvider);

    if (linksState.isLoading && linksState.shareLinks.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }

    if (linksState.shareLinks.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.link_off,
              size: 48,
              color: colorScheme.onSurfaceVariant,
            ),
            const SizedBox(height: 16),
            Text(
              'No share links yet',
              style: textTheme.titleMedium?.copyWith(
                color: colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'Create a share link to share this document',
              style: textTheme.bodyMedium?.copyWith(
                color: colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 16),
            FilledButton(
              onPressed: () => setState(() => _selectedTab = 0),
              child: const Text('Create Share Link'),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: linksState.shareLinks.length,
      itemBuilder: (context, index) {
        final link = linksState.shareLinks[index];
        return _buildShareLinkItem(context, link, shareService);
      },
    );
  }

  Widget _buildShareLinkItem(
      BuildContext context, ShareLink link, ShareService service) {
    final colorScheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;
    final linksNotifier =
        ref.read(shareLinksNotifierProvider(widget.documentId).notifier);

    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  link.permission == 'comment'
                      ? Icons.rate_review
                      : Icons.visibility,
                  size: 20,
                  color: colorScheme.primary,
                ),
                const SizedBox(width: 8),
                Text(
                  link.permission == 'comment' ? 'Can Comment' : 'View Only',
                  style: TextStyle(
                    color: colorScheme.primary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const Spacer(),
                if (!link.isUsable)
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 8,
                      vertical: 4,
                    ),
                    decoration: BoxDecoration(
                      color: colorScheme.errorContainer,
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      link.isExpired ? 'Expired' : 'Max Uses Reached',
                      style: TextStyle(
                        color: colorScheme.error,
                        fontSize: 12,
                      ),
                    ),
                  ),
              ],
            ),
            const SizedBox(height: 12),
            // Share URL
            Container(
              padding: const EdgeInsets.symmetric(
                horizontal: 12,
                vertical: 8,
              ),
              decoration: BoxDecoration(
                color: colorScheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  Expanded(
                    child: SelectableText(
                      link.getShareUrl(service.baseUrl),
                      style: textTheme.bodySmall,
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.copy, size: 18),
                    onPressed: () async {
                      final success = await linksNotifier.copyShareLink(link);
                      if (success && mounted) {
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(
                              content: Text('Link copied to clipboard')),
                        );
                      }
                    },
                  ),
                ],
              ),
            ),
            const SizedBox(height: 12),
            // Metadata
            Row(
              children: [
                Icon(
                  Icons.schedule,
                  size: 14,
                  color: colorScheme.onSurfaceVariant,
                ),
                const SizedBox(width: 4),
                Text(
                  service.formatExpiration(link.expiresAt),
                  style: textTheme.labelSmall?.copyWith(
                    color: colorScheme.onSurfaceVariant,
                  ),
                ),
                const SizedBox(width: 16),
                Icon(
                  Icons.link,
                  size: 14,
                  color: colorScheme.onSurfaceVariant,
                ),
                const SizedBox(width: 4),
                Text(
                  service.formatAccessCount(
                      link.accessCount, link.maxAccessCount),
                  style: textTheme.labelSmall?.copyWith(
                    color: colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            // Delete button
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton.icon(
                  onPressed: () async {
                    final confirmed = await showDialog<bool>(
                      context: context,
                      builder: (context) => AlertDialog(
                        title: const Text('Delete Share Link'),
                        content: const Text(
                          'Are you sure you want to delete this share link? '
                          'Anyone with the link will no longer be able to access the document.',
                        ),
                        actions: [
                          TextButton(
                            onPressed: () => Navigator.pop(context, false),
                            child: const Text('Cancel'),
                          ),
                          FilledButton(
                            onPressed: () => Navigator.pop(context, true),
                            style: FilledButton.styleFrom(
                              backgroundColor: colorScheme.error,
                            ),
                            child: const Text('Delete'),
                          ),
                        ],
                      ),
                    );

                    if (confirmed == true) {
                      try {
                        await linksNotifier.deleteShareLink(link.token);
                        if (mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(
                              content: Text('Share link deleted successfully'),
                            ),
                          );
                        }
                      } catch (e) {
                        if (mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text('Failed to delete share link: $e'),
                              backgroundColor: colorScheme.error,
                            ),
                          );
                        }
                      }
                    }
                  },
                  icon: const Icon(Icons.delete, size: 18),
                  label: const Text('Delete'),
                  style: TextButton.styleFrom(
                    foregroundColor: colorScheme.error,
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
