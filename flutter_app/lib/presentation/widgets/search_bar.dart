import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:miniwiki/presentation/pages/search_page.dart';

/// Search bar widget for triggering search
class WikiSearchBar extends StatelessWidget {
  final VoidCallback? onTap;

  const WikiSearchBar({
    this.onTap,
    super.key,
  });

  /// Returns the platform-aware keyboard shortcut display string
  String _getShortcutDisplay(BuildContext context) {
    final isMacOS = defaultTargetPlatform == TargetPlatform.macOS;
    return isMacOS ? '⌘K' : 'Ctrl+K';
  }

  @override
  Widget build(BuildContext context) => Material(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
        child: InkWell(
          onTap: onTap ?? () => _navigateToSearch(context),
          borderRadius: BorderRadius.circular(8),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            child: Row(
              children: [
                Icon(
                  Icons.search,
                  size: 20,
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
                const SizedBox(width: 8),
                Text(
                  'Search documents...',
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                    fontSize: 14,
                  ),
                ),
                const Spacer(),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.outlineVariant,
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    _getShortcutDisplay(context),
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.onSurfaceVariant,
                      fontSize: 12,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      );

  void _navigateToSearch(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(builder: (BuildContext context) => const SearchPage()),
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(ObjectFlagProperty<VoidCallback?>.has('onTap', onTap));
  }
}

/// Search app bar widget for use in pages
class SearchAppBar extends StatefulWidget implements PreferredSizeWidget {
  final TextEditingController? controller;
  final String? hintText;
  final ValueChanged<String>? onChanged;
  final VoidCallback? onClear;
  final VoidCallback? onSubmit;

  const SearchAppBar({
    this.controller,
    this.hintText,
    this.onChanged,
    this.onClear,
    this.onSubmit,
    super.key,
  });

  @override
  Size get preferredSize => const Size.fromHeight(kToolbarHeight);

  @override
  State<SearchAppBar> createState() => _SearchAppBarState();

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<TextEditingController?>('controller', controller));
    properties.add(StringProperty('hintText', hintText));
    properties.add(ObjectFlagProperty<ValueChanged<String>?>.has('onChanged', onChanged));
    properties.add(ObjectFlagProperty<VoidCallback?>.has('onClear', onClear));
    properties.add(ObjectFlagProperty<VoidCallback?>.has('onSubmit', onSubmit));
  }
}

class _SearchAppBarState extends State<SearchAppBar> {
  late final TextEditingController _fallbackController;

  @override
  void initState() {
    super.initState();
    _fallbackController = TextEditingController();
  }

  @override
  void dispose() {
    _fallbackController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final effectiveController = widget.controller ?? _fallbackController;
    return AppBar(
      title: ValueListenableBuilder<TextEditingValue>(
        valueListenable: effectiveController,
        builder: (context, value, child) => TextField(
          controller: effectiveController,
          autofocus: true,
          decoration: InputDecoration(
            hintText: widget.hintText ?? 'Search...',
            border: InputBorder.none,
            prefixIcon: const Icon(Icons.search),
            suffixIcon: value.text.isNotEmpty
                ? IconButton(
                    icon: const Icon(Icons.clear),
                    onPressed: () {
                      effectiveController.clear();
                      widget.onClear?.call();
                    },
                  )
                : null,
          ),
          onChanged: widget.onChanged,
          onSubmitted: (_) => widget.onSubmit?.call(),
          textInputAction: TextInputAction.search,
        ),
      ),
      actions: [
        ValueListenableBuilder<TextEditingValue>(
          valueListenable: effectiveController,
          builder: (context, value, child) => value.text.isNotEmpty
              ? TextButton(
                  onPressed: widget.onSubmit,
                  child: const Text('Search'),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<TextEditingController?>(
        'controller', widget.controller));
    properties.add(StringProperty('hintText', widget.hintText));
    properties.add(ObjectFlagProperty<ValueChanged<String>?>.has(
        'onChanged', widget.onChanged));
    properties
        .add(ObjectFlagProperty<VoidCallback?>.has('onClear', widget.onClear));
    properties.add(
        ObjectFlagProperty<VoidCallback?>.has('onSubmit', widget.onSubmit));
  }
}
