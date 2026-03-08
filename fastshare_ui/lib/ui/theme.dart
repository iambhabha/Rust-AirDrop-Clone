import 'package:flutter/material.dart';

class AppTheme {
  // Shadcn/UI Zinc Design System
  static const Color background = Color(0xFF09090B);
  static const Color primary = Color(0xFFF97316); // Rust Orange
  static const Color surface = Color(0xFF09090B);
  static const Color card = Color(0xFF18181B);
  static const Color border = Color(0xFF27272A);
  static const Color foreground = Color(0xFFFAFAFA);
  static const Color mutedForeground = Color(0xFFA1A1AA);

  static ThemeData get darkTheme {
    return ThemeData.dark().copyWith(
      useMaterial3: true,
      scaffoldBackgroundColor: background,
      colorScheme: const ColorScheme.dark(
        primary: primary,
        secondary: primary,
        surface: surface,
        onSurface: foreground,
        onBackground: foreground,
        surfaceContainerHigh: card,
        outline: border,
      ),
      dividerColor: border,
      cardTheme: CardThemeData(
        color: surface,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
          side: const BorderSide(color: border, width: 1),
        ),
      ),
      textTheme: const TextTheme(
        displayLarge: TextStyle(
          fontWeight: FontWeight.bold,
          fontSize: 34,
          letterSpacing: -0.5,
          color: foreground,
        ),
        titleLarge: TextStyle(
          fontWeight: FontWeight.w600,
          fontSize: 22,
          letterSpacing: -0.5,
          color: foreground,
        ),
        titleMedium: TextStyle(
          fontWeight: FontWeight.w600,
          fontSize: 18,
          color: foreground,
        ),
        bodyLarge: TextStyle(fontSize: 16, color: foreground),
        bodyMedium: TextStyle(fontSize: 14, color: mutedForeground),
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: Colors.transparent,
        elevation: 0,
        centerTitle: true,
        titleTextStyle: TextStyle(
          fontWeight: FontWeight.bold,
          fontSize: 18,
          color: foreground,
        ),
      ),
    );
  }
}
