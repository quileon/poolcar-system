import 'package:flutter/material.dart';
import 'package:mobile/pages/history_page.dart';
import 'package:mobile/pages/login_page.dart';
import 'package:mobile/pages/camera_page.dart';
import 'package:mobile/pages/post_camera_page.dart';
import 'package:mobile/pages/splash_page.dart';
import 'package:mobile/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Poolcar System',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      initialRoute: '/',
      routes: {
        '/': (context) => const SplashPage(),
        '/login': (context) => const LoginPage(),
        '/history': (context) => const HistoryPage(),
        '/camera': (context) => const CameraPage(),
        '/post-camera': (context) => const PostCameraPage(),
      },
    );
  }
}
