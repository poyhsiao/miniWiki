import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/presentation/providers/space_provider.dart';

class SpaceDetailPage extends ConsumerWidget {
  final String spaceId;

  const SpaceDetailPage({required this.spaceId, super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final spaceState = ref.watch(spaceProvider);
    final spaceProviderNotifier = ref.read(spaceProvider.notifier);

    ref.listen<SpaceState>(spaceProvider, (previous, next) {
      if (next.selectedSpace == null && next.spaces.isNotEmpty) {
        final space = next.spaces.firstWhere((s) => s.id == spaceId, orElse: () => next.spaces.first);
        spaceProviderNotifier.selectSpace(space);
      }
    });

    final space = spaceState.selectedSpace ?? spaceState.spaces.firstWhere(
      (s) => s.id == spaceId,
      orElse: () => spaceState.spaces.first,
    );

    return Scaffold(
      appBar: AppBar(
        title: Text(space.name),
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            tooltip: 'New Document',
            onPressed: () => _showCreateDocumentDialog(context),
          ),
          IconButton(
            icon: const Icon(Icons.people),
            tooltip: 'Members',
            onPressed: () => Navigator.pushNamed(
              context,
              '/spaces/$spaceId/members',
            ),
          ),
          IconButton(
            icon: const Icon(Icons.settings),
            tooltip: 'Settings',
            onPressed: () => Navigator.pushNamed(
              context,
              '/spaces/$spaceId/settings',
              arguments: space,
            ),
          ),
        ],
      ),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (space.description != null)
            Padding(
              padding: const EdgeInsets.all(16),
              child: Text(
                space.description!,
                style: TextStyle(color: Colors.grey[600]),
              ),
            ),
          Expanded(
            child: _buildDocumentList(context, spaceId),
          ),
        ],
      ),
    );
  }

  Widget _buildDocumentList(BuildContext context, String spaceId) =>
      FutureBuilder<List<dynamic>>(
        future: Future.value([]),
      builder: (context, snapshot) {
        if (snapshot.connectionState == ConnectionState.waiting) {
          return const Center(child: CircularProgressIndicator());
        }

        return Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(Icons.note_alt_outlined, size: 64, color: Colors.grey[400]),
              const SizedBox(height: 16),
              Text(
                'No documents yet',
                style: TextStyle(fontSize: 18, color: Colors.grey[600]),
              ),
              const SizedBox(height: 8),
              ElevatedButton.icon(
                onPressed: () => _showCreateDocumentDialog(context),
                icon: const Icon(Icons.add),
                label: const Text('Create your first document'),
              ),
            ],
          ),
        );
      },
    );

  void _showCreateDocumentDialog(BuildContext context) {
    final titleController = TextEditingController();

    showDialog(
      context: context,
      builder: (BuildContext context) => AlertDialog(
        title: const Text('New Document'),
        content: TextField(
          controller: titleController,
          decoration: const InputDecoration(
            labelText: 'Document Title',
            border: OutlineInputBorder(),
          ),
          autofocus: true,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              if (titleController.text.isNotEmpty) {
                Navigator.pop(context);
              }
            },
            child: const Text('Create'),
          ),
        ],
      ),
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('spaceId', spaceId));
  }
}
