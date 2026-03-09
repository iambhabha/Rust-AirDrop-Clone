import 'dart:ui';
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import 'package:liquid_glass_renderer/liquid_glass_renderer.dart';
import '../../src/rust/api/simple.dart';
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
    return DraggableScrollableSheet(
      initialChildSize: 0.9,
      minChildSize: 0.5,
      maxChildSize: 0.95,
      expand: false,
      snap: true,
      builder: (context, scrollController) {
        return Container(
          decoration: BoxDecoration(
            color: AppTheme.background,
            borderRadius: const BorderRadius.vertical(top: Radius.circular(24)),
            border: Border.all(color: AppTheme.border, width: 1),
          ),
          clipBehavior: Clip.antiAlias,
          child: Stack(
            children: [
              Column(
                children: [
                  const SizedBox(height: 12),
                  // Drag Handle
                  Container(
                    width: 40,
                    height: 4,
                    decoration: BoxDecoration(
                      color: AppTheme.mutedForeground.withOpacity(0.3),
                      borderRadius: BorderRadius.circular(2),
                    ),
                  ),
                  const SizedBox(height: 12),
                  Expanded(
                    child: ListView(
                      controller: scrollController,
                      physics: const BouncingScrollPhysics(),
                      padding: EdgeInsets.symmetric(horizontal: 16.w),
                      children: [
                        _buildProfile(),
                        buildSettingsGroup([
                          buildSettingsRow(
                            context: context,
                            icon: CupertinoIcons.person_fill,
                            color: AppTheme.mutedForeground,
                            title: 'Profile',
                            isNav: true,
                          ),
                          buildSettingsRow(
                            context: context,
                            icon: CupertinoIcons.desktopcomputer,
                            color: AppTheme.mutedForeground,
                            title: 'Devices',
                            trailingText: '${savedDevices.length} ',
                            isNav: true,
                          ),
                        ]),
                        const SizedBox(height: 20),
                        _buildMainSettings(context),
                        const SizedBox(height: 12),
                        _buildSocialGroup(context),
                        const SizedBox(height: 60),
                      ],
                    ),
                  ),
                ],
              ),
              Positioned(top: 10, right: 16, child: _doneBtx(context)),
            ],
          ),
        );
      },
    );
  }

  Theme _doneBtx(BuildContext context) {
    return Theme(
      data: Theme.of(context).copyWith(highlightColor: Colors.transparent),
      child: FloatingActionButton(
        backgroundColor: Colors.transparent,
        elevation: 0,
        highlightElevation: 0,
        disabledElevation: 0,
        materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
        focusColor: Colors.transparent,
        hoverColor: Colors.transparent,
        splashColor: Colors.transparent,
        onPressed: () => Navigator.of(context).pop(),
        child: LiquidGlassLayer(
          useBackdropGroup: true,
          child: LiquidStretch(
            child: LiquidGlass(
              shape: LiquidRoundedSuperellipse(borderRadius: 15),
              child: Padding(
                padding: const EdgeInsets.all(10.0).copyWith(top: 8, bottom: 8),
                child: Text(
                  'Done',
                  style: TextStyle(
                    color: AppTheme.foreground,
                    fontWeight: FontWeight.bold,
                    fontSize: 12,
                  ),
                ),
              ),
            ),
          ),
        ),
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

  Widget _buildMainSettings(BuildContext context) {
    return Column(
      children: [
        buildSettingsGroup([
          buildSettingsRow(
            context: context,
            icon: CupertinoIcons.bell_fill,
            color: Colors.blueAccent,
            title: 'Notifications',
            trailingText: 'On',
          ),
          buildSettingsRow(
            context: context,
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
            context: context,
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
            context: context,
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

  Widget _buildSocialGroup(BuildContext context) {
    return buildSettingsGroup([
      buildSettingsRow(
        context: context,
        icon: CupertinoIcons.envelope_fill,
        color: AppTheme.mutedForeground,
        title: 'Email Support',
        trailingText: 'Contact',
      ),
      buildSettingsRow(
        context: context,
        icon: CupertinoIcons.bubble_left_bubble_right_fill,
        color: AppTheme.mutedForeground,
        title: 'Discord',
        trailingText: 'Join',
      ),
      buildSettingsRow(
        context: context,
        icon: CupertinoIcons.xmark,
        color: AppTheme.foreground,
        title: 'X.com',
        trailingText: '@RustDrop',
      ),
    ]);
  }
}
