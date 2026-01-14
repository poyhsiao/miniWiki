import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/space.dart';
import 'package:miniwiki/presentation/providers/space_provider.dart';

class SidebarNavigation extends ConsumerStatefulWidget {
  final double width;
  final VoidCallback onToggle;
  final bool isCollapsed;

  const SidebarNavigation({
    required this.onToggle,
    super.key,
    this.width = 280,
    this.isCollapsed = false,
  });

  @override
  ConsumerState<SidebarNavigation> createState() => _SidebarNavigationState();

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DoubleProperty('width', width));
    properties.add(ObjectFlagProperty<VoidCallback>.has('onToggle', onToggle));
    properties.add(DiagnosticsProperty<bool>('isCollapsed', isCollapsed));
  }
}

class _SidebarNavigationState extends ConsumerState<SidebarNavigation> {
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final spaceState = ref.watch(spaceProvider);
    final spaces = spaceState.spaces;
    final selectedSpace = spaceState.selectedSpace;
    final isLoading = spaceState.isLoading;

    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      width: widget.isCollapsed ? 60 : widget.width,
      curve: Curves.easeInOut,
      child: Drawer(
        child: Column(
          children: [
            _buildHeader(context),
            if (isLoading)
              const LinearProgressIndicator()
            else
              const Divider(height: 1),
            Expanded(
              child: Scrollbar(
                controller: _scrollController,
                child: ListView(
                  controller: _scrollController,
                  padding: EdgeInsets.zero,
                  children: [
                    _buildSpacesSection(context, spaces, selectedSpace),
                    if (!widget.isCollapsed) ...[
                      const Divider(height: 24),
                      _buildSettingsSection(context),
                    ],
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHeader(BuildContext context) => SafeArea(
        bottom: false,
        child: Container(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              IconButton(
                icon: const Icon(Icons.menu),
                onPressed: widget.onToggle,
                tooltip:
                    widget.isCollapsed ? 'Expand sidebar' : 'Collapse sidebar',
              ),
              if (!widget.isCollapsed) ...[
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'miniWiki',
                    style: Theme.of(context).textTheme.titleLarge?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                  ),
                ),
              ],
            ],
          ),
        ),
      );

  Widget _buildSpacesSection(
      BuildContext context, List<Space> spaces, Space? selectedSpace) {
    if (widget.isCollapsed) {
      return Column(
        children: spaces
            .map((space) =>
                _buildCollapsedSpaceItem(context, space, selectedSpace))
            .toList(),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
          child: Row(
            children: [
              Text(
                'SPACES',
                style: Theme.of(context).textTheme.labelSmall?.copyWith(
                      color: Colors.grey[600],
                      fontWeight: FontWeight.w600,
                      letterSpacing: 0.5,
                    ),
              ),
              const Spacer(),
              Tooltip(
                message: 'Create new space',
                child: IconButton(
                  icon: const Icon(Icons.add, size: 18),
                  onPressed: () => _showCreateSpaceDialog(context),
                ),
              ),
            ],
          ),
        ),
        if (spaces.isEmpty)
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              children: [
                Icon(Icons.folder_off, size: 32, color: Colors.grey[400]),
                const SizedBox(height: 8),
                Text(
                  'No spaces yet',
                  style: TextStyle(color: Colors.grey[600], fontSize: 13),
                ),
                const SizedBox(height: 8),
                FilledButton.icon(
                  onPressed: () => _showCreateSpaceDialog(context),
                  icon: const Icon(Icons.add, size: 16),
                  label: const Text('Create Space'),
                  style: FilledButton.styleFrom(
                    padding:
                        const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                  ),
                ),
              ],
            ),
          )
        else
          ...spaces.map((space) => _SpaceItem(
                space: space,
                isSelected: selectedSpace?.id == space.id,
                onTap: () => _selectSpace(context, space),
              )),
      ],
    );
  }

