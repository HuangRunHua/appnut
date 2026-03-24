import 'package:flutter/material.dart';

// TODO: 由你来实现搜索页面 UI

class SearchView extends StatelessWidget {
  const SearchView({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Search')),
      body: const Center(child: Text('Search View - TODO')),
    );
  }
}
