import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import '../../models/transfer_progress.dart';

class ReceivedStack extends StatelessWidget {
  final List<TransferProgress> activeIncoming;
  final Map<String, dynamic>? outgoingProgress;

  const ReceivedStack({
    super.key,
    required this.activeIncoming,
    this.outgoingProgress,
  });

  @override
  Widget build(BuildContext context) {
    if (activeIncoming.isNotEmpty || outgoingProgress != null) {
      return Padding(
        padding: EdgeInsets.symmetric(horizontal: 16.w),
        child: const Text(
          'Transfer in progress...',
          style: TextStyle(color: Colors.white70),
        ),
      );
    }
    return Padding(
      padding: EdgeInsets.symmetric(horizontal: 16.w),
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          Positioned(
            top: 10,
            left: 10,
            right: 10,
            child: _buildStackChild(opacity: 0.05),
          ),
          Positioned(
            top: 5,
            left: 5,
            right: 5,
            child: _buildStackChild(opacity: 0.1),
          ),
          Container(
            height: 140.h,
            width: 140.w,
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: [
                  Colors.white.withOpacity(0.2),
                  Colors.white.withOpacity(0.05),
                ],
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
              ),
              borderRadius: BorderRadius.circular(24),
              border: Border.all(color: Colors.white.withOpacity(0.2)),
            ),
            child: Stack(
              clipBehavior: Clip.none,
              children: [
                Positioned(
                  bottom: -15.h,
                  left: 0,
                  right: 0,
                  child: Center(
                    child: Container(
                      padding: EdgeInsets.all(4.w),
                      decoration: const BoxDecoration(
                        color: Color(0xFF9000FF),
                        shape: BoxShape.circle,
                      ),
                      child: Icon(
                        Icons.computer,
                        color: Colors.white,
                        size: 24.w,
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
          Positioned(
            bottom: -50.h,
            left: 0,
            child: SizedBox(
              width: 140.w,
              child: Column(
                children: [
                  Text(
                    '11 Items',
                    style: TextStyle(
                      color: Colors.white,
                      fontWeight: FontWeight.bold,
                      fontSize: 14.sp,
                    ),
                  ),
                  Text(
                    'Received from DHSJB',
                    style: TextStyle(color: Colors.white54, fontSize: 10.sp),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStackChild({required double opacity}) {
    return Container(
      height: 140.h,
      width: double.infinity,
      decoration: BoxDecoration(
        color: Colors.white.withOpacity(opacity),
        borderRadius: BorderRadius.circular(24),
        border: Border.all(color: Colors.white.withOpacity(opacity + 0.05)),
      ),
    );
  }
}
