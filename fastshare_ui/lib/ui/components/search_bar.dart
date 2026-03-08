import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import '../screens/qr_scanner_screen.dart';

class SearchBar extends StatelessWidget {
  final TextEditingController controller;
  final Function(String) onQrResult;

  const SearchBar({
    super.key,
    required this.controller,
    required this.onQrResult,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.symmetric(horizontal: 16.w, vertical: 8.h),
      child: Container(
        decoration: BoxDecoration(
          color: const Color(0xFF1E1E1E),
          borderRadius: BorderRadius.circular(10),
        ),
        child: TextField(
          controller: controller,
          decoration: InputDecoration(
            hintText: 'Name or Email',
            hintStyle: const TextStyle(color: Colors.white38),
            prefixIcon: const Icon(Icons.search, color: Colors.white38),
            suffixIcon: IconButton(
              icon: const Icon(Icons.qr_code_scanner, color: Colors.white38),
              onPressed: () async {
                final ip = await Navigator.push<String>(
                  context,
                  MaterialPageRoute(builder: (_) => const QrScannerScreen()),
                );
                if (ip != null) onQrResult(ip);
              },
            ),
            border: InputBorder.none,
            contentPadding: const EdgeInsets.symmetric(vertical: 12),
          ),
          style: const TextStyle(color: Colors.white),
        ),
      ),
    );
  }
}