  Widget _buildCollapsedSpaceItem(
    BuildContext context,
    Space space,
    Space? selectedSpace,
  ) {
    final isSelected = selectedSpace?.id == space.id;
    final theme = Theme.of(context);

    return Tooltip(
      message: space.name,
      preferBelow: false,
      child: Material(
        color: isSelected
            ? theme.colorScheme.primaryContainer
            : Colors.transparent,
        child: InkWell(
          onTap: () => _selectSpace(context, space),
          child: Container(
            height: 48,
            alignment: Alignment.center,
            child: Text(
              space.icon ?? '📁',
              style: TextStyle(
                fontSize: 20,
                color: isSelected
                    ? theme.colorScheme.onPrimaryContainer
                    : theme.colorScheme.onSurface,
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildSettingsSection(BuildContext context) => Padding(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
        child: Column(
          children: [
            ListTile(
              leading: const Icon(Icons.settings_outlined, size: 20),
              title: const Text('Settings'),
              dense: true,
              contentPadding: const EdgeInsets.symmetric(horizontal: 8),
              onTap: () {},
            ),
            ListTile(
              leading: const Icon(Icons.help_outline, size: 20),
              title: const Text('Help & Support'),
              dense: true,
              contentPadding: const EdgeInsets.symmetric(horizontal: 8),
              onTap: () {},
            ),
          ],
        ),
      );

  void _selectSpace(BuildContext context, Space space) {
    ref.read(spaceProvider.notifier).selectSpace(space);
    Navigator.pushReplacementNamed(
      context,
      '/spaces/${space.id}',
      arguments: space,
    );
  }

  void _showCreateSpaceDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Create New Space'),
        content: const Text('Space creation dialog would open here.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Create'),
          ),
        ],
      ),
    );
  }
}

class _SpaceItem extends StatefulWidget {
  final Space space;
  final bool isSelected;
  final VoidCallback onTap;

  const _SpaceItem({
    required this.space,
    required this.isSelected,
    required this.onTap,
  });

  @override
  State<_SpaceItem> createState() => _SpaceItemState();

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<Space>('space', space));
    properties.add(DiagnosticsProperty<bool>('isSelected', isSelected));
    properties.add(ObjectFlagProperty<VoidCallback>.has('onTap', onTap));
  }
}

class _SpaceItemState extends State<_SpaceItem> {
  final bool _isExpanded = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Material(
          color: widget.isSelected
              ? theme.colorScheme.primaryContainer.withOpacity(0.3)
              : Colors.transparent,
          child: InkWell(
            onTap: widget.onTap,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
              child: Row(
                children: [
                  SizedBox(
                    width: 24,
                    height: 24,
                    child: Icon(
                      _isExpanded
                          ? Icons.keyboard_arrow_down
                          : Icons.keyboard_arrow_right,
                      size: 18,
                      color: Colors.grey[600],
                    ),
                  ),
                  const SizedBox(width: 4),
                  Container(
                    width: 28,
                    height: 28,
                    decoration: BoxDecoration(
                      color: theme.colorScheme.surfaceContainerHighest,
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Center(
                      child: Text(
                        widget.space.icon ?? '📁',
                        style: const TextStyle(fontSize: 14),
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      widget.space.name,
                      style: TextStyle(
                        fontSize: 14,
                        fontWeight: widget.isSelected
                            ? FontWeight.w600
                            : FontWeight.w400,
                        color: widget.isSelected
                            ? theme.colorScheme.primary
                            : theme.colorScheme.onSurface,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  PopupMenuButton<String>(
                    iconSize: 18,
                    padding: EdgeInsets.zero,
                    onSelected: (value) {},
                    itemBuilder: (context) => [
                      const PopupMenuItem(
                        value: 'settings',
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(Icons.settings, size: 16),
                            SizedBox(width: 8),
                            Text('Settings'),
                          ],
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ),
        if (_isExpanded)
          Padding(
            padding: const EdgeInsets.only(left: 36),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _DocumentTreeItem(
                  title: 'Getting Started',
                  icon: '📄',
                  onTap: () {},
                ),
                _DocumentTreeItem(
                  title: 'Sample Document',
                  icon: '📄',
                  onTap: () {},
                ),
              ],
            ),
          ),
      ],
    );
  }
}

class _DocumentTreeItem extends StatelessWidget {
  final String title;
  final String? icon;
  final VoidCallback onTap;
  final bool isSelected;

  const _DocumentTreeItem({
    required this.title,
    required this.onTap,
    this.icon,
    this.isSelected = false,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Material(
      color: isSelected
          ? theme.colorScheme.primaryContainer.withOpacity(0.3)
          : Colors.transparent,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
          child: Row(
            children: [
              const SizedBox(width: 12),
              Text(
                icon ?? '📄',
                style: const TextStyle(fontSize: 14),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  title,
                  style: TextStyle(
                    fontSize: 13,
                    color: isSelected
                        ? theme.colorScheme.primary
                        : theme.colorScheme.onSurface,
                  ),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('title', title));
    properties.add(StringProperty('icon', icon));
    properties.add(ObjectFlagProperty<VoidCallback>.has('onTap', onTap));
    properties.add(DiagnosticsProperty<bool>('isSelected', isSelected));
  }
}
