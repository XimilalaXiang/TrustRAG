import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_markdown/flutter_markdown.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';

import '../../auth/providers/auth_provider.dart';
import '../../dashboard/providers/workspace_provider.dart';
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
          if (!trimmed.startsWith('data: ')) continue;
          final jsonStr = trimmed.substring(5).trim();
          if (jsonStr.isEmpty) continue;
          try {
            final eventType = currentEventType.isNotEmpty
                ? currentEventType
                : (jsonDecode(jsonStr) is Map
                    ? (jsonDecode(jsonStr)['type'] ?? '')
                    : '');
            final event = jsonDecode(jsonStr);
            if (eventType == 'message_start') {
              assistantId = event['message_id'] ?? '';
            } else if (eventType == 'text_delta') {
              setState(() {
                _streamingContent += event['delta'] ?? event['text'] ?? '';
              });
              _scrollToBottom();
            } else if (eventType == 'message_end') {
              final fullContent = _streamingContent;
              final aiMsg = ChatMessage(
                id: assistantId,
                role: 'assistant',
                content: fullContent,
                createdAt: DateTime.now(),
              );
              ref.read(messagesProvider.notifier).state = [
                ...ref.read(messagesProvider),
                aiMsg,
              ];
              setState(() {
                _streamingContent = '';
              });
            } else if (eventType == 'error') {
              final errMsg = event is String ? event : (event['message'] ?? event.toString());
              if (mounted) {
                ScaffoldMessenger.of(context)
                    .showSnackBar(SnackBar(content: Text('AI 错误: $errMsg')));
              }
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
            BoxConstraints(maxWidth: MediaQuery.of(context).size.width * 0.6),
        margin: const EdgeInsets.symmetric(vertical: 6),
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
    );
  }

  Widget _buildStreamingBubble() {
    return Align(
      alignment: Alignment.centerLeft,
      child: Container(
        constraints:
            BoxConstraints(maxWidth: MediaQuery.of(context).size.width * 0.6),
        margin: const EdgeInsets.symmetric(vertical: 6),
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.surfaceContainerHighest,
          borderRadius: BorderRadius.circular(16),
        ),
        child: MarkdownBody(
          data: '$_streamingContent▌',
          selectable: true,
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
