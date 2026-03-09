import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

class AppTheme {
  // Shadcn/UI Zinc Design System
  static const Color background = Color(0xFF09090B);
  static const Color primary = Color(0xFFF97316); // Rust Orange
  static const Color accent = Color(0xFFFFB800); // Amber/Gold
  static const Color highlight = Color(0xFFFF4D00); // Vibrant Red-Orange

  static Gradient get primaryGradient => const LinearGradient(
    colors: [
      Color(0xFFFFB800), // Amber
      Color(0xFFF97316), // Rust
      Color(0xFFFF4D00), // Pulse Red-Orange
    ],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    stops: [0.0, 0.5, 1.0],
  );

  static Gradient get primaryGlow => RadialGradient(
    colors: [
      const Color(0xFFF97316).withOpacity(0.4),
      const Color(0xFFFF4D00).withOpacity(0.1),
      Colors.transparent,
    ],
    radius: 0.8,
  );

  static const Color secondary = Color(0xFFF97316); // Brand Orange
  static const Color secondaryLight = Color(0xFFFFB800); // Golden Amber
  static const Color secondaryDark = Color(0xFFEA580C); // Deep Burnt Orange

  static Gradient get secondaryGradient => const LinearGradient(
    colors: [
      Color(0xFFFFD60A), // Bright Gold
      Color(0xFFF97316), // Brand Orange
      Color(0xFFFFB800), // Amber
    ],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    stops: [0.0, 0.5, 1.0],
  );

  static const Color surface = Color(0xFF09090B);
  static const Color card = Color(0xFF18181B);
  static const Color border = Color(0xFF27272A);
  static const Color foreground = Color(0xFFFAFAFA);
  static const Color mutedForeground = Color(0xFFA1A1AA);

  static ThemeData get darkTheme {
    final textTheme = GoogleFonts.plusJakartaSansTextTheme(
      const TextTheme(
        displayLarge: TextStyle(
          fontWeight: FontWeight.bold,
          fontSize: 34,
          letterSpacing: -1.2,
          color: foreground,
        ),
        displayMedium: TextStyle(
          fontWeight: FontWeight.bold,
          fontSize: 28,
          letterSpacing: -1.0,
          color: foreground,
        ),
        titleLarge: TextStyle(
          fontWeight: FontWeight.w700,
          fontSize: 22,
          letterSpacing: -0.6,
          color: foreground,
        ),
        titleMedium: TextStyle(
          fontWeight: FontWeight.w600,
          fontSize: 18,
          letterSpacing: -0.4,
          color: foreground,
        ),
        bodyLarge: TextStyle(
          fontSize: 16,
          color: foreground,
          letterSpacing: -0.2,
        ),
        bodyMedium: TextStyle(
          fontSize: 14,
          color: mutedForeground,
          letterSpacing: -0.1,
        ),
        bodySmall: TextStyle(fontSize: 12, color: mutedForeground),
        labelLarge: TextStyle(
          fontWeight: FontWeight.w600,
          fontSize: 14,
          letterSpacing: 0.1,
          color: foreground,
        ),
      ),
    );

    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      scaffoldBackgroundColor: background,
      fontFamily: GoogleFonts.plusJakartaSans().fontFamily,
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
          borderRadius: BorderRadius.circular(12),
          side: const BorderSide(color: border, width: 1),
        ),
      ),
      textTheme: textTheme,
      primaryTextTheme: textTheme,
      appBarTheme: AppBarTheme(
        backgroundColor: Colors.transparent,
        elevation: 0,
        centerTitle: true,
        titleTextStyle: GoogleFonts.plusJakartaSans(
          fontWeight: FontWeight.w700,
          fontSize: 18,
          letterSpacing: -0.5,
          color: foreground,
        ),
      ),
    );
  }
}
