import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_markdown/flutter_markdown.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';

import '../../auth/providers/auth_provider.dart';
import '../../dashboard/providers/workspace_provider.dart';
import '../../reader/pages/pdf_viewer_page.dart';
import '../providers/chat_provider.dart';

class ChatPage extends ConsumerStatefulWidget {
  const ChatPage({super.key});

  @override
  ConsumerState<ChatPage> createState() => _ChatPageState();
}

class _ChatPageState extends ConsumerState<ChatPage> {
  final _controller = TextEditingController();
  final _scrollController = ScrollController();
  bool _isSending = false;
  String _streamingContent = '';
  List<Citation> _streamingCitations = [];

  @override
  void initState() {
    super.initState();
    final ws = ref.read(selectedWorkspaceProvider);
    if (ws != null) {
      ref.read(conversationProvider.notifier).loadConversations(ws.id);
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  Future<void> _sendMessage() async {
    final ws = ref.read(selectedWorkspaceProvider);
    if (ws == null || _controller.text.trim().isEmpty) return;

    var conv = ref.read(selectedConversationProvider);
    if (conv == null) {
      conv = await ref
          .read(conversationProvider.notifier)
          .createConversation(ws.id, _controller.text.trim());
      if (conv == null) return;
      ref.read(selectedConversationProvider.notifier).state = conv;
    }

    final userText = _controller.text.trim();
    _controller.clear();

    final userMsg = ChatMessage(
      id: 'temp-${DateTime.now().millisecondsSinceEpoch}',
      role: 'user',
      content: userText,
      createdAt: DateTime.now(),
    );

    final msgs = ref.read(messagesProvider);
    ref.read(messagesProvider.notifier).state = [...msgs, userMsg];

    setState(() {
      _isSending = true;
      _streamingContent = '';
      _streamingCitations = [];
    });

    _scrollToBottom();

    try {
      final prefs = await SharedPreferences.getInstance();
      final token = prefs.getString('auth_token') ?? '';
      final api = ref.read(apiClientProvider);
      var baseUrl = api.dio.options.baseUrl;
      if (baseUrl.startsWith('/')) {
        baseUrl = Uri.base.origin + baseUrl;
      }

      final request = http.Request(
        'POST',
        Uri.parse(
            '$baseUrl/workspaces/${ws.id}/conversations/${conv.id}/messages'),
      );
      request.headers['Content-Type'] = 'application/json';
      request.headers['Authorization'] = 'Bearer $token';
      request.headers['Accept'] = 'text/event-stream';
      request.body = jsonEncode({
        'content': userText,
        'stream': true,
      });

      final client = http.Client();
      final response = await client.send(request);

      if (response.statusCode != 200) {
        throw Exception('HTTP ${response.statusCode}');
      }

      String assistantId = '';
      String currentEventType = '';

      await for (final chunk
          in response.stream.transform(utf8.decoder)) {
        for (final line in chunk.split('\n')) {
          final trimmed = line.trim();
          if (trimmed.isEmpty) {
            currentEventType = '';
            continue;
          }
          if (trimmed.startsWith('event: ')) {
            currentEventType = trimmed.substring(7).trim();
            continue;
          }
          if (!trimmed.startsWith('data:')) continue;
          final jsonStr = trimmed.substring(5).trim();
          if (jsonStr.isEmpty) continue;

          final eventType = currentEventType;

          if (eventType == 'error') {
            if (mounted) {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Text('AI 错误: $jsonStr'),
                  backgroundColor: Colors.red,
                  duration: const Duration(seconds: 6),
                ),
              );
            }
            continue;
          }

          try {
            final event = jsonDecode(jsonStr);
            if (eventType == 'message_start') {
              assistantId = event['message_id'] ?? '';
            } else if (eventType == 'citation') {
              setState(() {
                _streamingCitations.add(Citation.fromJson(event));
              });
            } else if (eventType == 'text_delta') {
              setState(() {
                _streamingContent += event['delta'] ?? event['text'] ?? '';
              });
              _scrollToBottom();
            } else if (eventType == 'message_end') {
              final fullContent = _streamingContent;
              if (fullContent.isNotEmpty) {
                final aiMsg = ChatMessage(
                  id: assistantId,
                  role: 'assistant',
                  content: fullContent,
                  citations: List.from(_streamingCitations),
                  createdAt: DateTime.now(),
                );
                ref.read(messagesProvider.notifier).state = [
                  ...ref.read(messagesProvider),
                  aiMsg,
                ];
              }
              setState(() {
                _streamingContent = '';
                _streamingCitations = [];
              });
            }
          } catch (_) {}
        }
      }
      client.close();
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('发送失败: $e')));
      }
    } finally {
      if (mounted) {
        setState(() => _isSending = false);
      }
    }
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      }
    });
  }

  Future<void> _selectConversation(Conversation conv) async {
    ref.read(selectedConversationProvider.notifier).state = conv;
    try {
      final api = ref.read(apiClientProvider);
      final resp = await api.dio.get(
          '/workspaces/${conv.workspaceId}/conversations/${conv.id}/messages');
      final list =
          (resp.data as List).map((j) => ChatMessage.fromJson(j)).toList();
      ref.read(messagesProvider.notifier).state = list;
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    final ws = ref.watch(selectedWorkspaceProvider);
    final convs = ref.watch(conversationProvider);
    final messages = ref.watch(messagesProvider);
    final selectedConv = ref.watch(selectedConversationProvider);

    if (ws == null) {
      return Center(
        child:
            Text('请先选择工作区', style: TextStyle(color: Colors.grey.shade500)),
      );
    }

    return Row(
      children: [
        SizedBox(
          width: 260,
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.all(12),
                child: SizedBox(
                  width: double.infinity,
                  child: OutlinedButton.icon(
                    onPressed: () {
                      ref.read(selectedConversationProvider.notifier).state =
                          null;
                      ref.read(messagesProvider.notifier).state = [];
                    },
                    icon: const Icon(Icons.add, size: 18),
                    label: const Text('新对话'),
                  ),
                ),
              ),
              const Divider(height: 1),
              Expanded(
                child: convs.when(
                  loading: () =>
                      const Center(child: CircularProgressIndicator()),
                  error: (e, _) => Center(child: Text('$e')),
                  data: (list) {
                    if (list.isEmpty) {
                      return Center(
                        child: Text('暂无对话',
                            style: TextStyle(color: Colors.grey.shade500)),
                      );
                    }
                    return ListView.builder(
                      itemCount: list.length,
                      itemBuilder: (context, i) {
                        final conv = list[i];
                        final isSelected = selectedConv?.id == conv.id;
                        return ListTile(
                          selected: isSelected,
                          selectedTileColor: Theme.of(context)
                              .colorScheme
                              .primaryContainer
                              .withValues(alpha: 0.3),
                          leading: const Icon(Icons.chat_bubble_outline,
                              size: 18),
                          title: Text(
                            conv.title ?? '对话',
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                          onTap: () => _selectConversation(conv),
                          trailing: IconButton(
                            icon: const Icon(Icons.delete_outline, size: 16),
                            onPressed: () {
                              ref
                                  .read(conversationProvider.notifier)
                                  .deleteConversation(ws.id, conv.id);
                              if (selectedConv?.id == conv.id) {
                                ref
                                    .read(
                                        selectedConversationProvider.notifier)
                                    .state = null;
                                ref.read(messagesProvider.notifier).state =
                                    [];
                              }
                            },
                          ),
                        );
                      },
                    );
                  },
                ),
              ),
            ],
          ),
        ),
        const VerticalDivider(width: 1),
        Expanded(
          child: Column(
            children: [
              Expanded(
                child: messages.isEmpty && _streamingContent.isEmpty
                    ? _buildEmptyChat()
                    : _buildMessageList(messages),
              ),
              _buildInputBar(),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildEmptyChat() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.auto_awesome, size: 64, color: Colors.grey.shade300),
          const SizedBox(height: 16),
          Text('开始一段新对话',
              style: Theme.of(context)
                  .textTheme
                  .headlineSmall
                  ?.copyWith(color: Colors.grey)),
          const SizedBox(height: 8),
          Text('基于你的文档进行 AI 问答',
              style: TextStyle(color: Colors.grey.shade500)),
        ],
      ),
    );
  }

  Widget _buildMessageList(List<ChatMessage> messages) {
    return ListView.builder(
      controller: _scrollController,
      padding: const EdgeInsets.all(16),
      itemCount: messages.length + (_streamingContent.isNotEmpty ? 1 : 0),
      itemBuilder: (context, i) {
        if (i < messages.length) {
          return _buildMessageBubble(messages[i]);
        }
        return _buildStreamingBubble();
      },
    );
  }

  Widget _buildMessageBubble(ChatMessage msg) {
    final isUser = msg.role == 'user';
    return Align(
      alignment: isUser ? Alignment.centerRight : Alignment.centerLeft,
      child: Container(
        constraints:
            BoxConstraints(maxWidth: MediaQuery.of(context).size.width * 0.65),
        margin: const EdgeInsets.symmetric(vertical: 6),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              padding: const EdgeInsets.all(14),
              decoration: BoxDecoration(
                color: isUser
                    ? Theme.of(context).colorScheme.primaryContainer
                    : Theme.of(context).colorScheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(16),
              ),
              child: isUser
                  ? Text(msg.content)
                  : MarkdownBody(
                      data: msg.content,
                      selectable: true,
                    ),
            ),
            if (!isUser && msg.citations.isNotEmpty)
              _buildCitationCards(msg.citations),
          ],
        ),
      ),
    );
  }

  Widget _buildCitationCards(List<Citation> citations) {
    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.only(left: 4, bottom: 6),
            child: Text(
              '引用来源 (${citations.length})',
              style: TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w600,
                color: Theme.of(context).colorScheme.primary,
              ),
            ),
          ),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: citations.map((c) => _buildCitationChip(c)).toList(),
          ),
        ],
      ),
    );
  }

  Widget _buildCitationChip(Citation citation) {
    final scorePercent = (citation.score * 100).toStringAsFixed(0);
    return Tooltip(
      richMessage: TextSpan(
        children: [
          TextSpan(
            text: citation.text,
            style: const TextStyle(fontSize: 12),
          ),
        ],
      ),
      child: InkWell(
        borderRadius: BorderRadius.circular(8),
        onTap: () => _showCitationDetail(citation),
        child: Container(
          constraints: const BoxConstraints(maxWidth: 280),
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.primaryContainer.withValues(alpha: 0.3),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(
              color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.2),
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 22,
                height: 22,
                alignment: Alignment.center,
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.primary,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  '${citation.index + 1}',
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    color: Theme.of(context).colorScheme.onPrimary,
                  ),
                ),
              ),
              const SizedBox(width: 8),
              Flexible(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    if (citation.heading != null && citation.heading!.isNotEmpty)
                      Text(
                        citation.heading!,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w500),
                      ),
                    Row(
                      children: [
                        if (citation.page != null)
                          Text(
                            'p.${citation.page}',
                            style: TextStyle(fontSize: 11, color: Colors.grey.shade600),
                          ),
                        if (citation.page != null) const SizedBox(width: 6),
                        Text(
                          '$scorePercent%',
                          style: TextStyle(
                            fontSize: 11,
                            color: Theme.of(context).colorScheme.primary,
                            fontWeight: FontWeight.w500,
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _showCitationDetail(Citation citation) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Row(
          children: [
            Container(
              width: 28,
              height: 28,
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.primary,
                borderRadius: BorderRadius.circular(6),
              ),
              child: Text(
                '${citation.index + 1}',
                style: TextStyle(
                  fontWeight: FontWeight.bold,
                  color: Theme.of(context).colorScheme.onPrimary,
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                citation.heading ?? '引用 ${citation.index + 1}',
                style: const TextStyle(fontSize: 16),
              ),
            ),
          ],
        ),
        content: SizedBox(
          width: 500,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (citation.page != null)
                Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: Row(
                    children: [
                      Icon(Icons.description_outlined, size: 16, color: Colors.grey.shade600),
                      const SizedBox(width: 4),
                      Text('第 ${citation.page} 页',
                          style: TextStyle(color: Colors.grey.shade600, fontSize: 13)),
                      const Spacer(),
                      Text('相关度: ${(citation.score * 100).toStringAsFixed(1)}%',
                          style: TextStyle(
                            color: Theme.of(context).colorScheme.primary,
                            fontSize: 13,
                            fontWeight: FontWeight.w500,
                          )),
                    ],
                  ),
                ),
              const Divider(),
              const SizedBox(height: 8),
              ConstrainedBox(
                constraints: const BoxConstraints(maxHeight: 300),
                child: SingleChildScrollView(
                  child: Text(
                    citation.text,
                    style: const TextStyle(fontSize: 14, height: 1.6),
                  ),
                ),
              ),
            ],
          ),
        ),
        actions: [
          if (citation.page != null)
            TextButton.icon(
              onPressed: () {
                Navigator.pop(ctx);
                final ws = ref.read(selectedWorkspaceProvider);
                if (ws != null) {
                  Navigator.of(context).push(
                    MaterialPageRoute(
                      builder: (_) => PdfViewerPage(
                        workspaceId: ws.id,
                        documentId: citation.documentId,
                        title: citation.heading ?? '引用来源',
                        initialPage: citation.page,
                        highlightText: citation.text.length > 50
                            ? citation.text.substring(0, 50)
                            : citation.text,
                      ),
                    ),
                  );
                }
              },
              icon: const Icon(Icons.open_in_new, size: 16),
              label: const Text('查看原文'),
            ),
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('关闭'),
          ),
        ],
      ),
    );
  }

  Widget _buildStreamingBubble() {
    return Align(
      alignment: Alignment.centerLeft,
      child: Container(
        constraints:
            BoxConstraints(maxWidth: MediaQuery.of(context).size.width * 0.65),
        margin: const EdgeInsets.symmetric(vertical: 6),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              padding: const EdgeInsets.all(14),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(16),
              ),
              child: _streamingContent.isEmpty
                  ? Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Theme.of(context).colorScheme.primary,
                          ),
                        ),
                        const SizedBox(width: 8),
                        Text('思考中...', style: TextStyle(color: Colors.grey.shade500)),
                      ],
                    )
                  : MarkdownBody(
                      data: '$_streamingContent▌',
                      selectable: true,
                    ),
            ),
            if (_streamingCitations.isNotEmpty)
              _buildCitationCards(_streamingCitations),
          ],
        ),
      ),
    );
  }

  Widget _buildInputBar() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        border:
            Border(top: BorderSide(color: Colors.grey.shade200, width: 1)),
      ),
      child: Row(
        children: [
          Expanded(
            child: TextField(
              controller: _controller,
              minLines: 1,
              maxLines: 4,
              decoration: InputDecoration(
                hintText: '输入你的问题...',
                filled: true,
                fillColor:
                    Theme.of(context).colorScheme.surfaceContainerHighest,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(24),
                  borderSide: BorderSide.none,
                ),
                contentPadding:
                    const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
              ),
              onSubmitted: (_) => _sendMessage(),
            ),
          ),
          const SizedBox(width: 8),
          IconButton.filled(
            onPressed: _isSending ? null : _sendMessage,
            icon: _isSending
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2))
                : const Icon(Icons.send),
          ),
        ],
      ),
    );
  }
}
