import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/presentation/pages/search_page.dart';

/// Search bar widget for triggering search
class SearchBar extends ConsumerWidget {
  final bool expanded;
  final VoidCallback? onTap;

  const SearchBar({
    this.expanded = false,
    this.onTap,
    super.key,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Material(
      color: Theme.of(context).colorScheme.surfaceVariant,
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
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.outlineVariant,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  'Ctrl+K',
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
  }

  void _navigateToSearch(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(builder: (context) => const SearchPage()),
    );
  }
}

/// Search app bar widget for use in pages
class SearchAppBar extends ConsumerWidget implements PreferredSizeWidget {
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
  Widget build(BuildContext context, WidgetRef ref) {
    return AppBar(
      title: TextField(
        controller: controller,
        autofocus: true,
        decoration: InputDecoration(
          hintText: hintText ?? 'Search...',
          border: InputBorder.none,
          prefixIcon: const Icon(Icons.search),
          suffixIcon: controller?.text.isNotEmpty == true
              ? IconButton(
                  icon: const Icon(Icons.clear),
                  onPressed: onClear,
                )
              : null,
        ),
        onChanged: onChanged,
        onSubmitted: (_) => onSubmit?.call(),
        textInputAction: TextInputAction.search,
      ),
      actions: [
        if (controller?.text.isNotEmpty == true)
          TextButton(
            onPressed: onSubmit,
            child: const Text('Search'),
          ),
      ],
    );
  }
}
