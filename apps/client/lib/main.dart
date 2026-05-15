import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'core/router/app_router.dart';
import 'core/services/backend_manager.dart';
import 'core/theme/app_theme.dart';

final themeModeProvider = StateProvider<ThemeMode>((ref) => ThemeMode.system);

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  if (BackendManager.shouldRunEmbedded) {
    debugPrint('[App] Starting embedded backend...');
    await BackendManager().start();
    debugPrint('[App] Backend ready at ${BackendManager().baseUrl}');
  }

  runApp(const ProviderScope(child: TrustRAGApp()));
}

class TrustRAGApp extends ConsumerWidget {
  const TrustRAGApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode = ref.watch(themeModeProvider);
    return MaterialApp.router(
      title: 'TrustRAG',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.light,
      darkTheme: AppTheme.dark,
      themeMode: themeMode,
      routerConfig: appRouter,
    );
  }
}
