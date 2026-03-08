import 'package:cupertino_native/cupertino_native.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import '../theme.dart';

Widget buildSettingsGroup(List<Widget> rows) {
  return Container(
    clipBehavior: Clip.antiAlias,
    decoration: BoxDecoration(
      color: AppTheme.card.withOpacity(0.5),
      borderRadius: BorderRadius.circular(8),
      border: Border.all(color: AppTheme.border, width: 1),
    ),
    child: Column(
      children: rows
          .asMap()
          .entries
          .expand(
            (entry) => [
              entry.value,
              if (entry.key != rows.length - 1)
                Divider(color: AppTheme.border, height: 1, indent: 48),
            ],
          )
          .toList(),
    ),
  );
}

Widget buildSettingsRow({
  required IconData icon,
  required Color color,
  required String title,
  String? trailingText,
  Color? trailingColor,
  IconData? trailingIcon,
  bool isNav = false,
  bool isSwitch = false,
  bool switchValue = false,
  ValueChanged<bool>? onSwitchChanged,
  VoidCallback? onTap,
}) {
  return InkWell(
    onTap: isSwitch ? () => onSwitchChanged?.call(!switchValue) : onTap,
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(6),
            decoration: BoxDecoration(
              color: color.withOpacity(0.1),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: color.withOpacity(0.2), width: 1),
            ),
            child: Icon(icon, color: color, size: 16),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              title,
              style: const TextStyle(
                color: AppTheme.foreground,
                fontSize: 14,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
          if (trailingIcon != null) ...[
            Icon(
              trailingIcon,
              color: trailingColor ?? AppTheme.mutedForeground,
              size: 14,
            ),
            const SizedBox(width: 4),
          ],
          if (trailingText != null)
            Text(
              trailingText,
              style: TextStyle(
                color: trailingColor ?? AppTheme.mutedForeground,
                fontSize: 13,
              ),
            ),
          if (isNav) ...[
            const SizedBox(width: 4),
            Icon(
              CupertinoIcons.chevron_right,
              color: AppTheme.border,
              size: 14,
            ),
          ],
          if (isSwitch)
            CNSwitch(value: switchValue, onChanged: onSwitchChanged ?? (_) {}),
        ],
      ),
    ),
  );
}
