import 'package:flutter/material.dart';

class AppColors {
  static const primary = Color(0xFF6750A4);
  static const onPrimary = Color(0xFFFFFFFF);
  static const primaryContainer = Color(0xFFEADDFF);
  static const onPrimaryContainer = Color(0xFF21005D);
  static const secondary = Color(0xFF625B71);
  static const onSecondary = Color(0xFFFFFFFF);
  static const secondaryContainer = Color(0xFFE8DEF8);
  static const onSecondaryContainer = Color(0xFF1D192B);
  static const tertiary = Color(0xFF7D5260);
  static const onTertiary = Color(0xFFFFFFFF);
  static const tertiaryContainer = Color(0xFFFFD8E4);
  static const onTertiaryContainer = Color(0xFF31111D);
  static const error = Color(0xFFB3261E);
  static const onError = Color(0xFFFFFFFF);
  static const errorContainer = Color(0xFFF9DEDC);
  static const onErrorContainer = Color(0xFF410E0B);
  static const background = Color(0xFFFFFBFE);
  static const onBackground = Color(0xFF1C1B1F);
  static const surface = Color(0xFFFFFBFE);
  static const onSurface = Color(0xFF1C1B1F);
  static const surfaceVariant = Color(0xFFE7E0EC);
  static const onSurfaceVariant = Color(0xFF49454F);
  static const outline = Color(0xFF79747E);
  static const outlineVariant = Color(0xFFCAC4D0);
}

class AppTheme {
  static final lightTheme = ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(
      seedColor: AppColors.primary,
    ),
    fontFamily: 'Inter',
  );

  static final darkTheme = ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(
      seedColor: AppColors.primary,
      brightness: Brightness.dark,
    ),
    fontFamily: 'Inter',
  );
}
