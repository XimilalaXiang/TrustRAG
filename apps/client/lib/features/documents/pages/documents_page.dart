import 'dart:async';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../dashboard/providers/workspace_provider.dart';
import '../providers/document_provider.dart';
import 'document_viewer_page.dart';

class DocumentsPage extends ConsumerStatefulWidget {
  const DocumentsPage({super.key});

  @override
  ConsumerState<DocumentsPage> createState() => _DocumentsPageState();
}

class _DocumentsPageState extends ConsumerState<DocumentsPage> {
  Timer? _refreshTimer;

  @override
  void initState() {
    super.initState();
    final ws = ref.read(selectedWorkspaceProvider);
    if (ws != null) {
      ref.read(documentProvider.notifier).loadDocuments(ws.id);
    }
  }

  @override
  void dispose() {
    _refreshTimer?.cancel();
    super.dispose();
  }

  void _startAutoRefresh(List<Document> docs) {
    _refreshTimer?.cancel();
    final hasProcessing = docs.any((d) =>
        d.processingStatus == 'processing' ||
        d.processingStatus == 'chunking' ||
        d.processingStatus == 'embedding' ||
        d.processingStatus == 'pending');
    if (hasProcessing) {
      _refreshTimer = Timer(const Duration(seconds: 3), () {
        final ws = ref.read(selectedWorkspaceProvider);
        if (ws != null && mounted) {
          ref.read(documentProvider.notifier).loadDocuments(ws.id);
        }
      });
    }
  }

  Future<void> _uploadFile() async {
    final ws = ref.read(selectedWorkspaceProvider);
    if (ws == null) return;

    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['pdf', 'docx', 'txt'],
      withData: true,
    );

    if (result != null && result.files.isNotEmpty) {
      final file = result.files.first;
      if (file.bytes != null) {
        final success = await ref
            .read(documentProvider.notifier)
            .uploadDocument(ws.id, file.bytes!, file.name);
        if (mounted && success) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('${file.name} 上传成功')),
          );
        }
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final ws = ref.watch(selectedWorkspaceProvider);
    final docs = ref.watch(documentProvider);

    if (ws == null) {
      return Center(
        child: Text('请先选择工作区', style: TextStyle(color: Colors.grey.shade500)),
      );
    }

    return Column(
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
          decoration: BoxDecoration(
            border: Border(
                bottom: BorderSide(color: Colors.grey.shade200, width: 1)),
          ),
          child: Row(
            children: [
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('资料库',
                      style: Theme.of(context)
                          .textTheme
                          .titleLarge
                          ?.copyWith(fontWeight: FontWeight.bold)),
                  Text(ws.name,
                      style: Theme.of(context)
                          .textTheme
                          .bodySmall
                          ?.copyWith(color: Colors.grey)),
                ],
              ),
              const Spacer(),
              FilledButton.icon(
                onPressed: _uploadFile,
                icon: const Icon(Icons.upload_file, size: 18),
                label: const Text('上传文档'),
              ),
            ],
          ),
        ),
        Expanded(
          child: docs.when(
            loading: () => const Center(child: CircularProgressIndicator()),
            error: (e, _) => Center(child: Text('加载失败: $e')),
            data: (list) {
              _startAutoRefresh(list);
              if (list.isEmpty) {
                return Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.upload_file,
                          size: 80, color: Colors.grey.shade300),
                      const SizedBox(height: 16),
                      Text('暂无文档',
                          style: Theme.of(context)
                              .textTheme
                              .headlineSmall
                              ?.copyWith(color: Colors.grey)),
                      const SizedBox(height: 8),
                      Text('点击"上传文档"添加 PDF、DOCX 或 TXT 文件',
                          style: TextStyle(color: Colors.grey.shade500)),
                    ],
                  ),
                );
              }

              return ListView.builder(
                padding: const EdgeInsets.all(16),
                itemCount: list.length,
                itemBuilder: (context, index) {
                  final doc = list[index];
                  return Card(
                    child: ListTile(
                      onTap: () {
                        Navigator.of(context).push(
                          MaterialPageRoute(
                            builder: (_) => DocumentViewerPage(
                              document: doc,
                              workspaceId: ws.id,
                            ),
                          ),
                        );
                      },
                      leading: _fileIcon(doc.fileType),
                      title: Text(doc.originalFilename,
                          style: const TextStyle(fontWeight: FontWeight.w500)),
                      subtitle: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Text(doc.fileSizeFormatted),
                              const SizedBox(width: 12),
                              _statusChip(doc.processingStatus),
                              if (doc.chunkCount != null) ...[
                                const SizedBox(width: 12),
                                Text('${doc.chunkCount} 分块',
                                    style: TextStyle(
                                        color: Colors.grey.shade600, fontSize: 12)),
                              ],
                            ],
                          ),
                          if (doc.processingStatus == 'failed' && doc.processingError != null)
                            Padding(
                              padding: const EdgeInsets.only(top: 4),
                              child: Text(
                                doc.processingError!,
                                maxLines: 2,
                                overflow: TextOverflow.ellipsis,
                                style: TextStyle(fontSize: 11, color: Colors.red.shade400),
                              ),
                            ),
                        ],
                      ),
                      trailing: PopupMenuButton(
                        itemBuilder: (ctx) => [
                          const PopupMenuItem(
                              value: 'delete', child: Text('删除')),
                        ],
                        onSelected: (value) async {
                          if (value == 'delete') {
                            await ref
                                .read(documentProvider.notifier)
                                .deleteDocument(ws.id, doc.id);
                          }
                        },
                      ),
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

  Widget _fileIcon(String fileType) {
    IconData icon;
    Color color;
    switch (fileType.toLowerCase()) {
      case 'pdf':
        icon = Icons.picture_as_pdf;
        color = Colors.red;
        break;
      case 'docx':
        icon = Icons.description;
        color = Colors.blue;
        break;
      default:
        icon = Icons.insert_drive_file;
        color = Colors.grey;
    }
    return CircleAvatar(
      backgroundColor: color.withValues(alpha: 0.1),
      child: Icon(icon, color: color, size: 20),
    );
  }

  Widget _statusChip(String status) {
    Color color;
    String label;
    bool isLoading = false;
    switch (status) {
      case 'ready':
        color = Colors.green;
        label = '就绪';
        break;
      case 'processing':
        color = Colors.orange;
        label = '解析中';
        isLoading = true;
        break;
      case 'chunking':
        color = Colors.orange;
        label = '分块中';
        isLoading = true;
        break;
      case 'embedding':
        color = Colors.blue;
        label = '向量化中';
        isLoading = true;
        break;
      case 'failed':
        color = Colors.red;
        label = '失败';
        break;
      default:
        color = Colors.grey;
        label = '等待';
    }
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (isLoading) ...[
            SizedBox(
              width: 10,
              height: 10,
              child: CircularProgressIndicator(
                strokeWidth: 1.5,
                color: color,
              ),
            ),
            const SizedBox(width: 4),
          ],
          Text(label,
              style: TextStyle(color: color, fontSize: 12, fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }
}
