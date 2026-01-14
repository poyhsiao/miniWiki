import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/value_objects/role.dart';
import 'package:miniwiki/domain/entities/space_membership.dart';
import 'package:miniwiki/services/rbac_service.dart';
import 'package:miniwiki/domain/repositories/space_repository.dart';
import 'package:mocktail/mocktail.dart';

class MockSpaceRepository extends Mock implements SpaceRepository {}

void main() {
  group('RbacService', () {
    late RbacService rbacService;
    late MockSpaceRepository mockSpaceRepository;

    setUp(() {
      mockSpaceRepository = MockSpaceRepository();
      rbacService = RbacService(spaceRepository: mockSpaceRepository);
    });

    group('Role permissions', () {
      test('Owner role has all permissions', () {
        expect(Role.owner.hasPermission(Permission.viewDocuments), isTrue);
        expect(Role.owner.hasPermission(Permission.editDocuments), isTrue);
        expect(Role.owner.hasPermission(Permission.deleteDocuments), isTrue);
        expect(Role.owner.hasPermission(Permission.createDocuments), isTrue);
        expect(Role.owner.hasPermission(Permission.viewDocuments), isTrue);
        expect(Role.owner.hasPermission(Permission.comment), isTrue);
        expect(Role.owner.hasPermission(Permission.share), isTrue);
        expect(Role.owner.hasPermission(Permission.manageRoles), isTrue);
        expect(Role.owner.hasPermission(Permission.deleteSpace), isTrue);
      });

      test('Editor role has edit and create permissions but not delete', () {
        expect(Role.editor.hasPermission(Permission.manageMembers), isTrue);
        expect(Role.editor.hasPermission(Permission.editDocuments), isTrue);
        expect(Role.editor.hasPermission(Permission.deleteDocuments), isFalse);
        expect(Role.editor.hasPermission(Permission.createDocuments), isTrue);
        expect(Role.editor.hasPermission(Permission.viewDocuments), isTrue);
        expect(Role.editor.hasPermission(Permission.comment), isTrue);
        expect(Role.editor.hasPermission(Permission.share), isTrue);
        expect(Role.editor.hasPermission(Permission.manageRoles), isFalse);
        expect(Role.editor.hasPermission(Permission.deleteSpace), isFalse);
      });

      test('Commenter role has view and comment permissions only', () {
        expect(Role.commenter.hasPermission(Permission.manageMembers), isFalse);
        expect(Role.commenter.hasPermission(Permission.editDocuments), isFalse);
        expect(
            Role.commenter.hasPermission(Permission.deleteDocuments), isFalse);
        expect(
            Role.commenter.hasPermission(Permission.createDocuments), isFalse);
        expect(Role.commenter.hasPermission(Permission.viewDocuments), isTrue);
        expect(Role.commenter.hasPermission(Permission.comment), isTrue);
        expect(Role.commenter.hasPermission(Permission.share), isFalse);
        expect(Role.commenter.hasPermission(Permission.manageRoles), isFalse);
        expect(Role.commenter.hasPermission(Permission.deleteSpace), isFalse);
      });

      test('Viewer role has view-only permissions', () {
        expect(Role.viewer.hasPermission(Permission.manageMembers), isFalse);
        expect(Role.viewer.hasPermission(Permission.editDocuments), isFalse);
        expect(Role.viewer.hasPermission(Permission.deleteDocuments), isFalse);
        expect(Role.viewer.hasPermission(Permission.createDocuments), isFalse);
        expect(Role.viewer.hasPermission(Permission.viewDocuments), isTrue);
        expect(Role.viewer.hasPermission(Permission.comment), isFalse);
        expect(Role.viewer.hasPermission(Permission.share), isFalse);
        expect(Role.viewer.hasPermission(Permission.manageRoles), isFalse);
        expect(Role.viewer.hasPermission(Permission.deleteSpace), isFalse);
      });
    });

    group('Role hierarchy', () {
      test('Owner is highest role', () {
        expect(Role.owner.level, greaterThan(Role.editor.level));
        expect(Role.owner.level, greaterThan(Role.commenter.level));
        expect(Role.owner.level, greaterThan(Role.viewer.level));
      });

      test('Editor is higher than commenter', () {
        expect(Role.editor.level, greaterThan(Role.commenter.level));
        expect(Role.editor.level, greaterThan(Role.viewer.level));
      });

      test('Commenter is higher than viewer', () {
        expect(Role.commenter.level, greaterThan(Role.viewer.level));
      });
    });

    group('canAssignRole', () {
      test('Owner can assign any role', () {
        expect(Role.owner.canAssignRole(Role.owner), isTrue);
        expect(Role.owner.canAssignRole(Role.editor), isTrue);
        expect(Role.owner.canAssignRole(Role.commenter), isTrue);
        expect(Role.owner.canAssignRole(Role.viewer), isTrue);
      });

      test('Editor cannot assign owner or editor roles', () {
        expect(Role.editor.canAssignRole(Role.owner), isFalse);
        expect(Role.editor.canAssignRole(Role.editor), isFalse);
        expect(Role.editor.canAssignRole(Role.commenter), isTrue);
        expect(Role.editor.canAssignRole(Role.viewer), isTrue);
      });

      test('Commenter cannot assign any roles', () {
        final roles = RbacService.getAssignableRoles(Role.commenter);
        expect(roles, isEmpty);
      });

      test('Viewer cannot assign any roles', () {
        expect(Role.viewer.canAssignRole(Role.owner), isFalse);
        expect(Role.viewer.canAssignRole(Role.editor), isFalse);
        expect(Role.viewer.canAssignRole(Role.commenter), isFalse);
        expect(Role.viewer.canAssignRole(Role.viewer), isFalse);
      });
    });

    group('getHighestRole', () {
      test('Returns highest role from list', () {
        final roles = [Role.viewer, Role.commenter, Role.editor, Role.owner];
        expect(RbacService.getHighestRole(roles), equals(Role.owner));
      });

      test('Returns role when single role in list', () {
        expect(RbacService.getHighestRole([Role.editor]), equals(Role.editor));
      });

      test('Returns null for empty list', () {
        expect(RbacService.getHighestRole([]), isNull);
      });
    });

    group('canAccessSpace', () {
      const spaceId = 'space-1';
      const userId = 'user-1';

      test('returns true when user is space member', () async {
        final membership = SpaceMembership(
          id: 'membership-1',
          spaceId: spaceId,
          userId: userId,
          role: 'editor',
          joinedAt: DateTime.now(),
          invitedBy: 'user-0',
        );
        when(() => mockSpaceRepository.listMembers(spaceId))
            .thenAnswer((_) async => [membership]);

        final result = await rbacService.canAccessSpace(spaceId, userId);

        expect(result, isTrue);
      });

      test('returns false when user is not space member', () async {
        when(() => mockSpaceRepository.listMembers(spaceId))
            .thenAnswer((_) async => []);

        final result = await rbacService.canAccessSpace(spaceId, userId);

        expect(result, isFalse);
      });

      test('throws exception for empty space ID', () async {
        expect(
          () => rbacService.canAccessSpace('', userId),
          throwsArgumentError,
        );
      });

      test('throws exception for empty user ID', () async {
        expect(
          () => rbacService.canAccessSpace(spaceId, ''),
          throwsArgumentError,
        );
      });
    });

    group('hasPermissionForSpace', () {
      const spaceId = 'space-1';
      const userId = 'user-1';

      test('returns true when user has required permission', () async {
        final membership = SpaceMembership(
          id: 'membership-1',
          spaceId: spaceId,
          userId: userId,
          role: 'editor',
          joinedAt: DateTime.now(),
          invitedBy: 'user-0',
        );
        when(() => mockSpaceRepository.listMembers(spaceId))
            .thenAnswer((_) async => [membership]);

        final result = await rbacService.hasPermissionForSpace(
          spaceId,
          userId,
          Permission.editDocuments,
        );

        expect(result, isTrue);
      });

      test('returns false when user lacks required permission', () async {
        final membership = SpaceMembership(
          id: 'membership-1',
          spaceId: spaceId,
          userId: userId,
          role: 'viewer',
          joinedAt: DateTime.now(),
          invitedBy: 'user-0',
        );
        when(() => mockSpaceRepository.listMembers(spaceId))
            .thenAnswer((_) async => [membership]);

        final result = await rbacService.hasPermissionForSpace(
          spaceId,
          userId,
          Permission.editDocuments,
        );

        expect(result, isFalse);
      });

      test('throws exception for empty space ID', () async {
        expect(
          () => rbacService.hasPermissionForSpace(
            '',
            userId,
            Permission.editDocuments,
          ),
          throwsArgumentError,
        );
      });
    });

    group('canPerformAction', () {
      const spaceId = 'space-1';
      const userId = 'user-1';

      test('Owner can delete document', () async {
        final membership = SpaceMembership(
          id: 'membership-1',
          spaceId: spaceId,
          userId: userId,
          role: 'owner',
          joinedAt: DateTime.now(),
          invitedBy: 'user-0',
        );
        when(() => mockSpaceRepository.listMembers(spaceId))
            .thenAnswer((_) async => [membership]);

        final result = await rbacService.canPerformAction(
          spaceId,
          userId,
          ActionType.deleteDocument,
        );

        expect(result, isTrue);
      });

      test('Editor cannot delete document', () async {
        final membership = SpaceMembership(
          id: 'membership-1',
          spaceId: spaceId,
          userId: userId,
          role: 'editor',
          joinedAt: DateTime.now(),
          invitedBy: 'user-0',
        );
        when(() => mockSpaceRepository.listMembers(spaceId))
            .thenAnswer((_) async => [membership]);

        final result = await rbacService.canPerformAction(
          spaceId,
          userId,
          ActionType.deleteDocument,
        );

        expect(result, isFalse);
      });

      test('Viewer can view document', () async {
        final membership = SpaceMembership(
          id: 'membership-1',
          spaceId: spaceId,
          userId: userId,
          role: 'viewer',
          joinedAt: DateTime.now(),
          invitedBy: 'user-0',
        );
        when(() => mockSpaceRepository.listMembers(spaceId))
            .thenAnswer((_) async => [membership]);

        final result = await rbacService.canPerformAction(
          spaceId,
          userId,
          ActionType.viewDocument,
        );

        expect(result, isTrue);
      });

      test('Non-member cannot perform any actions', () async {
        when(() => mockSpaceRepository.listMembers(spaceId))
            .thenAnswer((_) async => []);

        final result = await rbacService.canPerformAction(
          spaceId,
          userId,
          ActionType.viewDocument,
        );

        expect(result, isFalse);
      });
    });

    group('getAllowedActions', () {
      test('Owner gets all actions', () {
        final owner = Role.owner;
        final allowedActions = owner.allowedActions;

        expect(allowedActions, contains(ActionType.viewDocument));
        expect(allowedActions, contains(ActionType.createDocument));
        expect(allowedActions, contains(ActionType.editDocument));
        expect(allowedActions, contains(ActionType.deleteDocument));
        expect(allowedActions, contains(ActionType.comment));
        expect(allowedActions, contains(ActionType.manageMembers));
        expect(allowedActions, contains(ActionType.share));
        expect(allowedActions, contains(ActionType.manageRoles));
      });

      test('Editor gets edit-related actions but not management', () {
        final editor = Role.editor;
        final allowedActions = editor.allowedActions;

        expect(allowedActions, contains(ActionType.viewDocument));
        expect(allowedActions, contains(ActionType.createDocument));
        expect(allowedActions, contains(ActionType.editDocument));
        expect(allowedActions.contains(ActionType.deleteDocument), isFalse);
        expect(allowedActions, contains(ActionType.comment));
        expect(allowedActions, contains(ActionType.manageMembers));
        expect(allowedActions, contains(ActionType.share));
        expect(allowedActions.contains(ActionType.manageRoles), isFalse);
      });

      test('Commenter gets limited actions', () {
        final commenter = Role.commenter;
        final allowedActions = commenter.allowedActions;

        expect(allowedActions, contains(ActionType.viewDocument));
        expect(allowedActions.contains(ActionType.createDocument), isFalse);
        expect(allowedActions.contains(ActionType.editDocument), isFalse);
        expect(allowedActions.contains(ActionType.deleteDocument), isFalse);
        expect(allowedActions, contains(ActionType.comment));
        expect(allowedActions.contains(ActionType.manageMembers), isFalse);
        expect(allowedActions.contains(ActionType.share), isFalse);
        expect(allowedActions.contains(ActionType.manageRoles), isFalse);
      });

      test('Viewer gets view-only actions', () {
        final viewer = Role.viewer;
        final allowedActions = viewer.allowedActions;

        expect(allowedActions, contains(ActionType.viewDocument));
        expect(allowedActions.contains(ActionType.createDocument), isFalse);
        expect(allowedActions.contains(ActionType.editDocument), isFalse);
        expect(allowedActions.contains(ActionType.deleteDocument), isFalse);
        expect(allowedActions.contains(ActionType.comment), isFalse);
        expect(allowedActions.contains(ActionType.manageMembers), isFalse);
        expect(allowedActions.contains(ActionType.share), isFalse);
        expect(allowedActions.contains(ActionType.manageRoles), isFalse);
      });
    });

    group('isValidRoleChange', () {
      test('Cannot change role to owner', () {
        expect(
          RbacService.isValidRoleChange(Role.editor, Role.owner, Role.owner),
          isFalse,
        );
      });

      test('Editor cannot assign owner role', () {
        expect(
          RbacService.isValidRoleChange(Role.viewer, Role.owner, Role.editor),
          isFalse,
        );
      });

      test('Owner cannot be demoted', () {
        expect(
          RbacService.isValidRoleChange(Role.owner, Role.editor, Role.owner),
          isFalse,
        );
      });

      test('Valid role change within hierarchy', () {
        expect(
          RbacService.isValidRoleChange(
              Role.viewer, Role.commenter, Role.owner),
          isTrue,
        );
      });
    });

    group('canManageUser', () {
      test('Owner can manage Editor', () {
        expect(RbacService.canManageUser(Role.owner, Role.editor), isTrue);
      });

      test('Editor can manage Commenter', () {
        expect(RbacService.canManageUser(Role.editor, Role.commenter), isTrue);
      });

      test('Editor cannot manage Owner', () {
        expect(RbacService.canManageUser(Role.editor, Role.owner), isFalse);
      });

      test('Same level cannot manage', () {
        expect(RbacService.canManageUser(Role.editor, Role.editor), isFalse);
      });
    });

    group('getAssignableRoles', () {
      test('Owner can assign all roles', () {
        final roles = RbacService.getAssignableRoles(Role.owner);
        expect(
            roles,
            containsAll(
                [Role.owner, Role.editor, Role.commenter, Role.viewer]));
      });

      test('Editor can assign Commenter and Viewer', () {
        final roles = RbacService.getAssignableRoles(Role.editor);
        expect(roles, contains(Role.commenter));
        expect(roles, contains(Role.viewer));
        expect(roles, isNot(contains(Role.owner)));
        expect(roles, isNot(contains(Role.editor)));
      });

      test('Commenter cannot assign any roles', () {
        print(
            'DEBUG_GETAssignable: Commenter.canAssignRole(owner) = ${Role.commenter.canAssignRole(Role.owner)}');
        print(
            'DEBUG_GETAssignable: Commenter.canAssignRole(editor) = ${Role.commenter.canAssignRole(Role.editor)}');
        print(
            'DEBUG_GETAssignable: Commenter.canAssignRole(commenter) = ${Role.commenter.canAssignRole(Role.commenter)}');
        print(
            'DEBUG_GETAssignable: Commenter.canAssignRole(viewer) = ${Role.commenter.canAssignRole(Role.viewer)}');

        final roles = RbacService.getAssignableRoles(Role.commenter);
        print('DEBUG_GETAssignable: getAssignableRoles(Commenter) = $roles');

        expect(roles, isEmpty);
      });
    });
  });
}
