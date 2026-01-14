import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/core/config/providers.dart';
import 'package:miniwiki/core/theme/app_theme.dart';
import 'package:miniwiki/data/datasources/local_storage.dart';
import 'package:miniwiki/presentation/pages/auth/login_page.dart';
import 'package:miniwiki/presentation/pages/home_page.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await initLocalStorage();

  final container = ProviderContainer();
  container.read(appConfigProvider);

  runApp(
    UncontrolledProviderScope(container: container, child: const MiniWikiApp()),
  );
}

class MiniWikiApp extends ConsumerWidget {
  const MiniWikiApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isDarkMode = ref.watch(themeProvider);
    final authState = ref.watch(authProvider);

    return MaterialApp(
      title: 'miniWiki',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.lightTheme,
      darkTheme: AppTheme.darkTheme,
      themeMode: isDarkMode ? ThemeMode.dark : ThemeMode.light,
      home: authState.isAuthenticated ? const HomePage() : const LoginPage(),
      routes: {
        '/login': (_) => const LoginPage(),
        '/home': (_) => const HomePage(),
      },
    );
  }
}
