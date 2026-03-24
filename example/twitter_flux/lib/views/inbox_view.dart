import 'package:flutter/material.dart';

// TODO: 由你来实现站内信页面 UI

class InboxView extends StatelessWidget {
  const InboxView({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Inbox')),
      body: const Center(child: Text('Inbox View - TODO')),
    );
  }
}
