import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

class AIIconHelper {
  static final Map<String, String> _cache = {};

  static String? getIconAsset(String name) {
    if (_cache.containsKey(name)) return _cache[name];

    final lower = name.toLowerCase();
    String? result;

    for (final entry in _matchers) {
      if (entry.pattern.hasMatch(lower)) {
        result = 'assets/icons/${entry.asset}';
        break;
      }
    }

    if (result != null) _cache[name] = result;
    return result;
  }

  static Widget buildIcon(String name, {double size = 24, Color? color}) {
    final asset = getIconAsset(name);
    if (asset != null) {
      final isMonochrome = !asset.contains('-color');
      return SvgPicture.asset(
        asset,
        width: size,
        height: size,
        colorFilter: isMonochrome && color != null
            ? ColorFilter.mode(color, BlendMode.srcIn)
            : null,
        placeholderBuilder: (_) => _buildFallbackIcon(name, size: size),
      );
    }
    return _buildFallbackIcon(name, size: size);
  }

  static Widget buildProviderAvatar(String providerOrModel,
      {double radius = 18, bool isDefault = false}) {
    final asset = getIconAsset(providerOrModel);
    if (asset != null) {
      return CircleAvatar(
        radius: radius,
        backgroundColor: isDefault ? Colors.green.shade50 : Colors.grey.shade100,
        child: Padding(
          padding: EdgeInsets.all(radius * 0.3),
          child: SvgPicture.asset(
            asset,
            width: radius * 1.2,
            height: radius * 1.2,
            placeholderBuilder: (_) =>
                _buildFallbackIcon(providerOrModel, size: radius),
          ),
        ),
      );
    }
    return CircleAvatar(
      radius: radius,
      backgroundColor: isDefault ? Colors.green.shade100 : Colors.grey.shade100,
      child: _buildFallbackIcon(providerOrModel, size: radius),
    );
  }

  static Widget _buildFallbackIcon(String name, {double size = 24}) {
    final letter = name.trim().isNotEmpty
        ? name.trim()[0].toUpperCase()
        : 'A';
    final color = _colorForName(name);
    return Container(
      width: size,
      height: size,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(size * 0.3),
      ),
      child: Text(
        letter,
        style: TextStyle(
          fontSize: size * 0.5,
          fontWeight: FontWeight.w600,
          color: color,
        ),
      ),
    );
  }

  static Color _colorForName(String name) {
    final hash = name.hashCode;
    final colors = [
      Colors.blue,
      Colors.purple,
      Colors.teal,
      Colors.orange,
      Colors.indigo,
      Colors.pink,
      Colors.cyan,
      Colors.deepOrange,
    ];
    return colors[hash.abs() % colors.length];
  }
}

class _IconMatcher {
  final RegExp pattern;
  final String asset;
  const _IconMatcher(this.pattern, this.asset);
}

final List<_IconMatcher> _matchers = [
  _IconMatcher(RegExp(r'(gpt|openai|o\d)'), 'openai.svg'),
  _IconMatcher(RegExp(r'claude'), 'claude-color.svg'),
  _IconMatcher(RegExp(r'anthropic'), 'anthropic.svg'),
  _IconMatcher(RegExp(r'gemini'), 'gemini-color.svg'),
  _IconMatcher(RegExp(r'gemma'), 'gemma-color.svg'),
  _IconMatcher(RegExp(r'google'), 'google-color.svg'),
  _IconMatcher(RegExp(r'deepseek'), 'deepseek-color.svg'),
  _IconMatcher(RegExp(r'grok'), 'grok.svg'),
  _IconMatcher(RegExp(r'qwen|qwq|qvq'), 'qwen-color.svg'),
  _IconMatcher(RegExp(r'doubao'), 'doubao-color.svg'),
  _IconMatcher(RegExp(r'zhipu|智谱|glm'), 'zhipu-color.svg'),
  _IconMatcher(RegExp(r'mistral'), 'mistral-color.svg'),
  _IconMatcher(RegExp(r'meta\b|(?<!o)llama'), 'meta-color.svg'),
  _IconMatcher(RegExp(r'hunyuan|tencent'), 'hunyuan-color.svg'),
  _IconMatcher(RegExp(r'perplexity'), 'perplexity-color.svg'),
  _IconMatcher(RegExp(r'ollama'), 'ollama.svg'),
  _IconMatcher(RegExp(r'kimi'), 'kimi-color.svg'),
  _IconMatcher(RegExp(r'moonshot|月之暗面'), 'moonshot.svg'),
  _IconMatcher(RegExp(r'minimax'), 'minimax-color.svg'),
  _IconMatcher(RegExp(r'silicon|硅基'), 'siliconflow.svg'),
  _IconMatcher(RegExp(r'openrouter'), 'openrouter.svg'),
  _IconMatcher(RegExp(r'nvidia'), 'nvidia-color.svg'),
  _IconMatcher(RegExp(r'xai'), 'xai.svg'),
  _IconMatcher(RegExp(r'cohere|command-.+'), 'cohere-color.svg'),
  _IconMatcher(RegExp(r'cloudflare'), 'cloudflare-color.svg'),
  _IconMatcher(RegExp(r'step|阶跃'), 'stepfun-color.svg'),
  _IconMatcher(RegExp(r'groq'), 'groq.svg'),
  _IconMatcher(RegExp(r'github'), 'github.svg'),
];
