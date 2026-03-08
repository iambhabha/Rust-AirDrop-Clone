import 'dart:ui';
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import '../../src/rust/api/simple.dart';
import '../../utils/extensions.dart';
import '../../models/device_info.dart';
import 'settings_widgets.dart';

class SettingsSheet extends StatelessWidget {
  final List<DeviceInfo> savedDevices;
  final bool checksumEnabled;
  final bool compressionEnabled;
  final Function(bool) onChecksumChanged;
  final Function(bool) onCompressionChanged;

  const SettingsSheet({
    super.key,
    required this.savedDevices,
    required this.checksumEnabled,
    required this.compressionEnabled,
    required this.onChecksumChanged,
    required this.onCompressionChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: ClipRRect(
        borderRadius: const BorderRadius.vertical(top: Radius.circular(24)),
        child: BackdropFilter(
          filter: ImageFilter.blur(sigmaX: 50, sigmaY: 50),
          child: Container(
            decoration: BoxDecoration(
              color: const Color(0xFF18181B).withOpacity(0.95),
              borderRadius: const BorderRadius.vertical(
                top: Radius.circular(24),
              ),
            ),
            height: context.height * 0.9,
            child: SafeArea(
              child: Column(
                children: [
                  _buildHeader(context),
                  _buildProfile(),
                  Expanded(
                    child: ListView(
                      padding: EdgeInsets.symmetric(horizontal: 16.w),
                      children: [
                        buildSettingsGroup([
                          buildSettingsRow(
                            icon: Icons.person,
                            color: Colors.grey,
                            title: 'Profile',
                            isNav: true,
                          ),
                          buildSettingsRow(
                            icon: Icons.computer,
                            color: Colors.grey,
                            title: 'Devices',
                            trailingText: '${savedDevices.length} ',
                            isNav: true,
                          ),
                        ]),
                        const SizedBox(height: 20),
                        _buildMainSettings(),
                        const SizedBox(height: 10),
                        _buildSocialGroup(),
                        const SizedBox(height: 30),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          Container(
            decoration: BoxDecoration(
              color: Colors.white.withOpacity(0.1),
              borderRadius: BorderRadius.circular(20),
            ),
            child: CupertinoButton(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              minSize: 0,
              onPressed: () => Navigator.of(context).pop(),
              child: const Text(
                'Done',
                style: TextStyle(
                  color: Colors.white,
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildProfile() {
    return Column(
      children: [
        const SizedBox(height: 10),
        Container(
          width: 80.w,
          height: 80.w,
          decoration: BoxDecoration(
            color: Colors.grey.withOpacity(0.5),
            shape: BoxShape.circle,
          ),
          child: Center(
            child: Text(
              'D',
              style: TextStyle(
                fontSize: 40.sp,
                fontWeight: FontWeight.bold,
                color: Colors.white,
              ),
            ),
          ),
        ),
        const SizedBox(height: 16),
        const Text(
          'dev',
          style: TextStyle(
            fontSize: 24,
            fontWeight: FontWeight.bold,
            color: Colors.white,
          ),
        ),
        const Text(
          'devrajheropanti@gmail.com',
          style: TextStyle(fontSize: 14, color: Colors.white54),
        ),
        const SizedBox(height: 24),
      ],
    );
  }

  Widget _buildMainSettings() {
    return Column(
      children: [
        buildSettingsGroup([
          buildSettingsRow(
            icon: Icons.notifications,
            color: Colors.redAccent,
            title: 'Push Notifications',
            trailingText: 'Enable',
            trailingColor: Colors.blueAccent,
          ),
          buildSettingsRow(
            icon: Icons.speed,
            color: Colors.pinkAccent,
            title: 'Fast Transfer (No Checksum)',
            isSwitch: true,
            switchValue: !checksumEnabled,
            onSwitchChanged: (v) {
              setChecksumEnabled(enabled: !v);
              onChecksumChanged(!v);
            },
          ),
          buildSettingsRow(
            icon: Icons.compress,
            color: Colors.purpleAccent,
            title: 'Data Compression',
            isSwitch: true,
            switchValue: compressionEnabled,
            onSwitchChanged: (v) {
              setCompressionEnabled(enabled: v);
              onCompressionChanged(v);
            },
          ),
          buildSettingsRow(
            icon: Icons.folder,
            color: Colors.blueAccent,
            title: 'Save Files to',
            trailingText: 'Rust Drop',
            trailingColor: Colors.blueAccent,
            trailingIcon: Icons.folder_open,
          ),
          buildSettingsRow(
            icon: Icons.image,
            color: Colors.blue,
            title: 'Add to Photos',
            isSwitch: true,
            switchValue: false,
          ),
        ]),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          child: Text(
            'Received files are saved to the Files app. When enabled, received photos and videos are also added to the Photos app.',
            style: TextStyle(color: Colors.white38, fontSize: 12.sp),
          ),
        ),
      ],
    );
  }

  Widget _buildSocialGroup() {
    return buildSettingsGroup([
      buildSettingsRow(
        icon: Icons.email,
        color: Colors.blue,
        title: 'Email',
        trailingText: 'hello@Rust Drop.io',
        trailingColor: Colors.blueAccent,
      ),
      buildSettingsRow(
        icon: Icons.chat_bubble,
        color: Colors.indigoAccent,
        title: 'Discord',
        trailingText: 'Join our community',
        trailingColor: Colors.blueAccent,
      ),
      buildSettingsRow(
        icon: Icons.close,
        color: Colors.black,
        title: 'X.com',
        trailingText: '@Rust Drop',
        trailingColor: Colors.blueAccent,
      ),
    ]);
  }
}
