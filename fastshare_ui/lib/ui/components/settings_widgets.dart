import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

Widget buildSettingsGroup(List<Widget> rows) {
  return Container(
    decoration: BoxDecoration(
      color: const Color(0xFF2C2C2E),
      borderRadius: BorderRadius.circular(20),
    ),
    child: Column(
      children: rows
          .expand(
            (widget) => [
              widget,
              if (widget != rows.last)
                Divider(
                  color: Colors.white.withOpacity(0.05),
                  height: 1,
                  indent: 50,
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
}) {
  return Padding(
    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
    child: Row(
      children: [
        Container(
          padding: const EdgeInsets.all(6),
          decoration: BoxDecoration(
            color: color,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(icon, color: Colors.white, size: 18),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: Text(
            title,
            style: const TextStyle(color: Colors.white, fontSize: 16),
          ),
        ),
        if (trailingIcon != null) ...[
          Icon(trailingIcon, color: trailingColor ?? Colors.white54, size: 16),
          const SizedBox(width: 4),
        ],
        if (trailingText != null)
          Text(
            trailingText,
            style: TextStyle(
              color: trailingColor ?? Colors.white54,
              fontSize: 16,
            ),
          ),
        if (isNav) const Icon(Icons.chevron_right, color: Colors.white38),
        if (isSwitch)
          CupertinoSwitch(
            value: switchValue,
            onChanged: onSwitchChanged,
            activeColor: Colors.greenAccent,
          ),
      ],
    ),
  );
}
