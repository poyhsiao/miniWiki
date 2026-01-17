# Flutter Performance Optimization Guide

**Date**: 2026-01-16
**Purpose**: Guidelines for optimizing Flutter widget rebuild performance

## Performance Principles

### 1. Use `const` Constructors for Widget Trees
Instead of `final`, use `const` for widgets that don't change.

```dart
// ❌ Bad - Constructor cannot be const because field is not final
class MyWidget extends StatelessWidget {
  Widget child; // Mutable field prevents const constructor

  MyWidget({required this.child});

  @override
  Widget build(BuildContext context) {
    // Non-const instantiation rebuilds every time parent rebuilds
    return Container(child: child);
  }
}

// ✅ Good - Immutable field allows const constructor
class MyWidget extends StatelessWidget {
  const MyWidget({super.key, required this.child});
  final Widget child; // Final field allows const

  @override
  Widget build(BuildContext context) {
    // const instantiation is cached and never rebuilds unless dependencies change
    return Container(child: child);
  }
}
```

### 2. Extract Subwidgets into `const` Widgets
Break large widgets into smaller, reusable `const` widgets.

```dart
// ❌ Bad - rebuilds entire structure on every frame
@override
Widget build(BuildContext context) {
  return Scaffold(
    appBar: AppBar(title: const Text('Document')), // Use const for literals
    body: ListView(
      children: [
        Card(child: const Text('Item 1')), // Use const for literals
        Card(child: const Text('Item 2')), // Use const for literals
      ],
    ),
  );
}

// ✅ Good - static AppBar, only rebuilds ListView
class DocumentAppBar extends StatelessWidget implements PreferredSizeWidget {
  const DocumentAppBar({required this.title});
  final String title;

  @override
  Size get preferredSize => const Size.fromHeight(kToolbarHeight);

  @override
  Widget build(BuildContext context) {
    // title is a runtime parameter, so we cannot use const here
    return AppBar(title: Text(title));
  }
}

// In parent
@override
Widget build(BuildContext context) {
  return Scaffold(
    appBar: const DocumentAppBar(title: 'Document'),  // const!
    body: ListView.builder(...),
  );
}
```

### 3. Use `ListView.builder` Instead of `Column` for Long Lists
For lists with many items, `ListView.builder` only renders visible items.

```dart
// ❌ Bad - renders all items, even off-screen
@override
Widget build(BuildContext context) {
  return Column(
    children: documents.map((doc) => DocumentTile(doc)).toList(),
  );
}

// ✅ Good - lazy loading, only renders visible items
@override
Widget build(BuildContext context) {
  return ListView.builder(
    itemCount: documents.length,
    itemBuilder: (context, index) {
      return DocumentTile(documents[index]);
    },
  );
}
```

### 4. Use Provider Selectors for Granular Rebuilds
Only listen to specific data you need, not the entire state.

```dart
// ❌ Bad - rebuilds when any user state changes
class UserEmailWidget extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(appStateProvider);
    return Text(state.user.email);
  }
}

// ✅ Good - only rebuilds when user.email changes
class UserEmailWidget extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final email = ref.watch(appStateProvider.select((state) => state.user.email));
    return Text(email);
  }
}
```

### 5. Use Automatic `keepAlive` for Tab Views
Prevent widget disposal in tabs with `AutomaticKeepAliveClientMixin`.

```dart
class _DocumentTabState extends State<DocumentTab>
    with AutomaticKeepAliveClientMixin {
  @override
  bool get wantKeepAlive => true;

  @override
  Widget build(BuildContext context) {
    super.build(context);
    return YourDocumentContent();
  }
}
```

### 6. Optimize Riverpod Providers
Use `select` for computed values instead of watching entire providers.

```dart
// ❌ Bad - rebuilds every time documents list changes
final docsCountProvider = Provider.autoDispose<int>((ref) {
  final docs = ref.watch(documentsProvider);
  return docs.length;
});

// ✅ Good - only recomputes if length changes
final docsCountProvider = Provider.autoDispose<int>((ref) {
  return ref.watch(documentsProvider.select((docs) => docs.length));
});
```

### 7. Use `RepaintBoundary` for Isolated Repaints
Wrap expensive widgets to limit repaint scope.

```dart
RepaintBoundary(
  child: ExpensiveChartWidget(),
)
```

