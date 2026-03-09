import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import 'package:flutter_svg/flutter_svg.dart';
import '../../models/transfer_progress.dart';
import '../../models/history_item.dart';
import '../../utils/extensions.dart';
import '../theme.dart';

class ReceivedStack extends StatefulWidget {
  final List<TransferProgress> activeIncoming;
  final List<HistoryItem> history;
  final TransferProgress? outgoingProgress;
  final Function(TransferProgress)? onProgressTap;
  final Function(HistoryItem)? onHistoryTap;

  const ReceivedStack({
    super.key,
    required this.activeIncoming,
    required this.history,
    this.outgoingProgress,
    this.onProgressTap,
    this.onHistoryTap,
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
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        AnimatedSwitcher(
          duration: const Duration(milliseconds: 600),
          child:
              (widget.activeIncoming.isNotEmpty ||
                  widget.outgoingProgress != null ||
                  widget.history.isNotEmpty)
              ? _buildActiveProgress()
              : _buildIdleStack(),
        ),
      ],
    );
  }

  Widget _buildActiveProgress() {
    final hasActive =
        widget.activeIncoming.isNotEmpty || widget.outgoingProgress != null;
    final hasHistory = widget.history.isNotEmpty;

    // Collect all items to show
    final List<Widget> items = [];

    if (hasActive) {
      items.addAll(
        widget.activeIncoming.map(
          (p) => _ReceivedDeck(
            title: p.fileName.fileName,
            subtitle:
                (p.status != null &&
                    (p.throughputBps == null || p.throughputBps == 0))
                ? p.status!
                : (p.throughputBps != null
                      ? p.throughputBps!.formatSpeed
                      : 'Receiving...'),
            itemCount: p.totalFiles ?? 1,
            isIncoming: true,
            icon: CupertinoIcons.globe,
            progress: p.progress,
            onTap: () => widget.onProgressTap?.call(p),
          ),
        ),
      );
    }

    if (widget.outgoingProgress != null) {
      final p = widget.outgoingProgress!;
      items.add(
        _ReceivedDeck(
          title: p.fileName.fileName,
          subtitle: p.throughputBps != null
              ? p.throughputBps!.formatSpeed
              : 'Sending...',
          itemCount: p.totalFiles ?? 1,
          isIncoming: false,
          icon: CupertinoIcons.globe,
          progress: p.progress,
          onTap: () => widget.onProgressTap?.call(p),
        ),
      );
    }

    if (!hasActive && hasHistory) {
      // Show most recent history item if no active transfers
      items.add(
        _ReceivedDeck(
          title: widget.history.first.fileName.fileName,
          subtitle: widget.history.first.isIncoming ? 'Received' : 'Sent',
          itemCount: widget.history.first.totalFiles,
          isIncoming: widget.history.first.isIncoming,
          progress: 1.0,
          onTap: () => widget.onHistoryTap?.call(widget.history.first),
        ),
      );
    }

    return Wrap(
      key: const ValueKey('active'),
      spacing: 16.w,
      runSpacing: 16.h,
      children: items
          .map(
            (item) => SizedBox(
              width:
                  (1.sw - 48.w) /
                  2, // Accounting for screen padding and wrap spacing
              child: item,
            ),
          )
          .toList(),
    );
  }

  Widget _buildIdleStack() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        const SizedBox(width: double.infinity),
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
            placeholderBuilder: (context) => SizedBox(
              width: 180.w,
              height: 180.w,
              child: const Center(
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

class _ReceivedDeck extends StatelessWidget {
  final String title;
  final String subtitle;
  final int itemCount;
  final bool isIncoming;
  final VoidCallback onTap;
  final IconData? icon;
  final double progress;

  const _ReceivedDeck({
    required this.title,
    required this.subtitle,
    required this.itemCount,
    required this.isIncoming,
    required this.onTap,
    required this.progress,
    this.icon,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: EdgeInsets.symmetric(vertical: 20.h),
        child: Column(
          children: [
            Stack(
              alignment: Alignment.bottomCenter,
              clipBehavior: Clip.none,
              children: [
                // Layer 3 (Backmost)
                Positioned(
                  bottom: 20.h,
                  child: Container(
                    width: 100.w,
                    height: 100.w,
                    decoration: BoxDecoration(
                      color: Colors.white.withOpacity(0.02),
                      borderRadius: BorderRadius.circular(20.r),
                      border: Border.all(color: Colors.white.withOpacity(0.05)),
                    ),
                  ),
                ),
                // Layer 2
                Positioned(
                  bottom: 10.h,
                  child: Container(
                    width: 115.w,
                    height: 115.w,
                    decoration: BoxDecoration(
                      color: Colors.white.withOpacity(0.04),
                      borderRadius: BorderRadius.circular(24.r),
                      border: Border.all(color: Colors.white.withOpacity(0.08)),
                    ),
                  ),
                ),
                // Main Layer (Front)
                Container(
                  width: 130.w,
                  height: 130.w,
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      begin: Alignment.topLeft,
                      end: Alignment.bottomRight,
                      colors: [
                        (isIncoming ? AppTheme.secondary : AppTheme.primary)
                            .withOpacity(0.12),
                        Colors.white.withOpacity(0.05),
                      ],
                    ),
                    borderRadius: BorderRadius.circular(28.r),
                    border: Border.all(
                      color:
                          (isIncoming ? AppTheme.secondary : AppTheme.primary)
                              .withOpacity(0.2),
                      width: 1.2,
                    ),
                    boxShadow: [
                      BoxShadow(
                        color:
                            (isIncoming ? AppTheme.secondary : AppTheme.primary)
                                .withOpacity(0.15),
                        blurRadius: 20,
                        offset: const Offset(0, 10),
                        spreadRadius: -5,
                      ),
                    ],
                  ),
                ),
                // Device Icon
                Positioned(
                  bottom: -15.h,
                  child: Stack(
                    alignment: Alignment.center,
                    children: [
                      // Glow Background
                      Container(
                        width: 48.w,
                        height: 48.w,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          boxShadow: [
                            BoxShadow(
                              color:
                                  (isIncoming
                                          ? AppTheme.secondary
                                          : AppTheme.primary)
                                      .withOpacity(0.4),
                              blurRadius: 12,
                              spreadRadius: 2,
                            ),
                          ],
                        ),
                      ),
                      // Progress Ring
                      SizedBox(
                        width: 46.w,
                        height: 46.w,
                        child: CircularProgressIndicator(
                          value: progress,
                          strokeWidth: 2.5,
                          backgroundColor: Colors.white.withOpacity(0.08),
                          valueColor: AlwaysStoppedAnimation<Color>(
                            isIncoming ? AppTheme.secondary : AppTheme.primary,
                          ),
                        ),
                      ),
                      // Icon Container
                      Container(
                        padding: EdgeInsets.all(8.w),
                        decoration: BoxDecoration(
                          gradient: isIncoming
                              ? AppTheme.secondaryGradient
                              : AppTheme.primaryGradient,
                          shape: BoxShape.circle,
                          border: Border.all(
                            color: AppTheme.background,
                            width: 3.w,
                          ),
                        ),
                        child: Icon(
                          icon ??
                              (isIncoming
                                  ? CupertinoIcons.desktopcomputer
                                  : CupertinoIcons.device_phone_portrait),
                          color: Colors.white,
                          size: 15.sp,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            SizedBox(height: 24.h),
            Text(
              itemCount == 1 ? title : '$itemCount Items',
              style: TextStyle(
                color: AppTheme.foreground,
                fontSize: 14.sp,
                fontWeight: FontWeight.bold,
                letterSpacing: -0.2,
              ),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            SizedBox(height: 2.h),
            Text(
              subtitle,
              style: TextStyle(
                color: AppTheme.mutedForeground,
                fontSize: 11.sp,
                fontWeight: FontWeight.w500,
              ),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ],
        ),
      ),
    );
  }
}
