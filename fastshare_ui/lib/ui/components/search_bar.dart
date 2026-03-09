import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import '../theme.dart';

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
      duration: const Duration(milliseconds: 200),
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
              color: AppTheme.card.withOpacity(0.5),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(
                color: Color.lerp(
                  AppTheme.border,
                  AppTheme.foreground.withOpacity(0.5),
                  _focusController.value,
                )!,
                width: 1,
              ),
              boxShadow: [
                if (_focusController.value > 0)
                  BoxShadow(
                    color: Colors.white.withOpacity(
                      0.05 * _focusController.value,
                    ),
                    blurRadius: 4,
                    spreadRadius: 1,
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
            hintStyle: TextStyle(
              color: AppTheme.mutedForeground,
              fontSize: 14.sp,
            ),
            prefixIcon: Icon(
              Icons.search,
              color: AppTheme.mutedForeground,
              size: 20.w,
            ),
            border: InputBorder.none,
            contentPadding: const EdgeInsets.symmetric(vertical: 14),
          ),
          style: const TextStyle(color: AppTheme.foreground),
        ),
      ),
    );
  }
}
