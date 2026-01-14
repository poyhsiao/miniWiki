import 'package:miniwiki/core/network/network_error.dart';
import 'package:miniwiki/domain/entities/space.dart';
import 'package:miniwiki/domain/entities/space_membership.dart';
import 'package:miniwiki/domain/repositories/space_repository.dart';

class SpaceService {
  final SpaceRepository spaceRepository;

  SpaceService({required this.spaceRepository});

  static const List<String> validRoles = ['owner', 'editor', 'commenter', 'viewer'];

  Future<List<Space>> listSpaces() async {
    try {
      return await spaceRepository.listSpaces();
    } catch (e) {
      throw NetworkError('Failed to fetch spaces: $e');
    }
  }

  Future<Space> getSpace(String spaceId) async {
    try {
      return await spaceRepository.getSpace(spaceId);
    } catch (e) {
      throw NetworkError('Failed to fetch space: $e');
    }
  }

  Future<Space> createSpace({
    required String name,
    String? icon,
    String? description,
    bool isPublic = false,
  }) async {
    if (name.isEmpty) {
      throw ArgumentError('Space name cannot be empty');
    }
    if (name.length > 200) {
      throw ArgumentError('Space name cannot exceed 200 characters');
    }

    try {
      return await spaceRepository.createSpace(
        name: name,
        icon: icon,
        description: description,
        isPublic: isPublic,
      );
    } catch (e) {
      throw NetworkError('Failed to create space: $e');
    }
  }

  Future<Space> updateSpace(
    String spaceId, {
    String? name,
    String? icon,
    String? description,
    bool? isPublic,
  }) async {
    if (name != null && name.isEmpty) {
      throw ArgumentError('Space name cannot be empty');
    }
    if (name != null && name.length > 200) {
      throw ArgumentError('Space name cannot exceed 200 characters');
    }

    try {
      return await spaceRepository.updateSpace(
        spaceId,
        name: name,
        icon: icon,
        description: description,
        isPublic: isPublic,
      );
    } catch (e) {
      throw NetworkError('Failed to update space: $e');
    }
  }

  Future<void> deleteSpace(String spaceId) async {
    try {
      await spaceRepository.deleteSpace(spaceId);
    } catch (e) {
      throw NetworkError('Failed to delete space: $e');
    }
  }

  Future<List<SpaceMembership>> listMembers(String spaceId) async {
    try {
      return await spaceRepository.listMembers(spaceId);
    } catch (e) {
      throw NetworkError('Failed to fetch members: $e');
    }
  }

  Future<SpaceMembership> addMember(
    String spaceId,
    String userId,
    String role,
  ) async {
    if (!validRoles.contains(role)) {
      throw ArgumentError('Invalid role: $role. Must be one of: ${validRoles.join(', ')}');
    }

    try {
      return await spaceRepository.addMember(spaceId, userId, role);
    } catch (e) {
      throw NetworkError('Failed to add member: $e');
    }
  }

  Future<SpaceMembership> updateMemberRole(
    String spaceId,
    String userId,
    String role,
  ) async {
    if (!validRoles.contains(role)) {
      throw ArgumentError('Invalid role: $role. Must be one of: ${validRoles.join(', ')}');
    }

    try {
      return await spaceRepository.updateMemberRole(spaceId, userId, role);
    } catch (e) {
      throw NetworkError('Failed to update member role: $e');
    }
  }

  Future<void> removeMember(String spaceId, String userId) async {
    try {
      await spaceRepository.removeMember(spaceId, userId);
    } catch (e) {
      throw NetworkError('Failed to remove member: $e');
    }
  }
}
