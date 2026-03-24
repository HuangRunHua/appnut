import 'package:flutter/material.dart';

// TODO: 由你来实现时间线页面 UI
// 调用 fluxClient.emit("timeline/load") 加载推文列表

class TimelineView extends StatelessWidget {
  const TimelineView({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Home')),
      body: const Center(child: Text('Timeline View - TODO')),
    );
  }
}
