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
        ...devices.map(
          (device) =>
              _DeviceIcon(device: device, onTap: () => onDeviceTap(device)),
        ),
        const _SpecialIcon(
          icon: Icons.devices,
          color: Colors.blueAccent,
          label: 'Set up a Device',
        ),
        const _SpecialIcon(
          icon: Icons.card_giftcard,
          color: Colors.blueAccent,
          label: 'Tell Someone\nAbout Blip',
        ),
      ],
    );
  }
}

class _DeviceIcon extends StatelessWidget {
  final DeviceInfo device;
  final VoidCallback onTap;

  const _DeviceIcon({required this.device, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 70.w,
            height: 70.w,
            decoration: BoxDecoration(
              color: device.isOnline
                  ? const Color(0xFF9000FF)
                  : const Color(0xFF333333),
              shape: BoxShape.circle,
            ),
            child: Center(
              child: device.isOnline
                  ? Icon(Icons.computer, color: Colors.white, size: 30.w)
                  : Text(
                      device.initial,
                      style: TextStyle(
                        color: Colors.white70,
                        fontSize: 24.sp,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
            ),
          ),
          SizedBox(height: 8.h),
          SizedBox(
            width: 70.w,
            child: Text(
              device.deviceName,
              textAlign: TextAlign.center,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                color: device.isOnline ? Colors.white : Colors.white54,
                fontSize: 12.sp,
              ),
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
          decoration: const BoxDecoration(
            color: Color(0xFF1E1E1E),
            shape: BoxShape.circle,
          ),
          child: Icon(icon, color: color, size: 30.w),
        ),
        SizedBox(height: 8.h),
        SizedBox(
          width: 70.w,
          child: Text(
            label,
            textAlign: TextAlign.center,
            style: TextStyle(color: Colors.white70, fontSize: 12.sp),
          ),
        ),
      ],
    );
  }
}
