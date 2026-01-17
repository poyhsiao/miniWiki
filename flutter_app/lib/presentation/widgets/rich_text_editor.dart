import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

class RichTextEditor extends StatefulWidget {
  final Map<String, dynamic> initialContent;
  final void Function(Map<String, dynamic> content) onContentChanged;

  const RichTextEditor({
    required this.initialContent,
    required this.onContentChanged,
    super.key,
  });

  @override
  State<RichTextEditor> createState() => _RichTextEditorState();

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<Map<String, dynamic>>(
        'initialContent', initialContent));
    properties.add(
        ObjectFlagProperty<void Function(Map<String, dynamic> content)>.has(
            'onContentChanged', onContentChanged));
  }
}

class _RichTextEditorState extends State<RichTextEditor> {
  final TextEditingController _controller = TextEditingController();
  final FocusNode _focusNode = FocusNode();
  final ScrollController _scrollController = ScrollController();

  @override
  void initState() {
    super.initState();
    _initializeContent();
  }

  @override
  void didUpdateWidget(covariant RichTextEditor oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.initialContent != widget.initialContent) {
      _controller.text = _extractText(widget.initialContent);
    }
  }

  void _initializeContent() {
    if (widget.initialContent.isNotEmpty) {
      final text = _extractText(widget.initialContent);
      _controller.text = text;
    }
  }

  String _extractText(Map<String, dynamic> content) {
    if (content['ops'] != null && content['ops'] is List) {
      final buffer = StringBuffer();
      final ops = content['ops'] as List;
      for (final op in ops) {
        if (op is Map<String, dynamic> &&
            op['insert'] != null &&
            op['insert'] is String) {
          buffer.write(op['insert']);
        }
      }
      return buffer.toString();
    }
    return '';
  }

  void _onContentChanged() {
    final content = <String, dynamic>{
      'ops': [
        {'insert': _controller.text}
      ],
    };
    widget.onContentChanged(content);
  }

  @override
  void dispose() {
    _controller.dispose();
    _focusNode.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Column(
        children: [
          _buildToolbar(context),
          Expanded(
            child: SingleChildScrollView(
              controller: _scrollController,
              padding: const EdgeInsets.all(16),
              child: TextField(
                controller: _controller,
                focusNode: _focusNode,
                maxLines: null,
                decoration: const InputDecoration(
                  hintText: 'Start writing...',
                  border: InputBorder.none,
                  isDense: true,
                ),
                style: const TextStyle(fontSize: 16, height: 1.5),
                onChanged: (_) => _onContentChanged(),
              ),
            ),
          ),
        ],
      );

  Widget _buildToolbar(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.surfaceContainerHighest,
          borderRadius: const BorderRadius.vertical(bottom: Radius.circular(8)),
        ),
        child: Wrap(
          spacing: 4,
          children: [
            _ToolbarButton(
              icon: Icons.format_bold,
              tooltip: 'Bold',
              onPressed: () => _formatText('bold'),
            ),
            _ToolbarButton(
              icon: Icons.format_italic,
              tooltip: 'Italic',
              onPressed: () => _formatText('italic'),
            ),
            _ToolbarButton(
              icon: Icons.format_underline,
              tooltip: 'Underline',
              onPressed: () => _formatText('underline'),
            ),
            _ToolbarButton(
              icon: Icons.strikethrough_s,
              tooltip: 'Strikethrough',
              onPressed: () => _formatText('strike'),
            ),
            const VerticalDivider(width: 16),
            _ToolbarButton(
              icon: Icons.format_list_bulleted,
              tooltip: 'Bullet List',
              onPressed: () => _insertList('bullet'),
            ),
            _ToolbarButton(
              icon: Icons.format_list_numbered,
              tooltip: 'Numbered List',
              onPressed: () => _insertList('numbered'),
            ),
            const VerticalDivider(width: 16),
            _ToolbarButton(
              icon: Icons.format_quote,
              tooltip: 'Quote',
              onPressed: () => _formatBlock('quote'),
            ),
            _ToolbarButton(
              icon: Icons.code,
              tooltip: 'Code Block',
              onPressed: () => _formatBlock('code'),
            ),
            _ToolbarButton(
              icon: Icons.link,
              tooltip: 'Link',
              onPressed: _insertLink,
            ),
          ],
        ),
      );

  void _formatText(String format) {
    // Placeholder for rich text formatting
    // Would need to integrate with a proper rich text editor
  }

  void _insertList(String type) {
    // Placeholder for list insertion
  }

  void _formatBlock(String type) {
    // Placeholder for block formatting
  }

  void _insertLink() {
    // Placeholder for link insertion
  }
}

class _ToolbarButton extends StatelessWidget {
  final IconData icon;
  final String tooltip;
  final VoidCallback onPressed;

  const _ToolbarButton({
    required this.icon,
    required this.tooltip,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) => IconButton(
        icon: Icon(icon, size: 20),
        tooltip: tooltip,
        onPressed: onPressed,
        style: IconButton.styleFrom(
          minimumSize: const Size(36, 36),
        ),
      );

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<IconData>('icon', icon));
    properties.add(StringProperty('tooltip', tooltip));
    properties
        .add(ObjectFlagProperty<VoidCallback>.has('onPressed', onPressed));
  }
}
