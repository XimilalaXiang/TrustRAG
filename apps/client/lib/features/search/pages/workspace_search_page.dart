import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../auth/providers/auth_provider.dart';
import '../../dashboard/providers/workspace_provider.dart';

class WorkspaceSearchPage extends ConsumerStatefulWidget {
  const WorkspaceSearchPage({super.key});

  @override
  ConsumerState<WorkspaceSearchPage> createState() =>
      _WorkspaceSearchPageState();
}

class _WorkspaceSearchPageState extends ConsumerState<WorkspaceSearchPage> {
  final _searchController = TextEditingController();
  List<Map<String, dynamic>> _results = [];
  bool _loading = false;
  String? _error;

  Future<void> _search() async {
    final query = _searchController.text.trim();
    if (query.isEmpty) return;

    final ws = ref.read(selectedWorkspaceProvider);
    if (ws == null) return;

    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      final api = ref.read(apiClientProvider);
      final resp = await api.dio.post(
        '/workspaces/${ws.id}/search',
        data: {
          'query': query,
          'top_k': 20,
          'mode': 'hybrid',
        },
      );

      final results = (resp.data['results'] as List?)
              ?.map((e) => e as Map<String, dynamic>)
              .toList() ??
          [];

      setState(() {
        _results = results;
        _loading = false;
      });
    } catch (e) {
      setState(() {
        _error = '搜索失败: $e';
        _loading = false;
      });
    }
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ws = ref.watch(selectedWorkspaceProvider);
    final theme = Theme.of(context);

    if (ws == null) {
      return Center(
        child: Text('请先选择工作区',
            style: TextStyle(color: Colors.grey.shade500)),
      );
    }

    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(20),
          decoration: BoxDecoration(
            border: Border(
                bottom: BorderSide(color: Colors.grey.shade200, width: 1)),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('全局搜索',
                  style: theme.textTheme.titleLarge
                      ?.copyWith(fontWeight: FontWeight.bold)),
              const SizedBox(height: 4),
              Text('在「${ws.name}」的所有文档中搜索',
                  style:
                      theme.textTheme.bodySmall?.copyWith(color: Colors.grey)),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _searchController,
                      decoration: InputDecoration(
                        hintText: '输入关键词或问题...',
                        prefixIcon: const Icon(Icons.search),
                        filled: true,
                        fillColor:
                            theme.colorScheme.surfaceContainerHighest,
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(12),
                          borderSide: BorderSide.none,
                        ),
                      ),
                      onSubmitted: (_) => _search(),
                    ),
                  ),
                  const SizedBox(width: 12),
                  FilledButton(
                    onPressed: _loading ? null : _search,
                    child: _loading
                        ? const SizedBox(
                            width: 20,
                            height: 20,
                            child:
                                CircularProgressIndicator(strokeWidth: 2))
                        : const Text('搜索'),
                  ),
                ],
              ),
            ],
          ),
        ),
        if (_error != null)
          Padding(
            padding: const EdgeInsets.all(16),
            child: Text(_error!,
                style: TextStyle(color: Colors.red.shade400)),
          ),
        Expanded(
          child: _results.isEmpty && !_loading
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.search,
                          size: 80, color: Colors.grey.shade300),
                      const SizedBox(height: 16),
                      Text(
                        _searchController.text.isEmpty
                            ? '输入关键词开始搜索'
                            : '未找到相关结果',
                        style: TextStyle(color: Colors.grey.shade500),
                      ),
                    ],
                  ),
                )
              : ListView.builder(
                  padding: const EdgeInsets.all(16),
                  itemCount: _results.length,
                  itemBuilder: (context, index) {
                    final r = _results[index];
                    final content = r['content'] as String? ?? '';
                    final docId =
                        r['document_id'] as String? ?? '';
                    final score = (r['relevance_score'] as num?)?.toDouble() ?? 0;
                    final page = r['page_start'] as int?;
                    final heading = r['heading_path'] as String?;

                    return Card(
                      margin: const EdgeInsets.only(bottom: 12),
                      child: Padding(
                        padding: const EdgeInsets.all(16),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Row(
                              children: [
                                Container(
                                  padding: const EdgeInsets.symmetric(
                                      horizontal: 8, vertical: 2),
                                  decoration: BoxDecoration(
                                    color: theme.colorScheme.primaryContainer,
                                    borderRadius: BorderRadius.circular(8),
                                  ),
                                  child: Text('#${index + 1}',
                                      style: TextStyle(
                                        fontSize: 12,
                                        fontWeight: FontWeight.bold,
                                        color: theme.colorScheme.primary,
                                      )),
                                ),
                                const SizedBox(width: 8),
                                Expanded(
                                  child: Text(
                                      docId.length > 8
                                          ? '文档 ${docId.substring(0, 8)}...'
                                          : '文档 $docId',
                                      style: const TextStyle(
                                          fontWeight: FontWeight.w600,
                                          fontSize: 14)),
                                ),
                                Text(
                                  '${(score * 100).toStringAsFixed(1)}%',
                                  style: TextStyle(
                                    color: theme.colorScheme.primary,
                                    fontWeight: FontWeight.w500,
                                    fontSize: 13,
                                  ),
                                ),
                              ],
                            ),
                            if (heading != null || page != null)
                              Padding(
                                padding: const EdgeInsets.only(top: 6),
                                child: Row(
                                  children: [
                                    if (heading != null) ...[
                                      Icon(Icons.bookmark_outline,
                                          size: 14,
                                          color: Colors.grey.shade500),
                                      const SizedBox(width: 4),
                                      Flexible(
                                        child: Text(heading,
                                            style: TextStyle(
                                                fontSize: 12,
                                                color:
                                                    Colors.grey.shade600),
                                            maxLines: 1,
                                            overflow:
                                                TextOverflow.ellipsis),
                                      ),
                                    ],
                                    if (page != null) ...[
                                      const SizedBox(width: 12),
                                      Icon(Icons.description_outlined,
                                          size: 14,
                                          color: Colors.grey.shade500),
                                      const SizedBox(width: 4),
                                      Text('第 $page 页',
                                          style: TextStyle(
                                              fontSize: 12,
                                              color:
                                                  Colors.grey.shade600)),
                                    ],
                                  ],
                                ),
                              ),
                            const SizedBox(height: 8),
                            Text(
                              content.length > 300
                                  ? '${content.substring(0, 300)}...'
                                  : content,
                              style: const TextStyle(
                                  fontSize: 13, height: 1.6),
                            ),
                          ],
                        ),
                      ),
                    );
                  },
                ),
        ),
      ],
    );
  }
}
