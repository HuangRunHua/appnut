import 'package:flutter/material.dart';

// TODO: 由你来实现登录页面 UI
// 调用 fluxClient.emit("auth/login", {"username": ..., "password": ...})

class LoginView extends StatelessWidget {
  final VoidCallback onLoginSuccess;

  const LoginView({super.key, required this.onLoginSuccess});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(child: Text('Login View - TODO')),
    );
  }
}
