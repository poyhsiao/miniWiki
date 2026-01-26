import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/value_objects/role.dart';

void main() {
  group('Role Enum Tests', () {
    test('Role has correct display names and levels', () {
      expect(Role.owner.displayName, 'Owner');
      expect(Role.owner.level, 100);

      expect(Role.editor.displayName, 'Editor');
      expect(Role.editor.level, 75);

      expect(Role.commenter.displayName, 'Commenter');
      expect(Role.commenter.level, 50);

      expect(Role.viewer.displayName, 'Viewer');
      expect(Role.viewer.level, 25);
    });

    test('Role fromString parses valid roles', () {
      expect(Role.fromString('owner'), Role.owner);
      expect(Role.fromString('editor'), Role.editor);
      expect(Role.fromString('commenter'), Role.commenter);
      expect(Role.fromString('viewer'), Role.viewer);
    });

    test('Role fromString is case insensitive', () {
      expect(Role.fromString('OWNER'), Role.owner);
      expect(Role.fromString('Owner'), Role.owner);
      expect(Role.fromString('EDITOR'), Role.editor);
    });

    test('Role fromString returns null for invalid role', () {
      expect(Role.fromString('invalid'), isNull);
      expect(Role.fromString('admin'), isNull);
      expect(Role.fromString(''), isNull);
    });

    test('Role toString returns displayName', () {
      expect(Role.owner.toString(), 'Owner');
      expect(Role.editor.toString(), 'Editor');
      expect(Role.commenter.toString(), 'Commenter');
      expect(Role.viewer.toString(), 'Viewer');
    });

    test('Role hasPermission works correctly', () {
      // Owner has all permissions
      expect(Role.owner.hasPermission(Permission.viewDocuments), true);
      expect(Role.owner.hasPermission(Permission.deleteSpace), true);

      // Editor has most permissions
      expect(Role.editor.hasPermission(Permission.viewDocuments), true);
      expect(Role.editor.hasPermission(Permission.createDocuments), true);
      expect(Role.editor.hasPermission(Permission.deleteSpace), false);

      // Commenter can view and comment
      expect(Role.commenter.hasPermission(Permission.viewDocuments), true);
      expect(Role.commenter.hasPermission(Permission.comment), true);
      expect(Role.commenter.hasPermission(Permission.createDocuments), false);

      // Viewer can only view
      expect(Role.viewer.hasPermission(Permission.viewDocuments), true);
      expect(Role.viewer.hasPermission(Permission.comment), false);
      expect(Role.viewer.hasPermission(Permission.createDocuments), false);
    });

    test('Role canAssignRole works correctly', () {
      // Owner can assign any role
      expect(Role.owner.canAssignRole(Role.owner), true);
      expect(Role.owner.canAssignRole(Role.editor), true);
      expect(Role.owner.canAssignRole(Role.commenter), true);
      expect(Role.owner.canAssignRole(Role.viewer), true);

      // Editor can assign lower roles but not owner
      expect(Role.editor.canAssignRole(Role.owner), false);
      expect(Role.editor.canAssignRole(Role.editor), false);
      expect(Role.editor.canAssignRole(Role.commenter), true);
      expect(Role.editor.canAssignRole(Role.viewer), true);

      // Commenter and Viewer cannot assign roles
      expect(Role.commenter.canAssignRole(Role.viewer), false);
      expect(Role.viewer.canAssignRole(Role.viewer), false);
    });

    test('Role allowedActions returns correct set', () {
      final ownerActions = Role.owner.allowedActions;
      expect(ownerActions.length, greaterThan(0));
      expect(ownerActions, contains(ActionType.viewDocument));
      expect(ownerActions, contains(ActionType.deleteDocument));
      expect(ownerActions, contains(ActionType.manageRoles));

      final viewerActions = Role.viewer.allowedActions;
      expect(viewerActions, contains(ActionType.viewDocument));
      expect(viewerActions, isNot(contains(ActionType.deleteDocument)));
    });
  });

  group('Permission Enum Tests', () {
    test('Permission has correct allowed roles', () {
      expect(Permission.viewDocuments.allowedRoles,
          containsAll([Role.owner, Role.editor, Role.commenter, Role.viewer]));
      expect(Permission.createDocuments.allowedRoles,
          containsAll([Role.owner, Role.editor]));
      expect(Permission.deleteDocuments.allowedRoles, equals([Role.owner]));
    });

    test('Permission isAllowed works correctly', () {
      expect(Permission.viewDocuments.isAllowed(Role.owner), true);
      expect(Permission.viewDocuments.isAllowed(Role.viewer), true);

      expect(Permission.deleteDocuments.isAllowed(Role.owner), true);
      expect(Permission.deleteDocuments.isAllowed(Role.editor), false);
    });

    test('Permission fromString parses valid permissions', () {
      expect(Permission.fromString('view'), Permission.viewDocuments);
      expect(Permission.fromString('view_documents'), Permission.viewDocuments);
      expect(Permission.fromString('create'), Permission.createDocuments);
      expect(Permission.fromString('create_documents'), Permission.createDocuments);
      expect(Permission.fromString('edit'), Permission.editDocuments);
      expect(Permission.fromString('delete'), Permission.deleteDocuments);
      expect(Permission.fromString('comment'), Permission.comment);
      expect(Permission.fromString('share'), Permission.share);
      expect(Permission.fromString('manage_members'), Permission.manageMembers);
      expect(Permission.fromString('manage_roles'), Permission.manageRoles);
      expect(Permission.fromString('delete_space'), Permission.deleteSpace);
    });

    test('Permission fromString is case insensitive', () {
      expect(Permission.fromString('VIEW'), Permission.viewDocuments);
      expect(Permission.fromString('View_Documents'), Permission.viewDocuments);
    });

    test('Permission fromString returns null for invalid permission', () {
      expect(Permission.fromString('invalid'), isNull);
      expect(Permission.fromString(''), isNull);
    });

    test('Permission requiresHigherRoleThan works correctly', () {
      expect(Permission.viewDocuments.requiresHigherRoleThan(Role.viewer), false);
      expect(Permission.deleteDocuments.requiresHigherRoleThan(Role.viewer), true);
      expect(Permission.manageRoles.requiresHigherRoleThan(Role.editor), true);
    });
  });

  group('ActionType Enum Tests', () {
    test('ActionType has correct allowed roles', () {
      expect(ActionType.viewDocument.allowedRoles,
          containsAll([Role.owner, Role.editor, Role.commenter, Role.viewer]));
      expect(ActionType.createDocument.allowedRoles,
          containsAll([Role.owner, Role.editor]));
      expect(ActionType.deleteSpace.allowedRoles, equals([Role.owner]));
    });

    test('ActionType isAllowed works correctly', () {
      expect(ActionType.viewDocument.isAllowed(Role.viewer), true);
      expect(ActionType.deleteDocument.isAllowed(Role.viewer), false);
      expect(ActionType.manageRoles.isAllowed(Role.owner), true);
      expect(ActionType.manageRoles.isAllowed(Role.editor), false);
    });

    test('ActionType requiredPermissions returns correct set', () {
      expect(ActionType.viewDocument.requiredPermissions,
          equals({Permission.viewDocuments}));
      expect(ActionType.createDocument.requiredPermissions,
          equals({Permission.createDocuments}));
      expect(ActionType.editDocument.requiredPermissions,
          equals({Permission.editDocuments}));
      expect(ActionType.comment.requiredPermissions,
          equals({Permission.comment}));
    });

    test('ActionType requiresHigherRoleThan works correctly', () {
      expect(ActionType.viewDocument.requiresHigherRoleThan(Role.viewer), false);
      expect(ActionType.deleteDocument.requiresHigherRoleThan(Role.viewer), true);
      expect(ActionType.manageRoles.requiresHigherRoleThan(Role.editor), true);
    });
  });

  group('RolePermissionsExtension Tests', () {
    test('hasAllPermissions works correctly', () {
      final ownerPerms = {
        Permission.viewDocuments,
        Permission.createDocuments,
        Permission.deleteDocuments,
      };

      expect(Role.owner.hasAllPermissions(ownerPerms), true);
      expect(Role.viewer.hasAllPermissions(ownerPerms), false);

      final viewerPerms = {Permission.viewDocuments};
      expect(Role.viewer.hasAllPermissions(viewerPerms), true);
    });

    test('hasAnyPermission works correctly', () {
      final mixedPerms = {
        Permission.viewDocuments,
        Permission.deleteDocuments,
      };

      expect(Role.viewer.hasAnyPermission(mixedPerms), true);
      expect(Role.viewer.hasAnyPermission({Permission.deleteDocuments}), false);
    });

    test('canPerformAction works correctly', () {
      expect(Role.viewer.canPerformAction(ActionType.viewDocument), true);
      expect(Role.viewer.canPerformAction(ActionType.createDocument), false);

      expect(Role.owner.canPerformAction(ActionType.deleteDocument), true);
      expect(Role.owner.canPerformAction(ActionType.manageRoles), true);
    });

    test('canPerformAllActions works correctly', () {
      final viewerActions = {
        ActionType.viewDocument,
        ActionType.viewMembers,
      };

      expect(Role.viewer.canPerformAllActions(viewerActions), true);

      final mixedActions = {
        ActionType.viewDocument,
        ActionType.deleteDocument,
      };

      expect(Role.viewer.canPerformAllActions(mixedActions), false);
      expect(Role.owner.canPerformAllActions(mixedActions), true);
    });
  });

  group('RbacConfig Tests', () {
    test('RbacConfig has correct constants', () {
      expect(RbacConfig.maxMembersPerSpace, 1000);
      expect(RbacConfig.maxInvitesPerDay, 50);
      expect(RbacConfig.defaultRole, Role.viewer);
    });

    test('RbacConfig assignableRoles contains all roles', () {
      expect(RbacConfig.assignableRoles.length, 4);
      expect(RbacConfig.assignableRoles, contains(Role.owner));
      expect(RbacConfig.assignableRoles, contains(Role.editor));
      expect(RbacConfig.assignableRoles, contains(Role.commenter));
      expect(RbacConfig.assignableRoles, contains(Role.viewer));
    });

    test('RbacConfig rolePermissions has correct mappings', () {
      expect(RbacConfig.rolePermissions[Role.owner]!.length, 9);
      expect(RbacConfig.rolePermissions[Role.editor]!.length, 6);
      expect(RbacConfig.rolePermissions[Role.commenter]!.length, 2);
      expect(RbacConfig.rolePermissions[Role.viewer]!.length, 1);
    });

    test('RbacConfig getPermissionsForRole returns correct set', () {
      final ownerPerms = RbacConfig.getPermissionsForRole(Role.owner);
      expect(ownerPerms.length, 9);
      expect(ownerPerms, contains(Permission.viewDocuments));
      expect(ownerPerms, contains(Permission.deleteSpace));

      final viewerPerms = RbacConfig.getPermissionsForRole(Role.viewer);
      expect(viewerPerms.length, 1);
      expect(viewerPerms, contains(Permission.viewDocuments));
    });

    test('RbacConfig canRoleBeModifiedBy works correctly', () {
      // Owner cannot be modified
      expect(RbacConfig.canRoleBeModifiedBy(Role.owner, Role.owner), false);

      // Editor can be modified by owner
      expect(RbacConfig.canRoleBeModifiedBy(Role.editor, Role.owner), true);

      // Editor cannot be modified by another editor (same level)
      expect(RbacConfig.canRoleBeModifiedBy(Role.editor, Role.editor), false);

      // Editor can be modified by owner only
      expect(RbacConfig.canRoleBeModifiedBy(Role.editor, Role.owner), true);
      expect(RbacConfig.canRoleBeModifiedBy(Role.editor, Role.editor), false);
    });

    test('RbacConfig isValidRoleTransition works correctly', () {
      // Cannot transition to owner
      expect(RbacConfig.isValidRoleTransition(Role.editor, Role.owner), false);
      expect(RbacConfig.isValidRoleTransition(Role.owner, Role.owner), false);

      // Can transition to same or lower level
      expect(RbacConfig.isValidRoleTransition(Role.editor, Role.editor), true);
      expect(RbacConfig.isValidRoleTransition(Role.editor, Role.commenter), true);
      expect(RbacConfig.isValidRoleTransition(Role.editor, Role.viewer), true);

      // Cannot transition to higher level
      expect(RbacConfig.isValidRoleTransition(Role.viewer, Role.editor), false);
      expect(RbacConfig.isValidRoleTransition(Role.commenter, Role.editor), false);
    });
  });

  group('Role Permission Matrix Tests', () {
    test('Owner has all permissions', () {
      final ownerPerms = RbacConfig.getPermissionsForRole(Role.owner);
      expect(ownerPerms, contains(Permission.viewDocuments));
      expect(ownerPerms, contains(Permission.createDocuments));
      expect(ownerPerms, contains(Permission.editDocuments));
      expect(ownerPerms, contains(Permission.deleteDocuments));
      expect(ownerPerms, contains(Permission.comment));
      expect(ownerPerms, contains(Permission.share));
      expect(ownerPerms, contains(Permission.manageMembers));
      expect(ownerPerms, contains(Permission.manageRoles));
      expect(ownerPerms, contains(Permission.deleteSpace));
    });

    test('Editor has correct permissions', () {
      final editorPerms = RbacConfig.getPermissionsForRole(Role.editor);
      expect(editorPerms, contains(Permission.viewDocuments));
      expect(editorPerms, contains(Permission.createDocuments));
      expect(editorPerms, contains(Permission.editDocuments));
      expect(editorPerms, contains(Permission.comment));
      expect(editorPerms, contains(Permission.share));
      expect(editorPerms, contains(Permission.manageMembers));
      expect(editorPerms, isNot(contains(Permission.deleteDocuments)));
      expect(editorPerms, isNot(contains(Permission.manageRoles)));
      expect(editorPerms, isNot(contains(Permission.deleteSpace)));
    });

    test('Commenter has correct permissions', () {
      final commenterPerms = RbacConfig.getPermissionsForRole(Role.commenter);
      expect(commenterPerms, contains(Permission.viewDocuments));
      expect(commenterPerms, contains(Permission.comment));
      expect(commenterPerms, isNot(contains(Permission.createDocuments)));
      expect(commenterPerms, isNot(contains(Permission.editDocuments)));
    });

    test('Viewer has minimal permissions', () {
      final viewerPerms = RbacConfig.getPermissionsForRole(Role.viewer);
      expect(viewerPerms, contains(Permission.viewDocuments));
      expect(viewerPerms.length, 1);
    });
  });

  group('Action Type Tests', () {
    test('AllActionTypes have requiredPermissions', () {
      for (final action in ActionType.values) {
        final perms = action.requiredPermissions;
        expect(perms, isNotEmpty,
            reason: '$action should have required permissions');
      }
    });

    test('InviteMember and RemoveMember require manageMembers', () {
      expect(ActionType.inviteMember.requiredPermissions,
          equals({Permission.manageMembers}));
      expect(ActionType.removeMember.requiredPermissions,
          equals({Permission.manageMembers}));
    });

    test('ExportDocument requires view permission', () {
      expect(ActionType.exportDocument.requiredPermissions,
          equals({Permission.viewDocuments}));
    });

    test('ViewVersionHistory requires view permission', () {
      expect(ActionType.viewVersionHistory.requiredPermissions,
          equals({Permission.viewDocuments}));
    });

    test('RestoreVersion requires edit permission', () {
      expect(ActionType.restoreVersion.requiredPermissions,
          equals({Permission.editDocuments}));
    });
  });
}
