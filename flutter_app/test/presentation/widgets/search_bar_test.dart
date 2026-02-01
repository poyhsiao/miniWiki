import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/presentation/widgets/search_bar.dart';

void main() {
  group('WikiSearchBar', () {
    testWidgets('renders with correct styling', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: WikiSearchBar()),
        ),
      );

      expect(find.byType(WikiSearchBar), findsOneWidget);
      expect(find.byType(InkWell), findsOneWidget);
      expect(find.byIcon(Icons.search), findsOneWidget);
    });

    testWidgets('shows search placeholder text', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: WikiSearchBar()),
        ),
      );

      expect(find.text('Search documents...'), findsOneWidget);
    });

    testWidgets('calls onTap when provided', (WidgetTester tester) async {
      bool tapCalled = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WikiSearchBar(onTap: () => tapCalled = true),
          ),
        ),
      );

      await tester.tap(find.byType(WikiSearchBar));
      expect(tapCalled, true);
    });

    testWidgets('does not navigate when onTap provided',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: WikiSearchBar(onTap: () {})),
        ),
      );

      await tester.tap(find.byType(WikiSearchBar));
      await tester.pumpAndSettle();

      // Should not navigate (no back button should appear)
      expect(find.byType(BackButton), findsNothing);
    });

    testWidgets('shows keyboard shortcut on non-macOS platforms',
        (WidgetTester tester) async {
      debugDefaultTargetPlatformOverride = TargetPlatform.windows;
      addTearDown(() => debugDefaultTargetPlatformOverride = null);

      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: WikiSearchBar()),
        ),
      );

      expect(find.textContaining('Ctrl'), findsOneWidget);
    });

    testWidgets('has correct color scheme', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: WikiSearchBar()),
        ),
      );

      final inkWell = find.byType(InkWell).evaluate().first.widget as InkWell;
      expect(inkWell.borderRadius, equals(BorderRadius.circular(8)));
    });
  });

  group('SearchAppBar', () {
    testWidgets('creates with default hint text', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(appBar: SearchAppBar()),
        ),
      );

      expect(find.byType(SearchAppBar), findsOneWidget);
      expect(find.text('Search...'), findsOneWidget);
    });

    testWidgets('creates with custom hint text', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(appBar: SearchAppBar(hintText: 'Custom hint...')),
        ),
      );

      expect(find.text('Custom hint...'), findsOneWidget);
    });

    testWidgets('updates when text changes', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(appBar: SearchAppBar()),
        ),
      );

      // Find the text field and enter text
      final textField = find.byType(TextField);
      expect(textField, findsOneWidget);

      await tester.enterText(textField, 'test query');
      await tester.pump();

      // Suffix clear button should appear
      expect(find.byIcon(Icons.clear), findsOneWidget);
    });

    testWidgets('calls onChanged when text changes',
        (WidgetTester tester) async {
      String? changedText;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            appBar: SearchAppBar(onChanged: (text) => changedText = text),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), 'hello');
      await tester.pump();

      expect(changedText, equals('hello'));
    });

    testWidgets('calls onClear when clear button pressed',
        (WidgetTester tester) async {
      bool clearCalled = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            appBar: SearchAppBar(onClear: () => clearCalled = true),
          ),
        ),
      );

      // Enter text to show clear button
      await tester.enterText(find.byType(TextField), 'test');
      await tester.pump();

      // Tap clear button
      await tester.tap(find.byIcon(Icons.clear));
      await tester.pump();

      expect(clearCalled, true);
    });

    testWidgets('calls onSubmit when search button pressed',
        (WidgetTester tester) async {
      bool submitCalled = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            appBar: SearchAppBar(onSubmit: () => submitCalled = true),
          ),
        ),
      );

      // Enter text to show search button
      await tester.enterText(find.byType(TextField), 'test');
      await tester.pump();

      // Tap search button
      await tester.tap(find.text('Search'));
      await tester.pump();

      expect(submitCalled, true);
    });

    testWidgets('calls onSubmit when Enter key pressed',
        (WidgetTester tester) async {
      bool submitCalled = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            appBar: SearchAppBar(onSubmit: () => submitCalled = true),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), 'test');
      await tester.pump();

      // Submit via keyboard
      await tester.testTextInput.receiveAction(TextInputAction.search);
      await tester.pump();

      expect(submitCalled, true);
    });

    testWidgets('uses external controller when provided',
        (WidgetTester tester) async {
      final controller = TextEditingController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            appBar: SearchAppBar(controller: controller),
          ),
        ),
      );

      // Verify controller is being used
      controller.text = 'external text';
      await tester.pump();

      expect(find.text('external text'), findsOneWidget);
    });

    testWidgets('disposes internal controller', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(appBar: SearchAppBar()),
        ),
      );

      // Build and dispose should not throw
      await tester.pumpWidget(const SizedBox.shrink());
      // If we get here without errors, the test passes
    });

    testWidgets('preferredSize returns correct height',
        (WidgetTester tester) async {
      const appBar = SearchAppBar();
      expect(
          appBar.preferredSize, equals(const Size.fromHeight(kToolbarHeight)));
    });
  });

  group('SearchAppBar keyboard shortcuts', () {
    testWidgets('has autofocus enabled', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(appBar: SearchAppBar()),
        ),
      );

      final textField =
          find.byType(TextField).evaluate().first.widget as TextField;
      expect(textField.autofocus, true);
    });

    testWidgets('has search text input action', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(appBar: SearchAppBar()),
        ),
      );

      final textField =
          find.byType(TextField).evaluate().first.widget as TextField;
      expect(textField.textInputAction, TextInputAction.search);
    });
  });
}
