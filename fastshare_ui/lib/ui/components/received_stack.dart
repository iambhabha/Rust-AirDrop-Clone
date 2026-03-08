import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import 'package:flutter_svg/flutter_svg.dart';
import '../../models/transfer_progress.dart';
import '../theme.dart';

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
        color: AppTheme.card,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.border, width: 1),
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
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        SizedBox(width: double.infinity),
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
          child: SvgPicture.asset(
            'assets/images/empty_state.svg',
            width: 180.w,
            height: 180.w,
            placeholderBuilder: (context) => Container(
              width: 180.w,
              height: 180.w,
              child: Center(
                child: CircularProgressIndicator(
                  color: AppTheme.primary,
                  strokeWidth: 2,
                ),
              ),
            ),
          ),
        ),
        SizedBox(height: 12.h),
        Text(
          'No received files found',
          style: TextStyle(
            color: AppTheme.foreground,
            fontWeight: FontWeight.bold,
            fontSize: 16.sp,
          ),
        ),
        SizedBox(height: 4.h),
        Text(
          'Your transfer history will appear here',
          style: TextStyle(color: AppTheme.mutedForeground, fontSize: 12.sp),
        ),
      ],
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
                  color: AppTheme.primary.withOpacity(0.1),
                  shape: BoxShape.circle,
                ),
                child: Icon(
                  isIncoming
                      ? CupertinoIcons.arrow_down_circle
                      : CupertinoIcons.arrow_up_circle,
                  color: AppTheme.primary,
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
                        color: AppTheme.foreground,
                        fontSize: 13.sp,
                        fontWeight: FontWeight.w600,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                    Text(
                      _formatSpeed(progress.speed),
                      style: TextStyle(
                        color: AppTheme.mutedForeground,
                        fontSize: 11.sp,
                      ),
                    ),
                  ],
                ),
              ),
              Text(
                '${progress.progress.toInt()}%',
                style: TextStyle(
                  color: AppTheme.primary,
                  fontWeight: FontWeight.bold,
                  fontSize: 12.sp,
                ),
              ),
            ],
          ),
          SizedBox(height: 10.h),
          ClipRRect(
            borderRadius: BorderRadius.circular(4),
            child: LinearProgressIndicator(
              value: progress.progress / 100,
              backgroundColor: AppTheme.border,
              valueColor: const AlwaysStoppedAnimation(AppTheme.primary),
              minHeight: 4.h,
            ),
          ),
        ],
      ),
    );
  }

  String _formatSpeed(String? speed) {
    if (speed == null) return "0 B/s";
    if (speed.contains("B") || speed.contains("M") || speed.contains("K"))
      return speed;
    final numSpeed = double.tryParse(speed);
    if (numSpeed == null) return speed;
    if (numSpeed > 1024 * 1024)
      return "${(numSpeed / (1024 * 1024)).toStringAsFixed(1)} MB/s";
    if (numSpeed > 1024) return "${(numSpeed / 1024).toStringAsFixed(1)} KB/s";
    return "${numSpeed.toStringAsFixed(0)} B/s";
  }
}
