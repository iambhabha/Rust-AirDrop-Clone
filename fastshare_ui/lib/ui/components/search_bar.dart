import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import '../screens/qr_scanner_screen.dart';

class SearchBar extends StatefulWidget {
  final TextEditingController controller;
  final Function(String) onQrResult;

  const SearchBar({
    super.key,
    required this.controller,
    required this.onQrResult,
  });

  @override
  State<SearchBar> createState() => _SearchBarState();
}

class _SearchBarState extends State<SearchBar>
    with SingleTickerProviderStateMixin {
  late AnimationController _focusController;
  final FocusNode _focusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    _focusController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );
    _focusNode.addListener(() {
      if (_focusNode.hasFocus) {
        _focusController.forward();
      } else {
        _focusController.reverse();
      }
    });
  }

  @override
  void dispose() {
    _focusController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.symmetric(horizontal: 16.w, vertical: 8.h),
      child: AnimatedBuilder(
        animation: _focusController,
        builder: (context, child) {
          return Container(
            decoration: BoxDecoration(
              color: const Color(0xFF1E1E1E),
              borderRadius: BorderRadius.circular(16),
              border: Border.all(
                color: Color.lerp(
                  Colors.white.withOpacity(0.05),
                  const Color(0xFF9000FF).withOpacity(0.5),
                  _focusController.value,
                )!,
                width: 1.5,
              ),
              boxShadow: [
                if (_focusController.value > 0)
                  BoxShadow(
                    color: const Color(
                      0xFF9000FF,
                    ).withOpacity(0.1 * _focusController.value),
                    blurRadius: 10,
                    spreadRadius: 2,
                  ),
              ],
            ),
            child: child,
          );
        },
        child: TextField(
          controller: widget.controller,
          focusNode: _focusNode,
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
                if (ip != null) widget.onQrResult(ip);
              },
            ),
            border: InputBorder.none,
            contentPadding: const EdgeInsets.symmetric(vertical: 14),
          ),
          style: const TextStyle(color: Colors.white),
        ),
      ),
    );
  }
}
