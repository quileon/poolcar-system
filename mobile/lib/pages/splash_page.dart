import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:mobile/src/rust/api/auth.dart';

class SplashPage extends StatefulWidget {
  const SplashPage({super.key});

  @override
  State<SplashPage> createState() => _SplashPageState();
}

class _SplashPageState extends State<SplashPage> {
  @override
  void initState() {
    super.initState();
    _checkAuth();
  }

  Future<void> _checkAuth() async {
    final prefs = await SharedPreferences.getInstance();
    final token = prefs.getString('auth_token');

    if (!mounted) return;

    if (token != null && token.isNotEmpty) {
      try {
        final result = await verify(token: token);
        if (result.success) {
          if (!mounted) return;
          Navigator.of(context).pushReplacementNamed('/history');
          return;
        } else {
          // Token is invalid, remove it
          await prefs.remove('auth_token');
        }
      } catch (e) {
        // In case of a network error or other exceptions, remove token or handle appropriately.
        // For simplicity, we just navigate to login if verify fails for any reason
        await prefs.remove('auth_token');
      }
    }

    if (!mounted) return;
    Navigator.of(context).pushReplacementNamed('/login');
  }

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(
        child: CircularProgressIndicator(),
      ),
    );
  }
}
