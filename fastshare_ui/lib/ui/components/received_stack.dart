import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import '../../models/transfer_progress.dart';

class ReceivedStack extends StatefulWidget {
  final List<TransferProgress> activeIncoming;
  final TransferProgress? outgoingProgress;

  const ReceivedStack({
    super.key,
    required this.activeIncoming,
    this.outgoingProgress,
  });

  @override
  State<ReceivedStack> createState() => _ReceivedStackState();
}

class _ReceivedStackState extends State<ReceivedStack>
    with SingleTickerProviderStateMixin {
  late AnimationController _floatController;

  @override
  void initState() {
    super.initState();
    _floatController = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 3),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _floatController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final bool hasActive =
        widget.activeIncoming.isNotEmpty || widget.outgoingProgress != null;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        AnimatedSwitcher(
          duration: const Duration(milliseconds: 600),
          child: hasActive ? _buildActiveProgress() : _buildIdleStack(),
        ),
      ],
    );
  }

  Widget _buildActiveProgress() {
    return Container(
      key: const ValueKey('active'),
      padding: EdgeInsets.all(20.w),
      width: double.infinity,
      decoration: BoxDecoration(
        color: const Color(0xFF1E1E1E),
        borderRadius: BorderRadius.circular(24),
        border: Border.all(color: Colors.white.withOpacity(0.05), width: 1.5),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (widget.activeIncoming.isNotEmpty) ...[
            ...widget.activeIncoming.map(
              (p) => _TransferItem(progress: p, isIncoming: true),
            ),
          ],
          if (widget.outgoingProgress != null) ...[
            _TransferItem(
              progress: widget.outgoingProgress!,
              isIncoming: false,
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildIdleStack() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        AnimatedBuilder(
          key: const ValueKey('idle'),
          animation: _floatController,
          builder: (context, child) {
            return Transform.translate(
              offset: Offset(
                0,
                -10 * Curves.easeInOut.transform(_floatController.value),
              ),
              child: child,
            );
          },
          child: Container(
            width: 140.w,
            height: 140.h,
            margin: EdgeInsets.only(left: 8.w),
            child: Stack(
              clipBehavior: Clip.none,
              children: [
                Positioned(
                  top: -15,
                  left: 15,
                  right: -15,
                  child: _buildStackChild(opacity: 0.03),
                ),
                Positioned(
                  top: -8,
                  left: 8,
                  right: -8,
                  child: _buildStackChild(opacity: 0.06),
                ),
                Container(
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      colors: [
                        Colors.white.withOpacity(0.2),
                        Colors.white.withOpacity(0.08),
                      ],
                      begin: Alignment.topLeft,
                      end: Alignment.bottomRight,
                    ),
                    borderRadius: BorderRadius.circular(28),
                    border: Border.all(
                      color: Colors.white.withOpacity(0.2),
                      width: 1.5,
                    ),
                  ),
                  child: Center(
                    child: Container(
                      padding: EdgeInsets.all(12.w),
                      decoration: const BoxDecoration(
                        color: Color(0xFF9000FF),
                        shape: BoxShape.circle,
                      ),
                      child: Icon(
                        Icons.computer,
                        color: Colors.white,
                        size: 28.w,
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
        SizedBox(height: 16.h),
        Padding(
          padding: EdgeInsets.only(left: 8.w),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                '0 Items',
                style: TextStyle(
                  color: Colors.white,
                  fontWeight: FontWeight.bold,
                  fontSize: 16.sp,
                ),
              ),
              Text(
                'No recent transfers',
                style: TextStyle(color: Colors.white54, fontSize: 12.sp),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildStackChild({required double opacity}) {
    return Container(
      height: 140.h,
      decoration: BoxDecoration(
        color: Colors.white.withOpacity(opacity),
        borderRadius: BorderRadius.circular(28),
        border: Border.all(color: Colors.white.withOpacity(opacity + 0.02)),
      ),
    );
  }
}

class _TransferItem extends StatelessWidget {
  final TransferProgress progress;
  final bool isIncoming;

  const _TransferItem({required this.progress, required this.isIncoming});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.only(bottom: 12.h),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                padding: const EdgeInsets.all(6),
                decoration: BoxDecoration(
                  color: const Color(0xFF9000FF).withOpacity(0.1),
                  shape: BoxShape.circle,
                ),
                child: Icon(
                  isIncoming ? Icons.download : Icons.upload,
                  color: const Color(0xFF9000FF),
                  size: 14.sp,
                ),
              ),
              SizedBox(width: 10.w),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      progress.fileName,
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 13.sp,
                        fontWeight: FontWeight.w600,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                    Text(
                      _formatSpeed(progress.speed),
                      style: TextStyle(color: Colors.white54, fontSize: 11.sp),
                    ),
                  ],
                ),
              ),
              Text(
                '${progress.progress.toInt()}%',
                style: TextStyle(
                  color: const Color(0xFF9000FF),
                  fontWeight: FontWeight.bold,
                  fontSize: 12.sp,
                ),
              ),
            ],
          ),
          SizedBox(height: 10.h),
          ClipRRect(
            borderRadius: BorderRadius.circular(10),
            child: LinearProgressIndicator(
              value: progress.progress / 100,
              backgroundColor: Colors.white.withOpacity(0.05),
              valueColor: const AlwaysStoppedAnimation(Color(0xFF9000FF)),
              minHeight: 6.h,
            ),
          ),
        ],
      ),
    );
  }

  String _formatSpeed(String? speed) {
    if (speed == null) return "0 B/s";
    // If it's already a formatted string from engine
    if (speed.contains("B") || speed.contains("M") || speed.contains("K"))
      return speed;

    // Otherwise try to parse as number
    final numSpeed = double.tryParse(speed);
    if (numSpeed == null) return speed;

    if (numSpeed > 1024 * 1024) {
      return "${(numSpeed / (1024 * 1024)).toStringAsFixed(1)} MB/s";
    } else if (numSpeed > 1024) {
      return "${(numSpeed / 1024).toStringAsFixed(1)} KB/s";
    } else {
      return "${numSpeed.toStringAsFixed(0)} B/s";
    }
  }
}
