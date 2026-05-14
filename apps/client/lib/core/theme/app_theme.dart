import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

class AppTheme {
  static const _teal = Color(0xFF20B2AA);
  static const _citationBlue = Color(0xFF0066CC);

  static ThemeData get light {
    final base = ThemeData.light(useMaterial3: true);
    final textTheme = GoogleFonts.interTextTheme(base.textTheme).copyWith(
      headlineMedium: GoogleFonts.inter(fontSize: 24, fontWeight: FontWeight.w700, height: 1.4, color: const Color(0xFF1A1A1A)),
      titleLarge: GoogleFonts.inter(fontSize: 18, fontWeight: FontWeight.w600, height: 1.5, color: const Color(0xFF1A1A1A)),
      bodyLarge: GoogleFonts.inter(fontSize: 16, fontWeight: FontWeight.w400, height: 1.7, color: const Color(0xFF1A1A1A)),
      bodyMedium: GoogleFonts.inter(fontSize: 14, fontWeight: FontWeight.w400, height: 1.5, color: const Color(0xFF1A1A1A)),
      bodySmall: GoogleFonts.inter(fontSize: 12, fontWeight: FontWeight.w500, color: const Color(0xFF6B7280)),
      labelLarge: GoogleFonts.inter(fontSize: 14, fontWeight: FontWeight.w600, color: const Color(0xFF1A1A1A)),
    );

    return base.copyWith(
      colorScheme: const ColorScheme.light(
        primary: _teal,
        secondary: _citationBlue,
        surface: Color(0xFFFAF8F5),
        onSurface: Color(0xFF1A1A1A),
        surfaceContainerHighest: Color(0xFFF3F4F6),
        outline: Color(0xFFE5E7EB),
        error: Color(0xFFEF4444),
      ),
      scaffoldBackgroundColor: Colors.white,
      textTheme: textTheme,
      appBarTheme: AppBarTheme(
        elevation: 0,
        scrolledUnderElevation: 0,
        backgroundColor: Colors.white,
        foregroundColor: const Color(0xFF1A1A1A),
        titleTextStyle: GoogleFonts.inter(fontSize: 18, fontWeight: FontWeight.w600, color: const Color(0xFF1A1A1A)),
      ),
      cardTheme: CardThemeData(
        elevation: 0,
        color: Colors.white,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: const BorderSide(color: Color(0xFFE5E7EB)),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: const Color(0xFFF3F4F6),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0xFFE5E7EB)),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0xFFE5E7EB)),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: _teal, width: 2),
        ),
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          elevation: 0,
          backgroundColor: _teal,
          foregroundColor: Colors.white,
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: _teal,
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
          side: const BorderSide(color: _teal),
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          backgroundColor: _teal,
          foregroundColor: Colors.white,
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
      ),
      dividerTheme: const DividerThemeData(color: Color(0xFFE5E7EB), thickness: 1),
      navigationRailTheme: const NavigationRailThemeData(
        backgroundColor: Colors.white,
        indicatorColor: Color(0xFFE0F5F3),
      ),
    );
  }

  static ThemeData get dark {
    final base = ThemeData.dark(useMaterial3: true);
    final textTheme = GoogleFonts.interTextTheme(base.textTheme).copyWith(
      headlineMedium: GoogleFonts.inter(fontSize: 24, fontWeight: FontWeight.w700, height: 1.4, color: const Color(0xFFF9FAFB)),
      titleLarge: GoogleFonts.inter(fontSize: 18, fontWeight: FontWeight.w600, height: 1.5, color: const Color(0xFFF9FAFB)),
      bodyLarge: GoogleFonts.inter(fontSize: 16, fontWeight: FontWeight.w400, height: 1.7, color: const Color(0xFFF9FAFB)),
      bodyMedium: GoogleFonts.inter(fontSize: 14, fontWeight: FontWeight.w400, height: 1.5, color: const Color(0xFFF9FAFB)),
      bodySmall: GoogleFonts.inter(fontSize: 12, fontWeight: FontWeight.w500, color: const Color(0xFF9CA3AF)),
      labelLarge: GoogleFonts.inter(fontSize: 14, fontWeight: FontWeight.w600, color: const Color(0xFFF9FAFB)),
    );

    return base.copyWith(
      colorScheme: const ColorScheme.dark(
        primary: _teal,
        secondary: Color(0xFF00B4D8),
        surface: Color(0xFF1C1C1C),
        onSurface: Color(0xFFF9FAFB),
        surfaceContainerHighest: Color(0xFF333333),
        outline: Color(0xFF3A3A3A),
        error: Color(0xFFEF4444),
      ),
      scaffoldBackgroundColor: const Color(0xFF1C1C1C),
      textTheme: textTheme,
      appBarTheme: AppBarTheme(
        elevation: 0,
        scrolledUnderElevation: 0,
        backgroundColor: const Color(0xFF262626),
        titleTextStyle: GoogleFonts.inter(fontSize: 18, fontWeight: FontWeight.w600, color: const Color(0xFFF9FAFB)),
      ),
      cardTheme: CardThemeData(
        elevation: 0,
        color: const Color(0xFF262626),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: const BorderSide(color: Color(0xFF3A3A3A)),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: const Color(0xFF262626),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0xFF3A3A3A)),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0xFF3A3A3A)),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: _teal, width: 2),
        ),
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      ),
      navigationRailTheme: const NavigationRailThemeData(
        backgroundColor: Color(0xFF1C1C1C),
      ),
      dividerTheme: const DividerThemeData(color: Color(0xFF3A3A3A), thickness: 1),
    );
  }
}
