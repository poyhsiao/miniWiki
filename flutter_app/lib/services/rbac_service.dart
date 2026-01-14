import 'package:collection/collection.dart';
import 'package:miniwiki/domain/entities/space_membership.dart';
import 'package:miniwiki/domain/value_objects/role.dart';
import 'package:miniwiki/domain/repositories/space_repository.dart';

class RbacService {
  final SpaceRepository spaceRepository;

  RbacService({required this.spaceRepository});

  Future<SpaceMembership?> _getMembership(String spaceId, String userId) async {
    final memberships = await spaceRepository.listMembers(spaceId);
    return memberships.firstWhereOrNull((m) => m.userId == userId);
  }

  Role _parseRole(String roleString) {
    return Role.fromString(roleString) ?? Role.viewer;
  }

  Future<bool> canAccessSpace(String spaceId, String userId) async {
    if (spaceId.isEmpty) {
      throw ArgumentError('Space ID cannot be empty');
    }
    if (userId.isEmpty) {
      throw ArgumentError('User ID cannot be empty');
    }

    final membership = await _getMembership(spaceId, userId);
    return membership != null;
  }

  Future<bool> hasPermissionForSpace(
    String spaceId,
    String userId,
    Permission permission,
  ) async {
    if (spaceId.isEmpty) {
      throw ArgumentError('Space ID cannot be empty');
    }
    if (userId.isEmpty) {
      throw ArgumentError('User ID cannot be empty');
    }

    final membership = await _getMembership(spaceId, userId);
    if (membership == null) {
      return false;
    }

    final role = _parseRole(membership.role);
    return role.hasPermission(permission);
  }

  Future<bool> canPerformAction(
    String spaceId,
    String userId,
    ActionType action,
  ) async {
    if (spaceId.isEmpty) {
      throw ArgumentError('Space ID cannot be empty');
    }
    if (userId.isEmpty) {
      throw ArgumentError('User ID cannot be empty');
    }

    final membership = await _getMembership(spaceId, userId);
    if (membership == null) {
      return false;
    }

    final role = _parseRole(membership.role);
    return role.canPerformAction(action);
  }

  Future<Set<Permission>> getUserPermissions(
    String spaceId,
    String userId,
  ) async {
    if (spaceId.isEmpty) {
      throw ArgumentError('Space ID cannot be empty');
    }
    if (userId.isEmpty) {
      throw ArgumentError('User ID cannot be empty');
    }

    final membership = await _getMembership(spaceId, userId);
    if (membership == null) {
      return {};
    }

    final role = _parseRole(membership.role);
    return RbacConfig.getPermissionsForRole(role);
  }

  Future<Set<ActionType>> getUserActions(
    String spaceId,
    String userId,
  ) async {
    if (spaceId.isEmpty) {
      throw ArgumentError('Space ID cannot be empty');
    }
    if (userId.isEmpty) {
      throw ArgumentError('User ID cannot be empty');
    }

    final membership = await _getMembership(spaceId, userId);
    if (membership == null) {
      return {};
    }

    final role = _parseRole(membership.role);
    return role.allowedActions;
  }

  Future<bool> canAssignRole(
    String spaceId,
    String assignerId,
    Role targetRole,
  ) async {
    if (spaceId.isEmpty) {
      throw ArgumentError('Space ID cannot be empty');
    }
    if (assignerId.isEmpty) {
      throw ArgumentError('Assigner ID cannot be empty');
    }

    final membership = await _getMembership(spaceId, assignerId);
    if (membership == null) {
      return false;
    }

    final role = _parseRole(membership.role);
    return role.canAssignRole(targetRole);
  }

  Future<Role?> getUserRole(String spaceId, String userId) async {
    if (spaceId.isEmpty) {
      throw ArgumentError('Space ID cannot be empty');
    }
    if (userId.isEmpty) {
      throw ArgumentError('User ID cannot be empty');
    }

    final membership = await _getMembership(spaceId, userId);
    if (membership == null) {
      return null;
    }

    return _parseRole(membership.role);
  }

  static bool isValidRoleChange(
    Role currentRole,
    Role newRole,
    Role requesterRole,
  ) {
    if (newRole == Role.owner) {
      return false;
    }

    if (!requesterRole.canAssignRole(newRole)) {
      return false;
    }

    if (currentRole == Role.owner && newRole != Role.owner) {
      return false;
    }

    return true;
  }

  static Role? getHighestRole(List<Role> roles) {
    if (roles.isEmpty) {
      return null;
    }

    return roles.reduce((a, b) => a.level > b.level ? a : b);
  }

  static bool canManageUser(Role managerRole, Role targetRole) {
    return managerRole.level > targetRole.level;
  }

  static Set<Role> getAssignableRoles(Role assignerRole) {
    return Role.values.where((role) {
      return assignerRole.canAssignRole(role);
    }).toSet();
  }

  Future<bool> canDeleteSpace(String spaceId, String userId) async {
    return hasPermissionForSpace(spaceId, userId, Permission.deleteSpace);
  }

  Future<bool> canManageMembers(String spaceId, String userId) async {
    return hasPermissionForSpace(spaceId, userId, Permission.manageMembers);
  }

  Future<bool> canEditDocument(
    String spaceId,
    String documentId,
    String userId,
  ) async {
    return hasPermissionForSpace(spaceId, userId, Permission.editDocuments);
  }

  Future<bool> canDeleteDocument(
    String spaceId,
    String documentId,
    String userId,
  ) async {
    return hasPermissionForSpace(spaceId, userId, Permission.deleteDocuments);
  }

  Future<bool> canComment(String spaceId, String userId) async {
    return hasPermissionForSpace(spaceId, userId, Permission.comment);
  }

  Future<bool> canShare(String spaceId, String userId) async {
    return hasPermissionForSpace(spaceId, userId, Permission.share);
  }
}