### 8. Optimize Functions in `build()` Method
Creating inline closures inside `build()` (e.g., passing an anonymous function to `ListView.builder`'s `itemBuilder`) is generally acceptable in Flutter. However, it can become a performance concern if the closure captures large state, is in a hot path invoked very frequently, or prevents compiler optimizations.

```dart
// ⚠️ Potential Issue - captures heavy state or prevents simple optimizations
@override
Widget build(BuildContext context) {
  final heavyState = ...;
  return ListView.builder(
    itemBuilder: (context, index) {
      return Card(
        child: Text("${items[index]} $heavyState"),
      );
    },
  );
}

// ✅ Better - Improved readability and avoids accidental capture

class DocumentList extends StatelessWidget {
  // Define items clearly in the class context
  final List<String> items = const ['Item 1', 'Item 2', 'Item 3'];

  const DocumentList({super.key});

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      itemCount: items.length,
      itemBuilder: _buildItem,
    );
  }

  Widget _buildItem(BuildContext context, int index) {
    return Card(child: Text(items[index]));
  }
}

```
Extracting to a method like `_buildItem` improves organization, enables potential method-level optimizations, and prevents accidental closure over heavy objects within the `build` method.

### 9. Use `const` Values for Primitive Types
Use `const` for numbers, strings, and booleans in widget trees.

```dart
// ❌ Bad
Container(
  padding: EdgeInsets.all(16),  // Creates new instance
)

// ✅ Good
Container(
  padding: const EdgeInsets.all(16),  // Reuses same instance
)
```

### 10. Lazy Load Heavy Widgets
Defer initialization of expensive widgets until needed.

```dart
// ✅ Good - lazy loading with ListView.builder
@override
Widget build(BuildContext context) {
  return ListView.builder(
    itemCount: items.length,
    itemBuilder: (context, index) {
      // items are only created when visible/near viewport
      return HeavyItemWidget(item: items[index]);
    },
  );
}
```

```dart
// ✅ Good - threshold-based pagination with ScrollController
class PaginatedList extends StatefulWidget {
  @override
  _PaginatedListState createState() => _PaginatedListState();
}

class _PaginatedListState extends State<PaginatedList> {
  final ScrollController _scrollController = ScrollController();
  // Internal collection to store items (loaded from API or passed via constructor)
  final List<String> items = [];
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_onScroll);
    // Trigger initial load to populate items before user scrolls
    _loadInitial();
  }

  Future<void> _loadInitial() async {
    if (mounted) {
      setState(() => _isLoading = true);
    }
    await _loadMore();
  }

  void _onScroll() {
    if (_scrollController.position.pixels >= _scrollController.position.maxScrollExtent - 200) {
      if (!_isLoading) {
        // Set loading flag synchronously to prevent multiple triggers
        setState(() => _isLoading = true);
        _loadMore();
      }
    }
  }

  Future<void> _loadMore() async {
    // _isLoading is already set to true by _onScroll or _loadInitial
    try {
      // Simulate API fetch
      await Future.delayed(const Duration(seconds: 1));

      if (mounted) {
        setState(() {
          items.addAll(List.generate(20, (i) => "New Item ${items.length + i}"));
          _isLoading = false;
        });
      }
    } catch (e) {
      // Log or handle the error
      debugPrint('_loadMore failed: $e');
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      controller: _scrollController,
      itemCount: items.length + (_isLoading ? 1 : 0),
      itemBuilder: (context, index) {
        if (index < items.length) return ListTile(title: Text(items[index]));
        return const Center(child: CircularProgressIndicator());
      },
    );
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }
}
```

## Performance Monitoring

### Enable Flutter Performance Overlay
Add to your `main.dart`:

```dart
import 'package:flutter/foundation.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();

  // Note: debugProfileBuildsEnabled adds Timeline events for every Widget built,
  // which makes frame timings non-representative. Only use for widget-build debugging,
  // not when running 'flutter run --profile' for accurate performance analysis.
  if (kDebugMode) {
    debugProfileBuildsEnabled = true;
  }

  runApp(MyApp());
}
```

> **Important**: `debugProfileBuildsEnabled` instruments every widget build with timeline events. This adds significant overhead and skews performance measurements. Only enable it when specifically debugging widget build issues in debug mode. Use `flutter run --profile` without this flag for accurate performance analysis.

### Run Performance Profile
```bash
# Run in profile mode for accurate analysis
flutter run --profile
# (Interact with app, observe Performance Overlay)
```

### Analyze Performance with DevTools
1. Run Flutter app in **profile mode**: `flutter run --profile`
2. Open DevTools:
   - **IDE Integration (Recommended)**: Click "Open DevTools" in VS Code (Command Palette: `Flutter: Open DevTools`) or Android Studio.
   - **CLI Link**: Use the DevTools URL printed in the terminal by `flutter run`.
   - **Standalone**: Run `dart devtools` (DevTools is included in the Flutter/Dart toolchain).
3. Navigate to **Performance** tab.
4. Record interaction and analyze frame times (look for `debugProfileBuildsEnabled` impact if enabled).

## Optimized Component Examples

### Document List Page
Uses `ListView.builder`, const constructors, and provider selectors.

### Document Editor Page
Uses `RepaintBoundary` for editor, lazy loading for attachments.

### Space Navigation
Uses `const` widgets, `AutomaticKeepAliveClientMixin` for tabs.

## Common Anti-Patterns to Avoid

1. ❌ Calling `setState()` inside `build()`
   → Move to lifecycle methods (`initState()`), event handlers, or use `FutureBuilder`/`StreamBuilder`.
   → Maintain pure build logic by deriving UI from immutable state or controllers.

2. ❌ Using `FutureBuilder` for already available data
   → Load data in `initState()` and use `ValueNotifier`

3. ❌ Nesting too many levels of widgets
   → Extract to custom widgets

4. ❌ Not using keys in lists
   → Always add `key:` parameter

5. ❌ Creating expensive widgets in `build()`
   → Cache or lazy load them

## Performance Targets

- **Frame time**: <16ms (60 FPS)
- **Build time**: ≤8ms for build pass + ≤8ms for raster pass (≈16ms total); exceptions allowed for explicitly complex widgets if frame drops are avoided.
- **Startup time**: Project target: time to first frame <3s (refer to Flutter docs for measuring time-to-first-frame).
- **Navigation time**: Project target: navigation <500ms.

## Riverpod Family (Parameterized Providers)

### What is Riverpod Family?

The `.family` modifier in Riverpod allows you to create **parameterized providers** that can accept arguments. This is useful when you need multiple instances of a provider with different configurations or when you need to fetch data based on dynamic parameters.

### When to Use Family

```dart
// ✅ Good use case - fetching document by ID
final documentProvider = FutureProvider.family<Document, String>((ref, id) async {
  final repository = ref.watch(documentRepositoryProvider);
  return repository.getDocument(id);
});

// Usage in widget
class DocumentView extends ConsumerWidget {
  final String documentId;

  const DocumentView({
    required this.documentId,
    super.key,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final documentAsync = ref.watch(documentProvider(documentId));
    return documentAsync.when(
      data: (doc) => Text(doc.title),
      loading: () => CircularProgressIndicator(),
      error: (err, stack) => Text('Error: $err'),
    );
  }
}
```

### Performance Implications

**Lifecycle & Caching:**
- Each unique parameter creates a **separate provider instance** with its own state and cache
- Instances are kept alive as long as they have listeners
- When all listeners are removed, the instance is disposed (with `.autoDispose`)

**Best Practices:**

1. **Use `.autoDispose` with Family** to prevent memory leaks:
```dart
// ✅ Good - automatically cleans up unused instances
final documentProvider = FutureProvider.autoDispose.family<Document, String>(
  (ref, id) async {
    final repository = ref.watch(documentRepositoryProvider);
    return repository.getDocument(id);
  },
);
```

2. **Avoid creating too many instances** - each parameter combination creates a new instance:
```dart
// ⚠️ Potential issue - creates many instances
final filteredDocsProvider = Provider.family<List<Document>, FilterParams>(
  (ref, params) {
    final docs = ref.watch(allDocumentsProvider);
    return docs.where((d) => d.matches(params)).toList();
  },
);

// ✅ Better - use select for simple filtering
final filteredDocsProvider = Provider<List<Document>>((ref) {
  final docs = ref.watch(allDocumentsProvider);
  final filter = ref.watch(filterStateProvider);
  return docs.where((d) => d.matches(filter)).toList();
});
```

3. **Use immutable parameters** - parameters should implement `==` and `hashCode`:
```dart
// ✅ Good - immutable parameter class
@immutable
class DocumentQuery {
  final String spaceId;
  final String? searchTerm;

  const DocumentQuery({required this.spaceId, this.searchTerm});

  @override
  bool operator ==(Object other) =>
    identical(this, other) ||
    other is DocumentQuery &&
    spaceId == other.spaceId &&
    searchTerm == other.searchTerm;

  @override
  int get hashCode => Object.hash(spaceId, searchTerm);
}

final documentsProvider = FutureProvider.autoDispose.family<List<Document>, DocumentQuery>(
  (ref, query) async {
    // Fetch documents based on query
  },
);
```

**Performance Comparison:**

| Approach | Memory Usage | Recomputation | Best For |
|----------|-------------|---------------|----------|
| Singleton Provider | Low | Only when dependencies change | Global state, shared data |
| Family Provider | Medium-High (per instance) | Per parameter combination | Dynamic data fetching, parameterized queries |
| Family + autoDispose | Medium (auto-cleanup) | Per parameter, auto-disposed | Most dynamic use cases |

### Summary

- **Use Family** when you need parameterized providers (e.g., fetching by ID)
- **Always use `.autoDispose`** with family to prevent memory leaks
- **Keep parameters immutable** and implement proper equality
- **Avoid excessive instances** - consider alternatives for simple filtering/transformation

## References

- [Flutter Performance Best Practices](https://docs.flutter.dev/perf/best-practices)
- [Flutter UI Performance](https://docs.flutter.dev/perf/ui-performance)
- [Riverpod Family Modifier](https://riverpod.dev/docs/concepts2/family)

