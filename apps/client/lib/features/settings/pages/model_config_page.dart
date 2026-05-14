import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/model_config_provider.dart';

class ModelConfigPage extends ConsumerStatefulWidget {
  const ModelConfigPage({super.key});

  @override
  ConsumerState<ModelConfigPage> createState() => _ModelConfigPageState();
}

class _ModelConfigPageState extends ConsumerState<ModelConfigPage> {
  @override
  void initState() {
    super.initState();
    ref.read(modelConfigProvider.notifier).loadConfigs();
  }

  @override
  Widget build(BuildContext context) {
    final configs = ref.watch(modelConfigProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('模型配置'),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showConfigDialog(),
        child: const Icon(Icons.add),
      ),
      body: configs.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('加载失败: $e')),
        data: (list) {
          if (list.isEmpty) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(Icons.model_training,
                      size: 80, color: Colors.grey.shade300),
                  const SizedBox(height: 16),
                  Text('暂无模型配置',
                      style: Theme.of(context)
                          .textTheme
                          .headlineSmall
                          ?.copyWith(color: Colors.grey)),
                  const SizedBox(height: 8),
                  FilledButton.icon(
                    onPressed: () => _showConfigDialog(),
                    icon: const Icon(Icons.add),
                    label: const Text('添加配置'),
                  ),
                ],
              ),
            );
          }

          return ListView.builder(
            padding: const EdgeInsets.all(16),
            itemCount: list.length,
            itemBuilder: (context, index) {
              final cfg = list[index];
              return Card(
                child: ListTile(
                  leading: CircleAvatar(
                    backgroundColor: cfg.isDefault
                        ? Colors.green.shade100
                        : Colors.grey.shade100,
                    child: Icon(
                      Icons.smart_toy,
                      color: cfg.isDefault ? Colors.green : Colors.grey,
                    ),
                  ),
                  title: Row(
                    children: [
                      Text(cfg.modelName,
                          style: const TextStyle(fontWeight: FontWeight.w600)),
                      if (cfg.isDefault) ...[
                        const SizedBox(width: 8),
                        Container(
                          padding: const EdgeInsets.symmetric(
                              horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: Colors.green.shade100,
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: const Text('默认',
                              style: TextStyle(
                                  fontSize: 11,
                                  color: Colors.green,
                                  fontWeight: FontWeight.w500)),
                        ),
                      ],
                    ],
                  ),
                  subtitle: Text(
                      '${cfg.provider} · ${cfg.apiBaseUrl}'),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        icon: const Icon(Icons.play_circle_outline),
                        tooltip: '测试连接',
                        onPressed: () async {
                          final ok = await ref
                              .read(modelConfigProvider.notifier)
                              .testConnection(cfg.id);
                          if (context.mounted) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(
                                content:
                                    Text(ok ? '连接成功' : '连接失败'),
                                backgroundColor: ok ? Colors.green : Colors.red,
                              ),
                            );
                          }
                        },
                      ),
                      IconButton(
                        icon: const Icon(Icons.edit_outlined, size: 20),
                        onPressed: () => _showConfigDialog(config: cfg),
                      ),
                      IconButton(
                        icon: const Icon(Icons.delete_outline, size: 20),
                        onPressed: () => ref
                            .read(modelConfigProvider.notifier)
                            .deleteConfig(cfg.id),
                      ),
                    ],
                  ),
                ),
              );
            },
          );
        },
      ),
    );
  }

  String _endpointHint(String provider) {
    switch (provider) {
      case 'openai':
        return 'https://api.openai.com/v1';
      case 'anthropic':
        return 'https://api.anthropic.com/v1';
      case 'ollama':
        return 'http://localhost:11434/v1';
      default:
        return 'https://your-api.com/v1';
    }
  }

  String _modelHint(String provider, String modelType) {
    if (modelType == 'embedding') {
      switch (provider) {
        case 'openai':
          return '如 text-embedding-3-small';
        case 'ollama':
          return '如 nomic-embed-text';
        default:
          return '如 bge-large-zh';
      }
    }
    switch (provider) {
      case 'openai':
        return '如 gpt-4o, gpt-4o-mini';
      case 'anthropic':
        return '如 claude-3-5-sonnet-20241022';
      case 'ollama':
        return '如 qwen2.5:7b, llama3.1:8b';
      default:
        return '如 your-model-name';
    }
  }

  void _showConfigDialog({ModelConfig? config}) {
    String selectedProvider = config?.provider ?? 'openai';
    final modelCtl = TextEditingController(text: config?.modelName ?? '');
    final endpointCtl =
        TextEditingController(text: config?.apiBaseUrl ?? '');
    final apiKeyCtl = TextEditingController();
    String modelType = 'llm';
    bool isDefault = config?.isDefault ?? false;

    showDialog(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDialogState) => AlertDialog(
          title: Text(config == null ? '添加模型配置' : '编辑模型配置'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                DropdownButtonFormField<String>(
                  initialValue: selectedProvider,
                  decoration: const InputDecoration(labelText: 'Provider'),
                  items: ['openai', 'anthropic', 'ollama', 'custom']
                      .map((p) => DropdownMenuItem(value: p, child: Text(p)))
                      .toList(),
                  onChanged: (v) {
                    setDialogState(() => selectedProvider = v ?? 'openai');
                    if (endpointCtl.text.isEmpty) {
                      endpointCtl.text = _endpointHint(selectedProvider);
                    }
                  },
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: modelCtl,
                  decoration: InputDecoration(
                    labelText: '模型名称',
                    hintText: _modelHint(selectedProvider, modelType),
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: endpointCtl,
                  decoration: InputDecoration(
                    labelText: 'API Endpoint',
                    hintText: _endpointHint(selectedProvider),
                    helperText: '填写到 /v1 即可，无需加 /chat/completions',
                    helperMaxLines: 2,
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: apiKeyCtl,
                  decoration: InputDecoration(
                    labelText: 'API Key',
                    hintText: config != null
                        ? '留空则不修改'
                        : selectedProvider == 'ollama'
                            ? '本地 Ollama 无需填写'
                            : 'sk-...',
                  ),
                  obscureText: true,
                ),
                const SizedBox(height: 12),
                DropdownButtonFormField<String>(
                  initialValue: modelType,
                  decoration: const InputDecoration(labelText: '模型类型'),
                  items: ['llm', 'embedding']
                      .map((t) => DropdownMenuItem(value: t, child: Text(t)))
                      .toList(),
                  onChanged: (v) =>
                      setDialogState(() => modelType = v ?? 'llm'),
                ),
                const SizedBox(height: 12),
                SwitchListTile(
                  title: const Text('设为默认'),
                  contentPadding: EdgeInsets.zero,
                  value: isDefault,
                  onChanged: (v) =>
                      setDialogState(() => isDefault = v),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(ctx),
              child: const Text('取消'),
            ),
            FilledButton(
              onPressed: () async {
                final data = <String, dynamic>{
                  'name': '${modelCtl.text} ($selectedProvider)',
                  'provider': selectedProvider,
                  'model_name': modelCtl.text,
                  'api_base_url': endpointCtl.text,
                  'is_default': isDefault,
                };
                if (apiKeyCtl.text.isNotEmpty) {
                  data['api_key'] = apiKeyCtl.text;
                }
                bool ok;
                if (config == null) {
                  ok = await ref
                      .read(modelConfigProvider.notifier)
                      .createConfig(data);
                } else {
                  ok = await ref
                      .read(modelConfigProvider.notifier)
                      .updateConfig(config.id, data);
                }
                if (ok && ctx.mounted) Navigator.pop(ctx);
              },
              child: Text(config == null ? '创建' : '保存'),
            ),
          ],
        ),
      ),
    );
  }
}
