import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/space_membership.dart';

void main() {
  group('SpaceMembership Entity Tests', () {
    test('SpaceMembership can be created with all fields', () {
      // Arrange & Act
      final membership = SpaceMembership(
        id: 'mem1',
        spaceId: 'space1',
        userId: 'user1',
        role: 'admin',
        joinedAt: DateTime(2024),
        invitedBy: 'inviter1',
      );

      // Assert
      expect(membership.id, 'mem1');
      expect(membership.spaceId, 'space1');
      expect(membership.userId, 'user1');
      expect(membership.role, 'admin');
      expect(membership.joinedAt, DateTime(2024));
      expect(membership.invitedBy, 'inviter1');
    });

    test('SpaceMembership with different roles', () {
      // Arrange & Act
      final adminMembership = SpaceMembership(
        id: 'mem1',
        spaceId: 'space1',
        userId: 'user1',
        role: 'admin',
        joinedAt: DateTime(2024),
        invitedBy: 'inviter1',
      );

      final editorMembership = SpaceMembership(
        id: 'mem2',
        spaceId: 'space1',
        userId: 'user2',
        role: 'editor',
        joinedAt: DateTime(2024),
        invitedBy: 'inviter1',
      );

      final viewerMembership = SpaceMembership(
        id: 'mem3',
        spaceId: 'space1',
        userId: 'user3',
        role: 'viewer',
        joinedAt: DateTime(2024),
        invitedBy: 'inviter1',
      );

      // Assert
      expect(adminMembership.role, 'admin');
      expect(editorMembership.role, 'editor');
      expect(viewerMembership.role, 'viewer');
    });

    test('SpaceMembership toJson creates correct JSON', () {
      // Arrange
      final membership = SpaceMembership(
        id: 'mem1',
        spaceId: 'space1',
        userId: 'user1',
        role: 'admin',
        joinedAt: DateTime(2024, 1, 1, 12),
        invitedBy: 'inviter1',
      );

      // Act
      final json = membership.toJson();

      // Assert
      expect(json['id'], 'mem1');
      expect(json['space_id'], 'space1');
      expect(json['user_id'], 'user1');
      expect(json['role'], 'admin');
      expect(json['joined_at'], '2024-01-01T12:00:00.000');
      expect(json['invited_by'], 'inviter1');
    });

    test('SpaceMembership fromJson creates instance correctly', () {
      // Arrange
      final json = {
        'id': 'mem1',
        'space_id': 'space1',
        'user_id': 'user1',
        'role': 'admin',
        'joined_at': '2024-01-01T12:00:00.000Z',
        'invited_by': 'inviter1',
      };

      // Act
      final membership = SpaceMembership.fromJson(json);

      // Assert
      expect(membership.id, 'mem1');
      expect(membership.spaceId, 'space1');
      expect(membership.userId, 'user1');
      expect(membership.role, 'admin');
      expect(membership.joinedAt, DateTime.utc(2024, 1, 1, 12));
      expect(membership.invitedBy, 'inviter1');
    });

    test('SpaceMembership equality works correctly', () {
      // Arrange
      final membership1 = SpaceMembership(
        id: 'mem1',
        spaceId: 'space1',
        userId: 'user1',
        role: 'admin',
        joinedAt: DateTime(2024),
        invitedBy: 'inviter1',
      );

      final membership2 = SpaceMembership(
        id: 'mem1',
        spaceId: 'space2',
        userId: 'user2',
        role: 'viewer',
        joinedAt: DateTime(2025),
        invitedBy: 'inviter2',
      );

      final membership3 = SpaceMembership(
        id: 'mem2',
        spaceId: 'space1',
        userId: 'user1',
        role: 'admin',
        joinedAt: DateTime(2024),
        invitedBy: 'inviter1',
      );

      // Assert
      expect(membership1, equals(membership2)); // Same ID
      expect(membership1, isNot(equals(membership3))); // Different ID
      expect(membership1.hashCode, equals(membership2.hashCode));
    });

    test('SpaceMembership can represent owner role', () {
      // Arrange & Act
      final ownerMembership = SpaceMembership(
        id: 'mem1',
        spaceId: 'space1',
        userId: 'owner1',
        role: 'owner',
        joinedAt: DateTime(2024),
        invitedBy: 'system',
      );

      // Assert
      expect(ownerMembership.role, 'owner');
    });

    test('SpaceMembership with different users in same space', () {
      // Arrange & Act
      final user1Membership = SpaceMembership(
        id: 'mem1',
        spaceId: 'space1',
        userId: 'user1',
        role: 'editor',
        joinedAt: DateTime(2024),
        invitedBy: 'inviter1',
      );

      final user2Membership = SpaceMembership(
        id: 'mem2',
        spaceId: 'space1',
        userId: 'user2',
        role: 'viewer',
        joinedAt: DateTime(2024, 2),
        invitedBy: 'inviter1',
      );

      // Assert
      expect(user1Membership.spaceId, user2Membership.spaceId);
      expect(user1Membership.userId, isNot(equals(user2Membership.userId)));
      expect(user1Membership.id, isNot(equals(user2Membership.id)));
    });

    test('SpaceMembership with same user in different spaces', () {
      // Arrange & Act
      final space1Membership = SpaceMembership(
        id: 'mem1',
        spaceId: 'space1',
        userId: 'user1',
        role: 'admin',
        joinedAt: DateTime(2024),
        invitedBy: 'inviter1',
      );

      final space2Membership = SpaceMembership(
        id: 'mem2',
        spaceId: 'space2',
        userId: 'user1',
        role: 'viewer',
        joinedAt: DateTime(2024, 2),
        invitedBy: 'inviter2',
      );

      // Assert
      expect(space1Membership.userId, space2Membership.userId);
      expect(space1Membership.spaceId, isNot(equals(space2Membership.spaceId)));
      expect(space1Membership.role, 'admin');
      expect(space2Membership.role, 'viewer');
    });
  });
}
