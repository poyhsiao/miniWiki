import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/space_membership.dart';
import 'package:miniwiki/presentation/providers/space_provider.dart';
import 'package:miniwiki/services/space_service.dart';

class MemberManagementPage extends ConsumerWidget {
  final String spaceId;

  const MemberManagementPage({super.key, required this.spaceId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final spaceState = ref.watch(spaceProvider);
    final members = spaceState.members;
    final isLoading = spaceState.isLoadingMembers;

    ref.listen<SpaceState>(spaceProvider, (previous, next) {
      if (next.members.isEmpty && !next.isLoadingMembers) {
        ref.read(spaceProvider.notifier).loadMembers(spaceId);
      }
    });

    return Scaffold(
      appBar: AppBar(
        title: const Text('Members'),
        actions: [
          IconButton(
            icon: const Icon(Icons.person_add),
            tooltip: 'Invite Member',
            onPressed: () => _showAddMemberDialog(context, ref),
          ),
        ],
      ),
      body: isLoading
          ? const Center(child: CircularProgressIndicator())
          : members.isEmpty
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.people_outline, size: 64, color: Colors.grey[400]),
                      const SizedBox(height: 16),
                      Text(
                        'No members yet',
                        style: TextStyle(fontSize: 18, color: Colors.grey[600]),
                      ),
                      const SizedBox(height: 8),
                      ElevatedButton.icon(
                        onPressed: () => _showAddMemberDialog(context, ref),
                        icon: const Icon(Icons.person_add),
                        label: const Text('Add the first member'),
                      ),
                    ],
                  ),
                )
              : ListView.builder(
                  padding: const EdgeInsets.all(8),
                  itemCount: members.length,
                  itemBuilder: (context, index) {
                    final member = members[index];
                    return Card(
                      child: ListTile(
                        leading: CircleAvatar(
                          child: Text(member.userId[0].toUpperCase()),
                        ),
                        title: Row(
                          children: [
                            Expanded(child: Text(member.userId)),
                            Container(
                              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                              decoration: BoxDecoration(
                                color: _getRoleColor(member.role).withOpacity(0.1),
                                borderRadius: BorderRadius.circular(4),
                              ),
                              child: Text(
                                member.role[0].toUpperCase() + member.role.substring(1),
                                style: TextStyle(
                                  color: _getRoleColor(member.role),
                                  fontSize: 12,
                                  fontWeight: FontWeight.w500,
                                ),
                              ),
                            ),
                          ],
                        ),
                        subtitle: Text('Joined ${_formatDate(member.joinedAt)}'),
                        trailing: PopupMenuButton<String>(
                          onSelected: (value) => _handleMenuAction(value, context, ref, member),
                          itemBuilder: (context) => [
                            const PopupMenuItem(
                              value: 'viewer',
                              child: Text('Set as Viewer'),
                            ),
                            const PopupMenuItem(
                              value: 'commenter',
                              child: Text('Set as Commenter'),
                            ),
                            const PopupMenuItem(
                              value: 'editor',
                              child: Text('Set as Editor'),
                            ),
                            if (member.role != 'owner')
                              const PopupMenuItem(
                                value: 'remove',
                                child: Text(
                                  'Remove Member',
                                  style: TextStyle(color: Colors.red),
                                ),
                              ),
                          ],
                        ),
                      ),
                    );
                  },
                ),
    );
  }

  Color _getRoleColor(String role) {
    switch (role) {
      case 'owner':
        return Colors.purple;
      case 'editor':
        return Colors.blue;
      case 'commenter':
        return Colors.orange;
      case 'viewer':
        return Colors.green;
      default:
        return Colors.grey;
    }
  }

  String _formatDate(DateTime date) {
    return '${date.year}/${date.month.toString().padLeft(2, '0')}/${date.day.toString().padLeft(2, '0')}';
  }

  void _handleMenuAction(
    String value,
    BuildContext context,
    WidgetRef ref,
    SpaceMembership member,
  ) {
    switch (value) {
      case 'viewer':
      case 'commenter':
      case 'editor':
        ref.read(spaceProvider.notifier).updateMemberRole(
              spaceId,
              member.userId,
              value,
            );
        break;
      case 'remove':
        _showRemoveConfirmation(context, ref, member);
        break;
    }
  }

  void _showRemoveConfirmation(BuildContext context, WidgetRef ref, SpaceMembership member) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Remove Member'),
        content: Text('Are you sure you want to remove ${member.userId} from this space?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            onPressed: () async {
              Navigator.pop(context);
              await ref.read(spaceProvider.notifier).removeMember(spaceId, member.userId);
            },
            child: const Text('Remove'),
          ),
        ],
      ),
    );
  }

  void _showAddMemberDialog(BuildContext context, WidgetRef ref) {
    final userIdController = TextEditingController();
    String selectedRole = 'viewer';
    bool isLoading = false;

    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: const Text('Invite Member'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  controller: userIdController,
                  decoration: const InputDecoration(
                    labelText: 'User ID or Email',
                    border: OutlineInputBorder(),
                  ),
                  autofocus: true,
                ),
                const SizedBox(height: 16),
                DropdownButtonFormField<String>(
                  value: selectedRole,
                  decoration: const InputDecoration(
                    labelText: 'Role',
                    border: OutlineInputBorder(),
                  ),
                  items: SpaceService.validRoles
                      .where((role) => role != 'owner')
                      .map((role) => DropdownMenuItem(
                            value: role,
                            child: Text(
                              role[0].toUpperCase() + role.substring(1),
                            ),
                          ))
                      .toList(),
                  onChanged: (value) => setState(() => selectedRole = value ?? 'viewer'),
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
              onPressed: isLoading || userIdController.text.isEmpty
                  ? null
                  : () async {
                      setState(() => isLoading = true);
                      try {
                        await ref.read(spaceProvider.notifier).addMember(
                              spaceId,
                              userIdController.text,
                              selectedRole,
                            );
                        if (context.mounted) {
                          Navigator.pop(context);
                        }
                      } catch (e) {
                        setState(() => isLoading = false);
                        if (context.mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text('Failed to add member: $e')),
                          );
                        }
                      }
                    },
              child: isLoading
                  ? const SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Text('Invite'),
            ),
          ],
        ),
      ),
    );
  }
}
