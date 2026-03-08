import 'dart:ui';
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import '../../src/rust/api/simple.dart';
import '../../utils/extensions.dart';
import '../../models/device_info.dart';
import '../theme.dart';
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
      child: Container(
        decoration: BoxDecoration(
          color: AppTheme.background,
          borderRadius: const BorderRadius.vertical(top: Radius.circular(16)),
          border: Border.all(color: AppTheme.border, width: 1),
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
                        icon: CupertinoIcons.person_fill,
                        color: AppTheme.mutedForeground,
                        title: 'Profile',
                        isNav: true,
                      ),
                      buildSettingsRow(
                        icon: CupertinoIcons.desktopcomputer,
                        color: AppTheme.mutedForeground,
                        title: 'Devices',
                        trailingText: '${savedDevices.length} ',
                        isNav: true,
                      ),
                    ]),
                    const SizedBox(height: 20),
                    _buildMainSettings(),
                    const SizedBox(height: 12),
                    _buildSocialGroup(),
                    const SizedBox(height: 30),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          const Text(
            'Settings',
            style: TextStyle(
              color: AppTheme.foreground,
              fontWeight: FontWeight.bold,
              fontSize: 16,
            ),
          ),
          IconButton(
            onPressed: () => Navigator.of(context).pop(),
            icon: const Icon(
              Icons.close,
              color: AppTheme.mutedForeground,
              size: 20,
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
          width: 70.w,
          height: 70.w,
          decoration: BoxDecoration(
            color: AppTheme.card,
            shape: BoxShape.circle,
            border: Border.all(color: AppTheme.border),
          ),
          child: Center(
            child: Text(
              'D',
              style: TextStyle(
                fontSize: 32.sp,
                fontWeight: FontWeight.bold,
                color: AppTheme.foreground,
              ),
            ),
          ),
        ),
        const SizedBox(height: 16),
        const Text(
          'dev',
          style: TextStyle(
            fontSize: 20,
            fontWeight: FontWeight.bold,
            color: AppTheme.foreground,
          ),
        ),
        const Text(
          'devrajheropanti@gmail.com',
          style: TextStyle(fontSize: 12, color: AppTheme.mutedForeground),
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
            icon: CupertinoIcons.bell_fill,
            color: Colors.blueAccent,
            title: 'Notifications',
            trailingText: 'On',
          ),
          buildSettingsRow(
            icon: CupertinoIcons.speedometer,
            color: AppTheme.primary,
            title: 'Fast Transfer',
            isSwitch: true,
            switchValue: !checksumEnabled,
            onSwitchChanged: (v) {
              setChecksumEnabled(enabled: !v);
              onChecksumChanged(!v);
            },
          ),
          buildSettingsRow(
            icon: CupertinoIcons.archivebox_fill,
            color: Colors.tealAccent,
            title: 'Compression',
            isSwitch: true,
            switchValue: compressionEnabled,
            onSwitchChanged: (v) {
              setCompressionEnabled(enabled: v);
              onCompressionChanged(v);
            },
          ),
          buildSettingsRow(
            icon: CupertinoIcons.folder_fill,
            color: Colors.amberAccent,
            title: 'Download Path',
            trailingText: 'Rust Drop',
            isNav: true,
          ),
        ]),
        Padding(
          padding: const EdgeInsets.all(12),
          child: Text(
            'High performance transfer mode skips checksum verification for maximum speed on trusted local networks.',
            style: TextStyle(color: AppTheme.mutedForeground, fontSize: 11.sp),
          ),
        ),
      ],
    );
  }

  Widget _buildSocialGroup() {
    return buildSettingsGroup([
      buildSettingsRow(
        icon: CupertinoIcons.envelope_fill,
        color: AppTheme.mutedForeground,
        title: 'Email Support',
        trailingText: 'Contact',
      ),
      buildSettingsRow(
        icon: CupertinoIcons.bubble_left_bubble_right_fill,
        color: AppTheme.mutedForeground,
        title: 'Discord',
        trailingText: 'Join',
      ),
      buildSettingsRow(
        icon: CupertinoIcons.xmark,
        color: AppTheme.foreground,
        title: 'X.com',
        trailingText: '@RustDrop',
      ),
    ]);
  }
}
