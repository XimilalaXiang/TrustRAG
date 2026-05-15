import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

final devModeProvider = StateNotifierProvider<DevModeNotifier, bool>((ref) {
  return DevModeNotifier();
});

class DevModeNotifier extends StateNotifier<bool> {
  DevModeNotifier() : super(false) {
    _load();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    state = prefs.getBool('dev_mode') ?? false;
  }

  Future<void> toggle() async {
    state = !state;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool('dev_mode', state);
  }
}

class DebugLogBuffer {
  static final DebugLogBuffer _instance = DebugLogBuffer._();
  factory DebugLogBuffer() => _instance;
  DebugLogBuffer._();

  final List<String> _logs = [];
  static const int maxLogs = 500;

  List<String> get logs => List.unmodifiable(_logs);

  void add(String message) {
    final timestamp = DateTime.now().toIso8601String().substring(11, 23);
    _logs.add('[$timestamp] $message');
    if (_logs.length > maxLogs) {
      _logs.removeRange(0, _logs.length - maxLogs);
    }
  }

  void clear() => _logs.clear();
}
