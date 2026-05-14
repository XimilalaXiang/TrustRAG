import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:syncfusion_flutter_pdfviewer/pdfviewer.dart';

import '../../../core/api/api_client.dart';
import '../../auth/providers/auth_provider.dart';

class PdfViewerPage extends ConsumerStatefulWidget {
  final String workspaceId;
  final String documentId;
  final String title;
  final int? initialPage;
  final String? highlightText;

  const PdfViewerPage({
    super.key,
    required this.workspaceId,
    required this.documentId,
    required this.title,
    this.initialPage,
    this.highlightText,
  });

  @override
  ConsumerState<PdfViewerPage> createState() => _PdfViewerPageState();
}

class _PdfViewerPageState extends ConsumerState<PdfViewerPage> {
  late PdfViewerController _pdfController;
  final TextEditingController _searchController = TextEditingController();
  PdfTextSearchResult? _searchResult;
  bool _isSearching = false;
  int _totalPages = 0;
  int _currentPage = 0;

  @override
  void initState() {
    super.initState();
    _pdfController = PdfViewerController();
    _initPdfUrl();
  }

  @override
  void dispose() {
    _pdfController.dispose();
    _searchController.dispose();
    _searchResult?.dispose();
    super.dispose();
  }

  String? _token;
  String _pdfUrl = '';

  Future<void> _initPdfUrl() async {
    final api = ref.read(apiClientProvider);
    final token = await ApiClient.getToken();
    setState(() {
      _token = token;
      _pdfUrl = '${api.baseUrl}/workspaces/${widget.workspaceId}/documents/${widget.documentId}/download';
    });
  }

  Map<String, String> get _headers {
    if (_token == null) return {};
    return {'Authorization': 'Bearer $_token'};
  }

  void _onDocumentLoaded(PdfDocumentLoadedDetails details) {
    setState(() {
      _totalPages = details.document.pages.count;
    });
    if (widget.initialPage != null && widget.initialPage! > 0) {
      _pdfController.jumpToPage(widget.initialPage!);
    }
    if (widget.highlightText != null && widget.highlightText!.isNotEmpty) {
      Future.delayed(const Duration(milliseconds: 500), () {
        _performSearch(widget.highlightText!);
      });
    }
  }

  void _onPageChanged(PdfPageChangedDetails details) {
    setState(() {
      _currentPage = details.newPageNumber;
    });
  }

  void _performSearch(String text) {
    if (text.isEmpty) return;
    _searchResult?.dispose();
    _searchResult = _pdfController.searchText(text);
    setState(() => _isSearching = true);
    _searchResult!.addListener(() {
      if (_searchResult!.hasResult) {
        setState(() {});
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(widget.title, style: const TextStyle(fontSize: 16)),
            if (_totalPages > 0)
              Text(
                '第 $_currentPage / $_totalPages 页',
                style: TextStyle(fontSize: 12, color: cs.onSurfaceVariant),
              ),
          ],
        ),
        actions: [
          IconButton(
            icon: Icon(_isSearching ? Icons.close : Icons.search),
            tooltip: _isSearching ? '关闭搜索' : '搜索',
            onPressed: () {
              if (_isSearching) {
                _searchResult?.clear();
                _searchController.clear();
                setState(() => _isSearching = false);
              } else {
                setState(() => _isSearching = true);
              }
            },
          ),
          IconButton(
            icon: const Icon(Icons.zoom_in),
            tooltip: '放大',
            onPressed: () {
              _pdfController.zoomLevel = (_pdfController.zoomLevel + 0.25).clamp(0.5, 3.0);
            },
          ),
          IconButton(
            icon: const Icon(Icons.zoom_out),
            tooltip: '缩小',
            onPressed: () {
              _pdfController.zoomLevel = (_pdfController.zoomLevel - 0.25).clamp(0.5, 3.0);
            },
          ),
        ],
      ),
      body: Column(
        children: [
          if (_isSearching)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              decoration: BoxDecoration(
                color: cs.surfaceContainerHighest,
                border: Border(bottom: BorderSide(color: cs.outline.withValues(alpha: 0.2))),
              ),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _searchController,
                      autofocus: true,
                      decoration: InputDecoration(
                        hintText: '在文档中搜索...',
                        isDense: true,
                        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                        border: OutlineInputBorder(borderRadius: BorderRadius.circular(8)),
                        suffixIcon: _searchResult != null && _searchResult!.totalInstanceCount > 0
                            ? Padding(
                                padding: const EdgeInsets.only(right: 8),
                                child: Center(
                                  widthFactor: 1,
                                  child: Text(
                                    '${_searchResult!.currentInstanceIndex}/${_searchResult!.totalInstanceCount}',
                                    style: TextStyle(fontSize: 12, color: cs.onSurfaceVariant),
                                  ),
                                ),
                              )
                            : null,
                      ),
                      onSubmitted: _performSearch,
                    ),
                  ),
                  const SizedBox(width: 8),
                  IconButton(
                    icon: const Icon(Icons.keyboard_arrow_up, size: 20),
                    tooltip: '上一个',
                    onPressed: _searchResult?.hasResult == true
                        ? () => _searchResult!.previousInstance()
                        : null,
                  ),
                  IconButton(
                    icon: const Icon(Icons.keyboard_arrow_down, size: 20),
                    tooltip: '下一个',
                    onPressed: _searchResult?.hasResult == true
                        ? () => _searchResult!.nextInstance()
                        : null,
                  ),
                ],
              ),
            ),
          Expanded(
            child: _pdfUrl.isEmpty
                ? const Center(child: CircularProgressIndicator())
                : SfPdfViewer.network(
              _pdfUrl,
              headers: _headers,
              controller: _pdfController,
              onDocumentLoaded: _onDocumentLoaded,
              onPageChanged: _onPageChanged,
              onDocumentLoadFailed: (PdfDocumentLoadFailedDetails details) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('PDF 加载失败: ${details.description}')),
                );
              },
              canShowScrollHead: true,
              canShowPaginationDialog: true,
              pageSpacing: 4,
            ),
          ),
        ],
      ),
    );
  }
}
