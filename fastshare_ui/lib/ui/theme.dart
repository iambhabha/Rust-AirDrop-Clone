import 'package:flutter/material.dart';

class AppTheme {
  static const Color background = Color(0xFF000000); // Pure black for iOS dark
  static const Color primary = Color(0xFF007AFF); // iOS Blue
  static const Color secondary = Color(0xFF5E5CE6); // iOS Indigo
  static const Color surface = Color(0xFF1C1C1E); // iOS elevated dark

  static ThemeData get darkTheme {
    return ThemeData.dark().copyWith(
      useMaterial3: true,
      scaffoldBackgroundColor: background,
      colorScheme: const ColorScheme.dark(
        primary: primary,
        secondary: secondary,
        surface: surface,
        surfaceContainerHighest: Color(0xFF2C2C2E),
      ),
      textTheme: const TextTheme(
        displayLarge: TextStyle(
          fontFamily: '.SF Pro Display',
          fontWeight: FontWeight.w700,
          fontSize: 34,
          letterSpacing: 0.37,
          color: Colors.white,
        ),
        titleLarge: TextStyle(
          fontFamily: '.SF Pro Text',
          fontWeight: FontWeight.w600,
          fontSize: 22,
          letterSpacing: 0.35,
          color: Colors.white,
        ),
        titleMedium: TextStyle(
          fontFamily: '.SF Pro Text',
          fontWeight: FontWeight.w600,
          fontSize: 17,
          letterSpacing: -0.4,
          color: Colors.white,
        ),
        bodyLarge: TextStyle(
          fontFamily: '.SF Pro Text',
          fontSize: 17,
          letterSpacing: -0.4,
          color: Colors.white70,
        ),
      ),
    );
  }
}
