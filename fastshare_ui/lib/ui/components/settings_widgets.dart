import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

Widget buildSettingsGroup(List<Widget> rows) {
  return Container(
    clipBehavior: Clip.antiAlias,
    decoration: BoxDecoration(
      color: const Color(0xFF2C2C2E),
      borderRadius: BorderRadius.circular(20),
    ),
    child: Column(
      children: rows
          .asMap()
          .entries
          .expand(
            (entry) => [
              entry.value,
              if (entry.key != rows.length - 1)
                Divider(
                  color: Colors.white.withOpacity(0.05),
                  height: 1,
                  indent: 52,
                ),
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
          AnimatedContainer(
            duration: const Duration(milliseconds: 300),
            padding: const EdgeInsets.all(6),
            decoration: BoxDecoration(
              color: color,
              borderRadius: BorderRadius.circular(8),
              boxShadow: [
                BoxShadow(
                  color: color.withOpacity(0.3),
                  blurRadius: 8,
                  spreadRadius: 1,
                ),
              ],
            ),
            child: Icon(icon, color: Colors.white, size: 18),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              title,
              style: const TextStyle(
                color: Colors.white,
                fontSize: 16,
                fontWeight: FontWeight.w400,
              ),
            ),
          ),
          if (trailingIcon != null) ...[
            Icon(
              trailingIcon,
              color: trailingColor ?? Colors.white54,
              size: 16,
            ),
            const SizedBox(width: 4),
          ],
          if (trailingText != null)
            Text(
              trailingText,
              style: TextStyle(
                color: trailingColor ?? Colors.white54,
                fontSize: 15,
              ),
            ),
          if (isNav) ...[
            const SizedBox(width: 4),
            const Icon(Icons.chevron_right, color: Colors.white38, size: 20),
          ],
          if (isSwitch)
            Transform.scale(
              scale: 0.8,
              child: CupertinoSwitch(
                value: switchValue,
                onChanged: onSwitchChanged,
                activeColor: const Color(0xFF30D158),
              ),
            ),
        ],
      ),
    ),
  );
}
