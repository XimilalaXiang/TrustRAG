import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../auth/providers/auth_provider.dart';
import '../../chat/pages/chat_page.dart';
import '../../documents/pages/documents_page.dart';
import '../../settings/pages/model_config_page.dart';
import '../providers/workspace_provider.dart';

class DashboardPage extends ConsumerStatefulWidget {
  const DashboardPage({super.key});

  @override
  ConsumerState<DashboardPage> createState() => _DashboardPageState();
}

class _DashboardPageState extends ConsumerState<DashboardPage> {
  int _selectedIndex = 0;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isWide = MediaQuery.of(context).size.width > 768;
    final authState = ref.watch(authProvider);

    if (authState.status == AuthStatus.unauthenticated) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        context.go('/login');
      });
    }

    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            extended: isWide,
            selectedIndex: _selectedIndex,
            onDestinationSelected: (i) => setState(() => _selectedIndex = i),
            leading: Padding(
              padding: const EdgeInsets.symmetric(vertical: 16),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    Icons.auto_stories_rounded,
                    color: theme.colorScheme.primary,
                    size: 28,
                  ),
                  if (isWide) ...[
                    const SizedBox(width: 8),
                    Text(
                      'TrustRAG',
                      style: theme.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ],
              ),
            ),
            trailing: Expanded(
              child: Align(
                alignment: Alignment.bottomCenter,
                child: Padding(
                  padding: const EdgeInsets.only(bottom: 16),
                  child: IconButton(
                    icon: const Icon(Icons.logout),
                    tooltip: '退出登录',
                    onPressed: () {
                      ref.read(authProvider.notifier).logout();
                      context.go('/login');
                    },
                  ),
                ),
              ),
            ),
            destinations: const [
              NavigationRailDestination(
                icon: Icon(Icons.chat_outlined),
                selectedIcon: Icon(Icons.chat),
                label: Text('对话'),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.folder_outlined),
                selectedIcon: Icon(Icons.folder),
                label: Text('资料库'),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.workspaces_outlined),
                selectedIcon: Icon(Icons.workspaces),
                label: Text('工作区'),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.settings_outlined),
                selectedIcon: Icon(Icons.settings),
                label: Text('设置'),
              ),
            ],
          ),
          const VerticalDivider(width: 1),
          Expanded(
            child: _buildContent(),
          ),
        ],
      ),
    );
  }

  Widget _buildContent() {
    switch (_selectedIndex) {
      case 0:
        return _buildChatView();
      case 1:
        return _buildDocumentsView();
      case 2:
        return _buildWorkspacesView();
      case 3:
        return _buildSettingsView();
      default:
        return const SizedBox.shrink();
    }
  }

  Widget _buildChatView() {
    return const ChatPage();
  }

  Widget _buildDocumentsView() {
    return const DocumentsPage();
  }

  Widget _buildWorkspacesView() {
    final workspaces = ref.watch(workspaceProvider);
    final selectedWs = ref.watch(selectedWorkspaceProvider);

    return Column(
      children: [
        _buildHeader(
          '工作区',
          actions: [
            FilledButton.icon(
              onPressed: () => _showCreateWorkspaceDialog(),
              icon: const Icon(Icons.add, size: 18),
              label: const Text('新建'),
            ),
          ],
        ),
        Expanded(
          child: workspaces.when(
            loading: () => const Center(child: CircularProgressIndicator()),
            error: (e, _) => Center(child: Text('加载失败: $e')),
            data: (list) {
              if (list.isEmpty) {
                return Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.workspaces_outlined,
                          size: 80, color: Colors.grey.shade300),
                      const SizedBox(height: 16),
                      Text(
                        '还没有工作区',
                        style:
                            Theme.of(context).textTheme.headlineSmall?.copyWith(
                                  color: Colors.grey,
                                ),
                      ),
                      const SizedBox(height: 16),
                      FilledButton.icon(
                        onPressed: () => _showCreateWorkspaceDialog(),
                        icon: const Icon(Icons.add),
                        label: const Text('创建第一个工作区'),
                      ),
                    ],
                  ),
                );
              }

              return ListView.builder(
                padding: const EdgeInsets.all(16),
                itemCount: list.length,
                itemBuilder: (context, index) {
                  final ws = list[index];
                  final isSelected = selectedWs?.id == ws.id;
                  return Card(
                    color: isSelected
                        ? Theme.of(context)
                            .colorScheme
                            .primary
                            .withValues(alpha: 0.08)
                        : null,
                    child: ListTile(
                      leading: CircleAvatar(
                        backgroundColor: isSelected
                            ? Theme.of(context).colorScheme.primary
                            : Colors.grey.shade300,
                        child: Icon(
                          Icons.workspaces,
                          color: isSelected ? Colors.white : Colors.grey,
                          size: 20,
                        ),
                      ),
                      title: Text(ws.name,
                          style: const TextStyle(fontWeight: FontWeight.w600)),
                      subtitle: Text(ws.description ?? '无描述'),
                      trailing: isSelected
                          ? Icon(Icons.check_circle,
                              color: Theme.of(context).colorScheme.primary)
                          : null,
                      onTap: () {
                        ref.read(selectedWorkspaceProvider.notifier).state = ws;
                        setState(() => _selectedIndex = 0);
                      },
                    ),
                  );
                },
              );
            },
          ),
        ),
      ],
    );
  }

  Widget _buildSettingsView() {
    return Column(
      children: [
        _buildHeader('设置'),
        Expanded(
          child: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              Card(
                child: ListTile(
                  leading: const Icon(Icons.model_training),
                  title: const Text('模型配置'),
                  subtitle: const Text('管理 LLM 和 Embedding 模型'),
                  trailing: const Icon(Icons.chevron_right),
                  onTap: () {
                    Navigator.of(context).push(
                      MaterialPageRoute(
                          builder: (_) => const ModelConfigPage()),
                    );
                  },
                ),
              ),
              const SizedBox(height: 8),
              Card(
                child: ListTile(
                  leading: const Icon(Icons.person),
                  title: const Text('账户信息'),
                  subtitle: Text(
                      ref.watch(authProvider).user?['email'] ?? '未知'),
                  trailing: const Icon(Icons.chevron_right),
                  onTap: () {},
                ),
              ),
              const SizedBox(height: 8),
              Card(
                child: ListTile(
                  leading: const Icon(Icons.info_outline),
                  title: const Text('关于'),
                  subtitle: const Text('TrustRAG v1.0.0'),
                  trailing: const Icon(Icons.chevron_right),
                  onTap: () {},
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildHeader(String title,
      {String? subtitle, List<Widget>? actions}) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      decoration: BoxDecoration(
        border:
            Border(bottom: BorderSide(color: Colors.grey.shade200, width: 1)),
      ),
      child: Row(
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: Theme.of(context).textTheme.titleLarge?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
              ),
              if (subtitle != null)
                Text(
                  subtitle,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Colors.grey,
                      ),
                ),
            ],
          ),
          const Spacer(),
          if (actions != null) ...actions,
        ],
      ),
    );
  }

  void _showCreateWorkspaceDialog() {
    final nameController = TextEditingController();
    final descController = TextEditingController();

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('新建工作区'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: nameController,
              decoration: const InputDecoration(labelText: '名称'),
              autofocus: true,
            ),
            const SizedBox(height: 12),
            TextField(
              controller: descController,
              decoration: const InputDecoration(labelText: '描述（可选）'),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () async {
              if (nameController.text.trim().isEmpty) return;
              final ws = await ref
                  .read(workspaceProvider.notifier)
                  .createWorkspace(
                    nameController.text.trim(),
                    descController.text.trim().isEmpty
                        ? null
                        : descController.text.trim(),
                  );
              if (ws != null && ctx.mounted) {
                ref.read(selectedWorkspaceProvider.notifier).state = ws;
                Navigator.pop(ctx);
              }
            },
            child: const Text('创建'),
          ),
        ],
      ),
    );
  }
}
