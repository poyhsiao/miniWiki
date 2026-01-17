// Basic Flutter widget test for miniWiki.
//
// This test verifies the app can be built correctly without errors.
// Full widget tests with providers should be added for specific features.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:miniwiki/main.dart';

void main() {
  testWidgets('MiniWiki app builds without errors',
      (WidgetTester tester) async {
    // Build our app with ProviderScope and sized container to avoid layout overflow
    await tester.pumpWidget(
      const SizedBox(
        width: 800,
        height: 600,
        child: ProviderScope(
          child: MiniWikiApp(),
        ),
      ),
    );

    // App should render MaterialApp without errors
    expect(find.byType(MaterialApp), findsOneWidget);
  });
}
