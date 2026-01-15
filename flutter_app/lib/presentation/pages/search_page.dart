import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/search_result.dart';
import 'package:miniwiki/presentation/providers/search_provider.dart';
import 'package:miniwiki/services/search_service.dart';

/// Search page for full-text document search
class SearchPage extends ConsumerStatefulWidget {
  final String? initialQuery;

  const SearchPage({
    this.initialQuery,
    super.key,
  });

  @override
  ConsumerState<SearchPage> createState() => _SearchPageState();

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('initialQuery', initialQuery));
  }
}

class _SearchPageState extends ConsumerState<SearchPage> {
  late final TextEditingController _queryController;
  final FocusNode _focusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    _queryController = TextEditingController(text: widget.initialQuery ?? '');
    if (widget.initialQuery?.isNotEmpty == true) {
      Future.microtask(
        () => ref
            .read(searchStateProvider.notifier)
            .searchImmediate(widget.initialQuery!),
      );
    }
    _focusNode.requestFocus();
  }

  @override
  void dispose() {
    _queryController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void _onQueryChanged(String query) {
    ref.read(searchStateProvider.notifier).search(query);
  }

  void _onClear() {
    _queryController.clear();
    ref.read(searchStateProvider.notifier).clear();
    _focusNode.requestFocus();
  }

  void _onSearch(String query) {
    if (query.trim().isNotEmpty) {
      ref.read(searchStateProvider.notifier).searchImmediate(query.trim());
    }
  }

  void _onResultTap(BuildContext context, SearchResult result) {
    // Navigate to document detail
    Navigator.of(context)
        .pop({'documentId': result.documentId, 'title': result.title});
  }

  @override
  Widget build(BuildContext context) {
    final searchState = ref.watch(searchStateProvider);
    final query = _queryController.text;

    return Scaffold(
      appBar: AppBar(
        title: TextField(
          controller: _queryController,
          focusNode: _focusNode,
          autofocus: true,
          decoration: InputDecoration(
            hintText: 'Search documents...',
            border: InputBorder.none,
            prefixIcon: const Icon(Icons.search),
            suffixIcon: IconButton(
              icon: const Icon(Icons.clear),
              onPressed: _onClear,
            ),
          ),
          onChanged: _onQueryChanged,
          onSubmitted: _onSearch,
          textInputAction: TextInputAction.search,
        ),
        actions: [
          if (query.isNotEmpty)
            TextButton(
              onPressed: () => _onSearch(query),
              child: const Text('Search'),
            ),
        ],
      ),
      body: _buildBody(context, searchState),
    );
  }

  Widget _buildBody(
      BuildContext context, AsyncValue<List<SearchResult>> state) {
    return state.when(
      data: (results) {
        if (_queryController.text.isEmpty) {
          return _buildEmptyState(context);
        }
        if (results.isEmpty) {
          return _buildNoResults(context);
        }
        return _buildResultsList(context, results);
      },
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (error, stack) => _buildErrorState(context, error.toString()),
    );
  }

  Widget _buildEmptyState(BuildContext context) => Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.search,
              size: 64,
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
            const SizedBox(height: 16),
            Text(
              'Search your documents',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              'Enter keywords to find documents across all your spaces',
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
          ],
        ),
      );

  Widget _buildNoResults(BuildContext context) => Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.search_off,
              size: 64,
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
            const SizedBox(height: 16),
            Text(
              'No results found',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              'Try different keywords or check your spelling',
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
          ],
        ),
      );

  Widget _buildErrorState(BuildContext context, String error) => Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.error_outline,
              size: 64,
              color: Theme.of(context).colorScheme.error,
            ),
            const SizedBox(height: 16),
            Text(
              'Search failed',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              error,
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
          ],
        ),
      );

  Widget _buildResultsList(BuildContext context, List<SearchResult> results) {
    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: results.length,
      itemBuilder: (context, index) =>
          _buildResultItem(context, results[index]),
    );
  }

  Widget _buildResultItem(BuildContext context, SearchResult result) {
    final query = _queryController.text;
    final titleSpans = SearchService.highlightQuery(
      result.title,
      query,
      baseStyle: Theme.of(context).textTheme.titleMedium,
      highlightStyle: Theme.of(context).textTheme.titleMedium?.copyWith(
        fontWeight: FontWeight.bold,
        backgroundColor: Colors.yellow.withOpacity(0.3),
      ),
    );
    final snippetSpans = SearchService.highlightQuery(
      result.snippet,
      query,
      baseStyle: Theme.of(context).textTheme.bodyMedium?.copyWith(
        color: Theme.of(context).colorScheme.onSurfaceVariant,
      ),
      highlightStyle: Theme.of(context).textTheme.bodyMedium?.copyWith(
        color: Theme.of(context).colorScheme.onSurfaceVariant,
        fontWeight: FontWeight.bold,
        backgroundColor: Colors.yellow.withOpacity(0.3),
      ),
    );

    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: InkWell(
        onTap: () => _onResultTap(context, result),
        borderRadius: BorderRadius.circular(8),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(Icons.description_outlined, size: 20),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text.rich(
                      TextSpan(children: titleSpans),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                result.spaceName,
                style: Theme.of(context).textTheme.labelSmall?.copyWith(
                      color: Theme.of(context).colorScheme.primary,
                    ),
              ),
              if (result.snippet.isNotEmpty) ...[
                const SizedBox(height: 8),
                Text.rich(
                  TextSpan(children: snippetSpans),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
