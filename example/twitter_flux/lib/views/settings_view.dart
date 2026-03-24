import 'package:flutter/material.dart';

// TODO: 由你来实现设置页面 UI

class SettingsView extends StatelessWidget {
  const SettingsView({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: const Center(child: Text('Settings View - TODO')),
    );
  }
}
