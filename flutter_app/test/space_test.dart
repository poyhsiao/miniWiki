import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/space.dart';

void main() {
  group('Space Entity Tests', () {
    test('Space can be created with all fields', () {
      // Arrange & Act
      final space = Space(
        id: 'space1',
        ownerId: 'user1',
        name: 'Test Space',
        icon: '📁',
        description: 'A test space',
        createdAt: DateTime(2024),
        updatedAt: DateTime(2024),
      );

      // Assert
      expect(space.id, 'space1');
      expect(space.ownerId, 'user1');
      expect(space.name, 'Test Space');
      expect(space.icon, '📁');
      expect(space.description, 'A test space');
      expect(space.isPublic, false);
    });

    test('Space can be created with minimal fields', () {
      // Arrange & Act
      final space = Space(
        id: 'space1',
        ownerId: 'user1',
        name: 'Minimal Space',
        createdAt: DateTime(2024),
        updatedAt: DateTime(2024),
      );

      // Assert
      expect(space.id, 'space1');
      expect(space.name, 'Minimal Space');
      expect(space.icon, isNull);
      expect(space.description, isNull);
      expect(space.isPublic, false);
    });

    test('Space copyWith creates modified copy', () {
      // Arrange
      final original = Space(
        id: 'space1',
        ownerId: 'user1',
        name: 'Original Name',
        createdAt: DateTime(2024),
        updatedAt: DateTime(2024),
      );

      // Act
      final modified = original.copyWith(
        name: 'Modified Name',
        isPublic: true,
      );

      // Assert
      expect(modified.id, 'space1'); // Unchanged
      expect(modified.name, 'Modified Name');
      expect(modified.isPublic, true);
      expect(modified.ownerId, 'user1'); // Unchanged
    });

    test('Space toJson creates correct JSON', () {
      // Arrange
      final space = Space(
        id: 'space1',
        ownerId: 'user1',
        name: 'Test Space',
        icon: '📁',
        description: 'A test space',
        createdAt: DateTime(2024),
        updatedAt: DateTime(2024),
      );

      // Act
      final json = space.toJson();

      // Assert
      expect(json['id'], 'space1');
      expect(json['owner_id'], 'user1');
      expect(json['name'], 'Test Space');
      expect(json['icon'], '📁');
      expect(json['description'], 'A test space');
      expect(json['is_public'], false);
    });

    test('Space fromJson creates instance correctly', () {
      // Arrange
      final json = {
        'id': 'space1',
        'owner_id': 'user1',
        'name': 'Test Space',
        'icon': '📁',
        'description': 'A test space',
        'is_public': false,
        'created_at': '2024-01-01T00:00:00Z',
        'updated_at': '2024-01-01T00:00:00Z',
      };

      // Act
      final space = Space.fromJson(json);

      // Assert
      expect(space.id, 'space1');
      expect(space.ownerId, 'user1');
      expect(space.name, 'Test Space');
      expect(space.icon, '📁');
      expect(space.description, 'A test space');
    });

    test('Space fromJson handles null optional fields', () {
      // Arrange
      final json = {
        'id': 'space1',
        'owner_id': 'user1',
        'name': 'Test Space',
        'is_public': true,
      };

      // Act
      final space = Space.fromJson(json);

      // Assert
      expect(space.id, 'space1');
      expect(space.icon, isNull);
      expect(space.description, isNull);
    });

    test('Space equality works correctly', () {
      // Arrange
      final space1 = Space(
        id: 'space1',
        ownerId: 'user1',
        name: 'Test Space',
        createdAt: DateTime(2024),
        updatedAt: DateTime(2024),
      );

      final space2 = Space(
        id: 'space1',
        ownerId: 'user1',
        name: 'Test Space',
        createdAt: DateTime(2024),
        updatedAt: DateTime(2024),
      );

      final space3 = Space(
        id: 'space2',
        ownerId: 'user1',
        name: 'Different Space',
        createdAt: DateTime(2024),
        updatedAt: DateTime(2024),
      );

      // Assert
      expect(space1, equals(space2));
      expect(space1, isNot(equals(space3)));
      expect(space1.hashCode, equals(space2.hashCode));
    });

    test('Space toString returns formatted string', () {
      // Arrange & Act
      final space = Space(
        id: 'space1',
        ownerId: 'user1',
        name: 'Test Space',
        createdAt: DateTime(2024),
        updatedAt: DateTime(2024),
      );

      // Assert
      expect(space.toString(), 'Space(id: space1, name: Test Space)');
    });

    test('Space with public visibility', () {
      // Arrange & Act
      final space = Space(
        id: 'space1',
        ownerId: 'user1',
        name: 'Public Space',
        isPublic: true,
        createdAt: DateTime(2024),
        updatedAt: DateTime(2024),
      );

      // Assert
      expect(space.isPublic, true);
    });

    test('Space with emoji icon', () {
      // Arrange & Act
      final space = Space(
        id: 'space1',
        ownerId: 'user1',
        name: 'Emoji Space',
        icon: '🚀',
        createdAt: DateTime(2024),
        updatedAt: DateTime(2024),
      );

      // Assert
      expect(space.icon, '🚀');
    });
  });
}
