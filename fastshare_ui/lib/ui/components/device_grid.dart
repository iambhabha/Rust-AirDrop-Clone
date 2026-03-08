import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import '../../models/device_info.dart';

class DeviceGrid extends StatelessWidget {
  final List<DeviceInfo> devices;
  final Function(DeviceInfo) onDeviceTap;

  const DeviceGrid({
    super.key,
    required this.devices,
    required this.onDeviceTap,
  });

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 20.w,
      runSpacing: 20.h,
      alignment: WrapAlignment.start,
      children: [
        ...devices.asMap().entries.map(
          (entry) => TweenAnimationBuilder<double>(
            duration: Duration(milliseconds: 400 + (entry.key * 100)),
            tween: Tween(begin: 0.0, end: 1.0),
            curve: Curves.easeOutBack,
            builder: (context, value, child) {
              return Transform.scale(
                scale: value,
                child: Opacity(opacity: value.clamp(0.0, 1.0), child: child),
              );
            },
            child: _DeviceIcon(
              device: entry.value,
              onTap: () => onDeviceTap(entry.value),
            ),
          ),
        ),
        _buildSpecialIcon(
          icon: Icons.devices,
          color: Colors.white60,
          label: 'Set up a Device',
          delay: devices.length,
        ),
        _buildSpecialIcon(
          icon: Icons.card_giftcard,
          color: Colors.white60,
          label: 'Tell Someone\nAbout Rust Drop',
          delay: devices.length + 1,
        ),
      ],
    );
  }

  Widget _buildSpecialIcon({
    required IconData icon,
    required Color color,
    required String label,
    required int delay,
  }) {
    return TweenAnimationBuilder<double>(
      duration: Duration(milliseconds: 400 + (delay * 100)),
      tween: Tween(begin: 0.0, end: 1.0),
      curve: Curves.easeOutBack,
      builder: (context, value, child) {
        return Transform.scale(
          scale: value,
          child: Opacity(opacity: value.clamp(0.0, 1.0), child: child),
        );
      },
      child: _SpecialIcon(icon: icon, color: color, label: label),
    );
  }
}

class _DeviceIcon extends StatefulWidget {
  final DeviceInfo device;
  final VoidCallback onTap;

  const _DeviceIcon({required this.device, required this.onTap});

  @override
  State<_DeviceIcon> createState() => _DeviceIconState();
}

class _DeviceIconState extends State<_DeviceIcon>
    with SingleTickerProviderStateMixin {
  late AnimationController _pulseController;

  @override
  void initState() {
    super.initState();
    _pulseController = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 2),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _pulseController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: widget.onTap,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Stack(
            alignment: Alignment.center,
            clipBehavior: Clip.none,
            children: [
              // Online Glow/Pulse
              if (widget.device.isOnline)
                AnimatedBuilder(
                  animation: _pulseController,
                  builder: (context, child) {
                    return Container(
                      width: 70.w + (12 * _pulseController.value),
                      height: 70.w + (12 * _pulseController.value),
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        color: const Color(
                          0xFF9000FF,
                        ).withOpacity(0.2 * (1 - _pulseController.value)),
                      ),
                    );
                  },
                ),
              // Main Icon Container
              AnimatedContainer(
                duration: const Duration(milliseconds: 500),
                width: 70.w,
                height: 70.w,
                decoration: BoxDecoration(
                  color: widget.device.isOnline
                      ? const Color(0xFF9000FF)
                      : const Color(0xFF1E1E1E),
                  shape: BoxShape.circle,
                  border: Border.all(
                    color: widget.device.isOnline
                        ? Colors.white.withOpacity(0.2)
                        : Colors.white.withOpacity(0.05),
                    width: 1.5,
                  ),
                  boxShadow: widget.device.isOnline
                      ? [
                          BoxShadow(
                            color: const Color(0xFF9000FF).withOpacity(0.5),
                            blurRadius: 20,
                            spreadRadius: 2,
                          ),
                        ]
                      : [],
                ),
                child: Center(
                  child: widget.device.isOnline
                      ? Icon(Icons.computer, color: Colors.white, size: 30.w)
                      : Text(
                          widget.device.initial,
                          style: TextStyle(
                            color: Colors.white70,
                            fontSize: 24.sp,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                ),
              ),
              // Nearby Green Dot Indicator
              if (widget.device.isOnline)
                Positioned(
                  top: 2.w,
                  right: 2.w,
                  child: Container(
                    width: 14.w,
                    height: 14.w,
                    decoration: BoxDecoration(
                      color: const Color(0xFF30D158), // iOS Green
                      shape: BoxShape.circle,
                      border: Border.all(color: Colors.black, width: 2),
                      boxShadow: [
                        BoxShadow(
                          color: const Color(0xFF30D158).withOpacity(0.6),
                          blurRadius: 6,
                          spreadRadius: 1,
                        ),
                      ],
                    ),
                  ),
                ),
            ],
          ),
          SizedBox(height: 8.h),
          SizedBox(
            width: 70.w,
            child: AnimatedDefaultTextStyle(
              duration: const Duration(milliseconds: 300),
              textAlign: TextAlign.center,
              style: TextStyle(
                color: widget.device.isOnline ? Colors.white : Colors.white54,
                fontSize: 11.sp,
                fontWeight: widget.device.isOnline
                    ? FontWeight.w600
                    : FontWeight.normal,
              ),
              child: Text(
                widget.device.deviceName,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ),
          if (widget.device.isOnline)
            Text(
              'Nearby',
              style: TextStyle(
                color: const Color(0xFF30D158),
                fontSize: 9.sp,
                fontWeight: FontWeight.bold,
              ),
            ),
        ],
      ),
    );
  }
}

class _SpecialIcon extends StatelessWidget {
  final IconData icon;
  final Color color;
  final String label;

  const _SpecialIcon({
    required this.icon,
    required this.color,
    required this.label,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 70.w,
          height: 70.w,
          decoration: BoxDecoration(
            color: const Color(0xFF1E1E1E),
            shape: BoxShape.circle,
            border: Border.all(
              color: Colors.white.withOpacity(0.05),
              width: 1.5,
            ),
          ),
          child: Icon(icon, color: color, size: 28.w),
        ),
        SizedBox(height: 8.h),
        SizedBox(
          width: 70.w,
          child: Text(
            label,
            textAlign: TextAlign.center,
            style: TextStyle(color: Colors.white54, fontSize: 11.sp),
          ),
        ),
        // Placeholder for consistency with online devices
        Opacity(
          opacity: 0,
          child: Text('Nearby', style: TextStyle(fontSize: 9.sp)),
        ),
      ],
    );
  }
}
