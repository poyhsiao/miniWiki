/// Role enum representing user roles in a space
///
/// Each role has a specific level and associated permissions.
/// Roles are ordered by privilege level: Owner > Editor > Commenter > Viewer
enum Role {
  owner('Owner', 100),
  editor('Editor', 75),
  commenter('Commenter', 50),
  viewer('Viewer', 25);

  final String displayName;
  final int level;

  const Role(this.displayName, this.level);

  bool hasPermission(Permission permission) {
    final result = permission.allowedRoles.contains(this);
    return result;
  }

  bool canAssignRole(Role targetRole) {
    if (this == Role.owner) {
      return true;
    }
    if (this == Role.editor) {
      return targetRole.level < level && targetRole != Role.owner;
    }
    return false;
  }

  Set<ActionType> get allowedActions => ActionType.values.where((action) => action.allowedRoles.contains(this)).toSet();

  static Role? fromString(String role) {
    switch (role.toLowerCase()) {
      case 'owner':
        return Role.owner;
      case 'editor':
        return Role.editor;
      case 'commenter':
        return Role.commenter;
      case 'viewer':
        return Role.viewer;
      default:
        return null;
    }
  }

  @override
  String toString() => displayName;
}

/// Permission enum representing granular permissions
///
/// Each permission specifies which roles are allowed to perform it.
enum Permission {
  viewDocuments([Role.owner, Role.editor, Role.commenter, Role.viewer]),
  createDocuments([Role.owner, Role.editor]),
  editDocuments([Role.owner, Role.editor]),
  deleteDocuments([Role.owner]),
  comment([Role.owner, Role.editor, Role.commenter]),
  share([Role.owner, Role.editor]),
  manageMembers([Role.owner, Role.editor]),
  manageRoles([Role.owner]),
  deleteSpace([Role.owner]);

  final List<Role> allowedRoles;

  const Permission(this.allowedRoles);

  bool isAllowed(Role role) => allowedRoles.contains(role);

  bool requiresHigherRoleThan(Role role) {
    final minRoleIndex =
        allowedRoles.map((r) => r.level).reduce((a, b) => a < b ? a : b);
    return role.level < minRoleIndex;
  }

  static Permission? fromString(String permission) {
    switch (permission.toLowerCase()) {
      case 'view':
      case 'view_documents':
        return Permission.viewDocuments;
      case 'create':
      case 'create_documents':
        return Permission.createDocuments;
      case 'edit':
      case 'edit_documents':
        return Permission.editDocuments;
      case 'delete':
      case 'delete_documents':
        return Permission.deleteDocuments;
      case 'comment':
        return Permission.comment;
      case 'share':
        return Permission.share;
      case 'manage_members':
        return Permission.manageMembers;
      case 'manage_roles':
        return Permission.manageRoles;
      case 'delete_space':
        return Permission.deleteSpace;
      default:
        return null;
    }
  }
}

/// ActionType enum representing high-level user actions
///
/// Actions are composed of one or more permissions and represent
/// common user workflows in the application.
enum ActionType {
  viewDocument([Role.owner, Role.editor, Role.commenter, Role.viewer]),
  createDocument([Role.owner, Role.editor]),
  editDocument([Role.owner, Role.editor]),
  deleteDocument([Role.owner]),
  comment([Role.owner, Role.editor, Role.commenter]),
  share([Role.owner, Role.editor]),
  manageMembers([Role.owner, Role.editor]),
  manageRoles([Role.owner]),
  deleteSpace([Role.owner]),
  inviteMember([Role.owner, Role.editor]),
  removeMember([Role.owner, Role.editor]),
  viewMembers([Role.owner, Role.editor, Role.commenter, Role.viewer]),
  exportDocument([Role.owner, Role.editor, Role.commenter, Role.viewer]),
  viewVersionHistory([Role.owner, Role.editor, Role.commenter, Role.viewer]),
  restoreVersion([Role.owner, Role.editor]);

  final List<Role> allowedRoles;

  const ActionType(this.allowedRoles);

  bool isAllowed(Role role) => allowedRoles.contains(role);

  bool requiresHigherRoleThan(Role role) {
    final minRoleIndex =
        allowedRoles.map((r) => r.level).reduce((a, b) => a < b ? a : b);
    return role.level < minRoleIndex;
  }

  Set<Permission> get requiredPermissions {
    switch (this) {
      case ActionType.viewDocument:
        return {Permission.viewDocuments};
      case ActionType.createDocument:
        return {Permission.createDocuments};
      case ActionType.editDocument:
        return {Permission.editDocuments};
      case ActionType.deleteDocument:
        return {Permission.deleteDocuments};
      case ActionType.comment:
        return {Permission.comment};
      case ActionType.share:
        return {Permission.share};
      case ActionType.manageMembers:
        return {Permission.manageMembers};
      case ActionType.manageRoles:
        return {Permission.manageRoles};
      case ActionType.deleteSpace:
        return {Permission.deleteSpace};
      case ActionType.inviteMember:
        return {Permission.manageMembers};
      case ActionType.removeMember:
        return {Permission.manageMembers};
      case ActionType.viewMembers:
        return {Permission.viewDocuments};
      case ActionType.exportDocument:
        return {Permission.viewDocuments};
      case ActionType.viewVersionHistory:
        return {Permission.viewDocuments};
      case ActionType.restoreVersion:
        return {Permission.editDocuments};
    }
  }
}

/// Helper extension to check multiple permissions at once
extension RolePermissionsExtension on Role {
  bool hasAllPermissions(Set<Permission> permissions) => permissions.every(hasPermission);

  bool hasAnyPermission(Set<Permission> permissions) => permissions.any(hasPermission);

  bool canPerformAction(ActionType action) => action.isAllowed(this);

  bool canPerformAllActions(Set<ActionType> actions) => actions.every(canPerformAction);
}

/// RBAC configuration constants
class RbacConfig {
  static const int maxMembersPerSpace = 1000;
  static const int maxInvitesPerDay = 50;
  static const List<Role> assignableRoles = [
    Role.owner,
    Role.editor,
    Role.commenter,
    Role.viewer
  ];
  static const Role defaultRole = Role.viewer;
  static const Map<Role, Set<Permission>> rolePermissions = {
    Role.owner: {
      Permission.viewDocuments,
      Permission.createDocuments,
      Permission.editDocuments,
      Permission.deleteDocuments,
      Permission.comment,
      Permission.share,
      Permission.manageMembers,
      Permission.manageRoles,
      Permission.deleteSpace,
    },
    Role.editor: {
      Permission.viewDocuments,
      Permission.createDocuments,
      Permission.editDocuments,
      Permission.comment,
      Permission.share,
      Permission.manageMembers,
    },
    Role.commenter: {
      Permission.viewDocuments,
      Permission.comment,
    },
    Role.viewer: {
      Permission.viewDocuments,
    },
  };

  static Set<Permission> getPermissionsForRole(Role role) => rolePermissions[role] ?? {};

  static bool canRoleBeModifiedBy(Role targetRole, Role modifierRole) {
    if (targetRole == Role.owner) {
      return false;
    }
    return modifierRole.level > targetRole.level;
  }

  static bool isValidRoleTransition(Role fromRole, Role toRole) {
    if (toRole == Role.owner) {
      return false;
    }
    return fromRole.level >= toRole.level;
  }
}
