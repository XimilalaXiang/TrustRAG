import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../auth/providers/auth_provider.dart';

class ModelConfig {
  final String id;
  final String provider;
  final String modelName;
  final String apiEndpoint;
  final String modelType;
  final bool isDefault;

  ModelConfig({
    required this.id,
    required this.provider,
    required this.modelName,
    required this.apiEndpoint,
    required this.modelType,
    required this.isDefault,
  });

  factory ModelConfig.fromJson(Map<String, dynamic> json) {
    return ModelConfig(
      id: json['id'],
      provider: json['provider'],
      modelName: json['model_name'],
      apiEndpoint: json['api_endpoint'],
      modelType: json['model_type'] ?? 'llm',
      isDefault: json['is_default'] ?? false,
    );
  }
}

class ModelConfigNotifier
    extends StateNotifier<AsyncValue<List<ModelConfig>>> {
  final Ref ref;

  ModelConfigNotifier(this.ref) : super(const AsyncValue.data([]));

  Future<void> loadConfigs() async {
    state = const AsyncValue.loading();
    try {
      final api = ref.read(apiClientProvider);
      final resp = await api.dio.get('/model-configs');
      final list = (resp.data as List)
          .map((j) => ModelConfig.fromJson(j))
          .toList();
      state = AsyncValue.data(list);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  Future<bool> createConfig(Map<String, dynamic> data) async {
    try {
      final api = ref.read(apiClientProvider);
      await api.dio.post('/model-configs', data: data);
      await loadConfigs();
      return true;
    } catch (_) {
      return false;
    }
  }

  Future<bool> updateConfig(String id, Map<String, dynamic> data) async {
    try {
      final api = ref.read(apiClientProvider);
      await api.dio.put('/model-configs/$id', data: data);
      await loadConfigs();
      return true;
    } catch (_) {
      return false;
    }
  }

  Future<bool> deleteConfig(String id) async {
    try {
      final api = ref.read(apiClientProvider);
      await api.dio.delete('/model-configs/$id');
      state = AsyncValue.data(
        (state.value ?? []).where((c) => c.id != id).toList(),
      );
      return true;
    } catch (_) {
      return false;
    }
  }

  Future<bool> testConnection(String id) async {
    try {
      final api = ref.read(apiClientProvider);
      final resp = await api.dio.post('/model-configs/$id/test');
      return resp.data['success'] == true;
    } catch (_) {
      return false;
    }
  }
}

final modelConfigProvider = StateNotifierProvider<ModelConfigNotifier,
    AsyncValue<List<ModelConfig>>>((ref) {
  return ModelConfigNotifier(ref);
});
