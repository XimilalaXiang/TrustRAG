import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

class BackendManager {
  static final BackendManager _instance = BackendManager._();
  factory BackendManager() => _instance;
  BackendManager._();

  Process? _process;
  int? _port;
  bool _isRunning = false;
  final _readyCompleter = Completer<void>();

  int? get port => _port;
  bool get isRunning => _isRunning;
  String get baseUrl => 'http://127.0.0.1:$_port';
  Future<void> get ready => _readyCompleter.future;

  /// Whether this platform should run an embedded backend.
  static bool get shouldRunEmbedded {
    if (kIsWeb) return false;
    return Platform.isWindows || Platform.isLinux || Platform.isMacOS || Platform.isAndroid;
  }

  Future<void> start() async {
    if (!shouldRunEmbedded || _isRunning) return;

    _port = await _findFreePort();
    final backendPath = await _findBackendBinary();

    if (backendPath == null) {
      debugPrint('[BackendManager] Backend binary not found, skipping embedded mode');
      if (!_readyCompleter.isCompleted) _readyCompleter.complete();
      return;
    }

    final dataDir = await _getDataDir();
    await Directory(dataDir).create(recursive: true);

    final dbPath = p.join(dataDir, 'trustrag.db');

    final env = {
      'TRUSTRAG__LISTEN_ADDR': '127.0.0.1:$_port',
      'TRUSTRAG__DATABASE_URL': 'sqlite://$dbPath?mode=rwc',
      'TRUSTRAG__DATA_DIR': dataDir,
      'TRUSTRAG__JWT_SECRET': _generateJwtSecret(),
      'TRUSTRAG__DOC_PROCESSOR_URL': 'http://127.0.0.1:0',
      'TRUSTRAG__MAX_UPLOAD_SIZE_MB': '100',
      'RUST_LOG': 'trustrag_backend=info',
    };

    debugPrint('[BackendManager] Starting backend on port $_port');
    debugPrint('[BackendManager] Data dir: $dataDir');
    debugPrint('[BackendManager] Binary: $backendPath');

    try {
      _process = await Process.start(
        backendPath,
        [],
        environment: env,
        workingDirectory: p.dirname(backendPath),
      );

      _process!.stdout.listen((data) {
        final line = String.fromCharCodes(data).trim();
        if (line.isNotEmpty) debugPrint('[Backend] $line');
        if (line.contains('ready and listening')) {
          _isRunning = true;
          if (!_readyCompleter.isCompleted) _readyCompleter.complete();
        }
      });

      _process!.stderr.listen((data) {
        final line = String.fromCharCodes(data).trim();
        if (line.isNotEmpty) debugPrint('[Backend:ERR] $line');
      });

      _process!.exitCode.then((code) {
        debugPrint('[BackendManager] Backend exited with code $code');
        _isRunning = false;
        _process = null;
      });

      // Wait up to 15 seconds for backend to be ready
      await _readyCompleter.future.timeout(
        const Duration(seconds: 15),
        onTimeout: () {
          debugPrint('[BackendManager] Backend startup timed out, proceeding anyway');
          _isRunning = true;
          if (!_readyCompleter.isCompleted) _readyCompleter.complete();
        },
      );
    } catch (e) {
      debugPrint('[BackendManager] Failed to start backend: $e');
      if (!_readyCompleter.isCompleted) _readyCompleter.complete();
    }
  }

  Future<void> stop() async {
    if (_process != null) {
      debugPrint('[BackendManager] Stopping backend...');
      _process!.kill(ProcessSignal.sigterm);
      try {
        await _process!.exitCode.timeout(const Duration(seconds: 5));
      } catch (_) {
        _process!.kill(ProcessSignal.sigkill);
      }
      _process = null;
      _isRunning = false;
    }
  }

  Future<int> _findFreePort() async {
    final server = await ServerSocket.bind(InternetAddress.loopbackIPv4, 0);
    final port = server.port;
    await server.close();
    return port;
  }

  Future<String?> _findBackendBinary() async {
    if (Platform.isAndroid) {
      return _findAndroidBinary();
    }

    final binaryName = Platform.isWindows
        ? 'trustrag-backend.exe'
        : 'trustrag-backend';

    final exeDir = p.dirname(Platform.resolvedExecutable);
    final candidates = [
      p.join(exeDir, binaryName),
      p.join(exeDir, 'data', 'flutter_assets', binaryName),
      p.join(exeDir, '..', 'Resources', binaryName), // macOS .app bundle
      p.join(exeDir, '..', 'lib', binaryName),
      p.join(exeDir, 'backend', binaryName),
    ];

    for (final candidate in candidates) {
      if (await File(candidate).exists()) {
        return candidate;
      }
    }

    return null;
  }

  Future<String?> _findAndroidBinary() async {
    // On Android, the native library is bundled in the APK's lib directory.
    // Flutter loads native libs from the app's nativeLibraryDir.
    // We named it libtrustrap_backend.so to satisfy Android's lib naming convention.
    const libName = 'libtrustrap_backend.so';
    final appInfo = await getApplicationSupportDirectory();
    final nativeLibDir = p.join(p.dirname(p.dirname(appInfo.path)), 'lib');

    final candidates = [
      p.join(nativeLibDir, libName),
      '/data/data/com.trustrag.app/lib/$libName',
    ];

    for (final candidate in candidates) {
      if (await File(candidate).exists()) {
        debugPrint('[BackendManager] Found Android binary: $candidate');
        return candidate;
      }
    }

    // Fallback: search the app's native library path via resolvedExecutable parent
    final exeDir = p.dirname(Platform.resolvedExecutable);
    final fallback = p.join(exeDir, '..', 'lib', libName);
    if (await File(fallback).exists()) {
      return fallback;
    }

    return null;
  }

  Future<String> _getDataDir() async {
    final appSupport = await getApplicationSupportDirectory();
    return p.join(appSupport.path, 'TrustRAG');
  }

  String _generateJwtSecret() {
    final hostname = Platform.isAndroid ? 'android-device' : Platform.localHostname;
    final seed = hostname + Platform.operatingSystem;
    return seed.hashCode.toRadixString(36).padLeft(32, 'x');
  }
}
