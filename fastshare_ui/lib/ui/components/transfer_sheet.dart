import 'dart:ui';
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import '../theme.dart';
import '../../models/transfer_progress.dart';
import '../../utils/extensions.dart';
import 'package:open_filex/open_filex.dart';

class TransferSheet extends StatelessWidget {
  final PendingIncoming? pending;
  final TransferProgress? progress;
  final VoidCallback onAccept;
  final VoidCallback onDecline;
  final VoidCallback onCancel;
  final VoidCallback? onPause;
  final VoidCallback? onOpen;

  const TransferSheet({
    super.key,
    this.pending,
    this.progress,
    required this.onAccept,
    required this.onDecline,
    required this.onCancel,
    this.onPause,
    this.onOpen,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: AppTheme.background.withOpacity(0.8),
        borderRadius: const BorderRadius.vertical(top: Radius.circular(32)),
        border: Border.all(color: AppTheme.border.withOpacity(0.5), width: 1),
      ),
      clipBehavior: Clip.antiAlias,
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 30, sigmaY: 30),
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 24.w, vertical: 12.h),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Drag Handle
              Container(
                width: 36,
                height: 4,
                decoration: BoxDecoration(
                  color: AppTheme.mutedForeground.withOpacity(0.3),
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
              const SizedBox(height: 32),

              // Icon Section
              _buildIconSection(),

              const SizedBox(height: 24),

              // Text Section
              _buildTextSection(),

              const SizedBox(height: 32),

              // Action Section
              if (pending != null)
                _buildRequestActions()
              else if (progress != null)
                (progress!.progress >= 1.0 ||
                        progress!.status?.toLowerCase() == "received" ||
                        progress!.status?.toLowerCase() == "completed")
                    ? _buildCompletedActions(context)
                    : _buildProgressActions(),

              const SizedBox(height: 24),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildIconSection() {
    return Stack(
      alignment: Alignment.center,
      children: [
        // File Icon Backdrop
        Container(
          width: 140.w,
          height: 140.w,
          decoration: BoxDecoration(
            color: Colors.white.withOpacity(0.04),
            borderRadius: BorderRadius.circular(28),
          ),
        ),
        // File Icon
        Icon(_getFileIcon(), size: 100.w, color: Colors.white.withOpacity(0.9)),
        // Device Icon Overlay
        Positioned(
          bottom: 0,
          right: 0,
          child: Container(
            width: 48.w,
            height: 48.w,
            decoration: BoxDecoration(
              color: AppTheme.primary,
              shape: BoxShape.circle,
              border: Border.all(color: const Color(0xFF141416), width: 4),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.4),
                  blurRadius: 12,
                  offset: const Offset(0, 4),
                ),
              ],
            ),
            child: const Icon(
              CupertinoIcons.desktopcomputer,
              size: 22,
              color: Colors.white,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildTextSection() {
    final fileName = pending?.fileName ?? progress?.fileName ?? "Unknown File";
    final from = pending?.fromAddr ?? progress?.fromAddr ?? "Someone";
    final status = progress != null
        ? "Receiving from $from"
        : "From your $from";

    return Column(
      children: [
        Text(
          fileName,
          textAlign: TextAlign.center,
          maxLines: 2,
          overflow: TextOverflow.ellipsis,
          style: TextStyle(
            fontSize: 18.sp,
            fontWeight: FontWeight.w600,
            color: AppTheme.foreground,
            letterSpacing: -0.5,
          ),
        ),
        const SizedBox(height: 8),
        Text(
          status,
          style: TextStyle(fontSize: 14.sp, color: AppTheme.mutedForeground),
        ),
      ],
    );
  }

  Widget _buildRequestActions() {
    return Row(
      children: [
        Expanded(
          child: _buildButton(
            text: "Decline",
            onPressed: onDecline,
            isPrimary: false,
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: _buildButton(
            text: "Accept",
            onPressed: onAccept,
            isPrimary: true,
          ),
        ),
      ],
    );
  }

  Widget _buildCompletedActions(BuildContext context) {
    return Column(
      children: [
        Row(
          children: [
            Expanded(
              child: _buildButton(
                text: "Open File",
                onPressed: () {
                  if (onOpen != null) {
                    onOpen!();
                  } else if (progress?.savedPath != null) {
                    OpenFilex.open(progress!.savedPath!);
                  } else {
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(content: Text("File path not found")),
                    );
                  }
                },
                isPrimary: true,
              ),
            ),
          ],
        ),
        SizedBox(height: 12.h),
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(
            "Dismiss",
            style: TextStyle(
              color: AppTheme.mutedForeground,
              fontSize: 14.sp,
              fontWeight: FontWeight.w500,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildProgressActions() {
    final totalBytes = (progress?.totalBytes ?? 0).formatSize;
    final percentage = ((progress?.progress ?? 0) * 100).toInt();
    final speed = (progress?.throughputBps ?? 0).formatSpeed;

    return Container(
      height: 94.h,
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: Colors.white.withOpacity(0.03),
        borderRadius: BorderRadius.circular(28),
        border: Border.all(color: Colors.white.withOpacity(0.08), width: 1),
      ),
      child: Stack(
        children: [
          // Background Progress
          FractionallySizedBox(
            widthFactor: progress?.progress ?? 0,
            child: ShimmerProgress(
              child: Container(
                decoration: BoxDecoration(
                  gradient: LinearGradient(
                    colors: [
                      AppTheme.primary.withOpacity(0.05),
                      AppTheme.primary.withOpacity(0.4),
                    ],
                    begin: Alignment.centerLeft,
                    end: Alignment.centerRight,
                  ),
                  border: const Border(
                    right: BorderSide(color: AppTheme.primary, width: 0.5),
                  ),
                ),
              ),
            ),
          ),
          // Content
          Padding(
            padding: EdgeInsets.symmetric(horizontal: 24.w),
            child: Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Text(
                        "$percentage% of $totalBytes",
                        style: TextStyle(
                          fontSize: 16.sp,
                          fontWeight: FontWeight.w600,
                          color: Colors.white.withOpacity(0.9),
                          letterSpacing: -0.2,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Row(
                        children: [
                          Icon(
                            CupertinoIcons.globe,
                            size: 14.sp,
                            color: Colors.white.withOpacity(0.5),
                          ),
                          const SizedBox(width: 8),
                          Text(
                            speed,
                            style: TextStyle(
                              fontSize: 13.sp,
                              color: Colors.white.withOpacity(0.5),
                              fontWeight: FontWeight.w500,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),

                // Buttons
                Row(
                  children: [
                    // Pause Button
                    GestureDetector(
                      onTap: onPause,
                      child: Container(
                        width: 44.w,
                        height: 44.h,
                        decoration: BoxDecoration(
                          color: Colors.white.withOpacity(0.05),
                          shape: BoxShape.circle,
                          border: Border.all(
                            color: Colors.white.withOpacity(0.1),
                            width: 1,
                          ),
                        ),
                        child: Icon(
                          (progress?.isPaused ?? false)
                              ? CupertinoIcons.play_fill
                              : CupertinoIcons.pause_fill,
                          size: 18.sp,
                          color: Colors.white.withOpacity(0.9),
                        ),
                      ),
                    ),
                    SizedBox(width: 12.w),
                    // Cancel Button
                    GestureDetector(
                      onTap: onCancel,
                      child: Container(
                        width: 44.w,
                        height: 44.w,
                        decoration: BoxDecoration(
                          color: const Color(0xFFFF453A).withOpacity(0.1),
                          shape: BoxShape.circle,
                          border: Border.all(
                            color: const Color(0xFFFF453A).withOpacity(0.2),
                            width: 1,
                          ),
                        ),
                        child: const Icon(
                          CupertinoIcons.xmark,
                          size: 18,
                          color: Color(0xFFFF453A),
                        ),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildButton({
    required String text,
    required VoidCallback onPressed,
    required bool isPrimary,
  }) {
    return CupertinoButton(
      padding: EdgeInsets.zero,
      onPressed: onPressed,
      child: Container(
        height: 54,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: isPrimary ? null : const Color(0xFF2C2C2E),
          gradient: isPrimary ? AppTheme.primaryGradient : null,
          borderRadius: BorderRadius.circular(16),
          boxShadow: isPrimary
              ? [
                  BoxShadow(
                    color: AppTheme.primary.withOpacity(0.3),
                    blurRadius: 12,
                    offset: const Offset(0, 4),
                  ),
                ]
              : null,
        ),
        child: Text(
          text,
          style: TextStyle(
            color: Colors.white,
            fontSize: 16.sp,
            fontWeight: FontWeight.w600,
          ),
        ),
      ),
    );
  }

  IconData _getFileIcon() {
    final fileName = (pending?.fileName ?? progress?.fileName ?? "")
        .toLowerCase();
    if (fileName.endsWith('.zip') ||
        fileName.endsWith('.rar') ||
        fileName.endsWith('.7z')) {
      return CupertinoIcons.archivebox_fill;
    } else if (fileName.endsWith('.mp4') ||
        fileName.endsWith('.mkv') ||
        fileName.endsWith('.mov')) {
      return CupertinoIcons.video_camera_solid;
    } else if (fileName.endsWith('.jpg') ||
        fileName.endsWith('.png') ||
        fileName.endsWith('.webp')) {
      return CupertinoIcons.photo;
    } else if (fileName.endsWith('.pdf') ||
        fileName.endsWith('.doc') ||
        fileName.endsWith('.txt')) {
      return CupertinoIcons.doc_fill;
    }
    return CupertinoIcons.doc_fill;
  }
}

class ShimmerProgress extends StatefulWidget {
  final Widget child;
  const ShimmerProgress({super.key, required this.child});

  @override
  State<ShimmerProgress> createState() => _ShimmerProgressState();
}

class _ShimmerProgressState extends State<ShimmerProgress>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1500),
    )..repeat();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return ShaderMask(
          blendMode: BlendMode.srcATop,
          shaderCallback: (bounds) {
            return LinearGradient(
              begin: const Alignment(-1.0, -0.3),
              end: const Alignment(1.0, 0.3),
              colors: [
                Colors.white.withOpacity(0),
                const Color(0xFFFFB800).withOpacity(0.15),
                Colors.white.withOpacity(0.6),
                const Color(0xFFFFB800).withOpacity(0.15),
                Colors.white.withOpacity(0),
              ],
              stops: [
                _controller.value - 0.25,
                _controller.value - 0.08,
                _controller.value,
                _controller.value + 0.08,
                _controller.value + 0.25,
              ],
            ).createShader(bounds);
          },
          child: widget.child,
        );
      },
    );
  }
}
