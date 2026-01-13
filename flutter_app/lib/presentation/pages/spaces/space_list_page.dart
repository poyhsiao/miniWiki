import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/space.dart';

class SpaceListPage extends ConsumerWidget {
  const SpaceListPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Spaces'),
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: () => _showCreateSpaceDialog(context),
          ),
        ],
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.folder_off, size: 64, color: Colors.grey[400]),
            const SizedBox(height: 16),
            Text(
              'No spaces yet',
              style: TextStyle(fontSize: 18, color: Colors.grey[600]),
            ),
            const SizedBox(height: 8),
            ElevatedButton.icon(
              onPressed: () => _showCreateSpaceDialog(context),
              icon: const Icon(Icons.add),
              label: const Text('Create your first space'),
            ),
          ],
        ),
      ),
    );
  }

  void _showCreateSpaceDialog(BuildContext context) {
    final nameController = TextEditingController();
    final descriptionController = TextEditingController();
    bool isPublic = false;

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Create New Space'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: nameController,
                decoration: const InputDecoration(
                  labelText: 'Space Name',
                  border: OutlineInputBorder(),
                ),
              ),
              const SizedBox(height: 16),
              TextField(
                controller: descriptionController,
                decoration: const InputDecoration(
                  labelText: 'Description (optional)',
                  border: OutlineInputBorder(),
                ),
                maxLines: 3,
              ),
              const SizedBox(height: 16),
              SwitchListTile(
                title: const Text('Public Space'),
                subtitle: const Text('Anyone with the link can view'),
                value: isPublic,
                onChanged: (value) => isPublic = value,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              if (nameController.text.isNotEmpty) {
                Navigator.pop(context);
              }
            },
            child: const Text('Create'),
          ),
        ],
      ),
    );
  }
}

class SpaceCard extends StatelessWidget {
  final Space space;

  const SpaceCard({super.key, required this.space});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: ListTile(
        leading: CircleAvatar(
          child: Text(space.icon ?? '📁'),
        ),
        title: Text(
          space.name,
          style: const TextStyle(fontWeight: FontWeight.w600),
        ),
        subtitle: Text(
          space.description ?? 'No description',
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        trailing: PopupMenuButton<String>(
          onSelected: (value) => _handleMenuAction(value, context),
          itemBuilder: (context) => [
            const PopupMenuItem(value: 'open', child: Text('Open')),
            const PopupMenuItem(value: 'settings', child: Text('Settings')),
            const PopupMenuItem(value: 'delete', child: Text('Delete')),
          ],
        ),
        onTap: () => Navigator.pushNamed(
          context,
          '/spaces/${space.id}',
          arguments: space,
        ),
      ),
    );
  }

  void _handleMenuAction(String value, BuildContext context) {
    switch (value) {
      case 'open':
        Navigator.pushNamed(context, '/spaces/${space.id}', arguments: space);
        break;
      case 'settings':
        Navigator.pushNamed(
          context,
          '/spaces/${space.id}/settings',
          arguments: space,
        );
        break;
      case 'delete':
        _showDeleteConfirmation(context);
        break;
    }
  }

  void _showDeleteConfirmation(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete Space'),
        content: Text('Are you sure you want to delete "${space.name}"?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            onPressed: () => Navigator.pop(context),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }
}
