import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/document.dart';

void main() {
  group('Document Entity Tests', () {
    test('Document can be created with required fields', () {
      // Arrange & Act
      const document = Document(
        id: 'doc1',
        spaceId: 'space1',
        title: 'Test Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      // Assert
      expect(document.id, 'doc1');
      expect(document.spaceId, 'space1');
      expect(document.title, 'Test Document');
      expect(document.createdBy, 'user1');
      expect(document.lastEditedBy, 'user1');
      expect(document.parentId, isNull);
      expect(document.icon, isNull);
      expect(document.content, isEmpty);
      expect(document.contentSize, 0);
      expect(document.isArchived, false);
      expect(document.createdAt, isNull);
      expect(document.updatedAt, isNull);
      expect(document.isSynced, true);
      expect(document.isDirty, false);
      expect(document.lastSyncedAt, isNull);
    });

    test('Document can be created with all fields', () {
      // Arrange & Act
      final now = DateTime(2024);
      final document = Document(
        id: 'doc1',
        spaceId: 'space1',
        parentId: 'parent1',
        title: 'Test Document',
        icon: '📄',
        content: <String, dynamic>{'delta': <dynamic>[]},
        contentSize: 1024,
        createdBy: 'user1',
        lastEditedBy: 'user2',
        createdAt: now,
        updatedAt: now,
        lastSyncedAt: now,
      );

      // Assert
      expect(document.id, 'doc1');
      expect(document.parentId, 'parent1');
      expect(document.icon, '📄');
      expect(document.content, <String, dynamic>{'delta': <dynamic>[]});
      expect(document.contentSize, 1024);
      expect(document.isArchived, false);
      expect(document.createdAt, now);
      expect(document.updatedAt, now);
      expect(document.lastSyncedAt, now);
    });

    test('Document copyWith creates modified copy', () {
      // Arrange
      const original = Document(
        id: 'doc1',
        spaceId: 'space1',
        title: 'Original Title',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      // Act
      final modified = original.copyWith(
        title: 'Modified Title',
        isDirty: true,
        lastEditedBy: 'user2',
      );

      // Assert
      expect(modified.id, 'doc1'); // Unchanged
      expect(modified.title, 'Modified Title');
      expect(modified.isDirty, true);
      expect(modified.lastEditedBy, 'user2');
      expect(modified.spaceId, 'space1'); // Unchanged
    });

    test('Document toJson creates correct JSON with all fields', () {
      // Arrange
      final now = DateTime(2024, 1, 1, 12);
      final document = Document(
        id: 'doc1',
        spaceId: 'space1',
        parentId: 'parent1',
        title: 'Test Document',
        icon: '📄',
        content: <String, dynamic>{'delta': <dynamic>[]},
        contentSize: 1024,
        createdBy: 'user1',
        lastEditedBy: 'user2',
        createdAt: now,
        updatedAt: now,
        lastSyncedAt: now,
      );

      // Act
      final json = document.toJson();

      // Assert
      expect(json['id'], 'doc1');
      expect(json['space_id'], 'space1');
      expect(json['parent_id'], 'parent1');
      expect(json['title'], 'Test Document');
      expect(json['icon'], '📄');
      expect(json['content'], <String, dynamic>{'delta': <dynamic>[]});
      expect(json['content_size'], 1024);
      expect(json['is_archived'], false);
      expect(json['created_by'], 'user1');
      expect(json['last_edited_by'], 'user2');
      expect(json['created_at'], '2024-01-01T12:00:00.000');
      expect(json['updated_at'], '2024-01-01T12:00:00.000');
      expect(json['is_synced'], true);
      expect(json['is_dirty'], false);
      expect(json['last_synced_at'], '2024-01-01T12:00:00.000');
    });

    test('Document toJson excludes null optional fields', () {
      // Arrange
      const document = Document(
        id: 'doc1',
        spaceId: 'space1',
        title: 'Test Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      // Act
      final json = document.toJson();

      // Assert
      expect(json['id'], 'doc1');
      expect(json.containsKey('parent_id'), false);
      expect(json.containsKey('icon'), false);
      expect(json.containsKey('created_at'), false);
      expect(json.containsKey('updated_at'), false);
      expect(json.containsKey('last_synced_at'), false);
    });

    test('Document fromJson creates instance with all fields', () {
      // Arrange
      final json = {
        'id': 'doc1',
        'space_id': 'space1',
        'parent_id': 'parent1',
        'title': 'Test Document',
        'icon': '📄',
        'content': <String, dynamic>{'delta': <dynamic>[]},
        'content_size': 1024,
        'is_archived': false,
        'created_by': 'user1',
        'last_edited_by': 'user2',
        'created_at': '2024-01-01T12:00:00.000Z',
        'updated_at': '2024-01-01T12:00:00.000Z',
        'is_synced': true,
        'is_dirty': false,
        'last_synced_at': '2024-01-01T12:00:00.000Z',
      };

      // Act
      final document = Document.fromJson(json);

      // Assert
      expect(document.id, 'doc1');
      expect(document.spaceId, 'space1');
      expect(document.parentId, 'parent1');
      expect(document.title, 'Test Document');
      expect(document.icon, '📄');
      expect(document.contentSize, 1024);
      expect(document.isArchived, false);
      expect(document.createdBy, 'user1');
      expect(document.lastEditedBy, 'user2');
    });

    test('Document fromJson handles null optional fields', () {
      // Arrange
      final json = {
        'id': 'doc1',
        'space_id': 'space1',
        'title': 'Test Document',
        'created_by': 'user1',
        'last_edited_by': 'user1',
      };

      // Act
      final document = Document.fromJson(json);

      // Assert
      expect(document.parentId, isNull);
      expect(document.icon, isNull);
      expect(document.contentSize, 0); // Default
      expect(document.isArchived, false); // Default
      expect(document.createdAt, isNull);
      expect(document.updatedAt, isNull);
      expect(document.isSynced, true); // Default
      expect(document.isDirty, false); // Default
      expect(document.lastSyncedAt, isNull);
    });

    test('Document fromJson handles invalid date strings', () {
      // Arrange
      final json = {
        'id': 'doc1',
        'space_id': 'space1',
        'title': 'Test Document',
        'created_by': 'user1',
        'last_edited_by': 'user1',
        'created_at': 'invalid-date',
      };

      // Act
      final document = Document.fromJson(json);

      // Assert - Invalid date returns DateTime.now() not null
      expect(document.createdAt, isNotNull);
    });

    test('Document fromJson handles empty date strings', () {
      // Arrange
      final json = {
        'id': 'doc1',
        'space_id': 'space1',
        'title': 'Test Document',
        'created_by': 'user1',
        'last_edited_by': 'user1',
        'updated_at': '',
      };

      // Act
      final document = Document.fromJson(json);

      // Assert - Empty string returns null
      expect(document.updatedAt, isNull);
    });

    test('Document equality works correctly', () {
      // Arrange
      const doc1 = Document(
        id: 'doc1',
        spaceId: 'space1',
        title: 'Document 1',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      const doc2 = Document(
        id: 'doc1',
        spaceId: 'space2',
        title: 'Different Title',
        createdBy: 'user2',
        lastEditedBy: 'user2',
      );

      const doc3 = Document(
        id: 'doc2',
        spaceId: 'space1',
        title: 'Document 1',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      // Assert
      expect(doc1, equals(doc2)); // Same ID
      expect(doc1, isNot(equals(doc3))); // Different ID
      expect(doc1.hashCode, equals(doc2.hashCode));
    });

    test('Document toString returns formatted string', () {
      // Arrange & Act
      const document = Document(
        id: 'doc1',
        spaceId: 'space1',
        title: 'Test Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      // Assert
      expect(document.toString(), 'Document(id: doc1, title: Test Document)');
    });

    test('Document with hierarchical structure (parentId)', () {
      // Arrange & Act
      const parentDoc = Document(
        id: 'doc1',
        spaceId: 'space1',
        title: 'Parent Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      const childDoc = Document(
        id: 'doc2',
        spaceId: 'space1',
        parentId: 'doc1',
        title: 'Child Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      // Assert
      expect(parentDoc.parentId, isNull);
      expect(childDoc.parentId, 'doc1');
      expect(childDoc.spaceId, parentDoc.spaceId);
    });

    test('Document with offline-first sync status', () {
      // Arrange & Act
      const syncedDoc = Document(
        id: 'doc1',
        spaceId: 'space1',
        title: 'Synced Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      const dirtyDoc = Document(
        id: 'doc2',
        spaceId: 'space1',
        title: 'Unsynced Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
        isSynced: false,
        isDirty: true,
      );

      // Assert
      expect(syncedDoc.isSynced, true);
      expect(syncedDoc.isDirty, false);
      expect(dirtyDoc.isSynced, false);
      expect(dirtyDoc.isDirty, true);
    });

    test('Document with archived status', () {
      // Arrange & Act
      const activeDoc = Document(
        id: 'doc1',
        spaceId: 'space1',
        title: 'Active Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      const archivedDoc = Document(
        id: 'doc2',
        spaceId: 'space1',
        title: 'Archived Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
        isArchived: true,
      );

      // Assert
      expect(activeDoc.isArchived, false);
      expect(archivedDoc.isArchived, true);
    });

    test('Document with content size tracking', () {
      // Arrange & Act
      const smallDoc = Document(
        id: 'doc1',
        spaceId: 'space1',
        title: 'Small Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
        content: {'text': 'Hello'},
        contentSize: 5,
      );

      final largeDoc = Document(
        id: 'doc2',
        spaceId: 'space1',
        title: 'Large Document',
        createdBy: 'user1',
        lastEditedBy: 'user1',
        content: {'text': 'A' * 1000},
        contentSize: 1000,
      );

      // Assert
      expect(smallDoc.contentSize, 5);
      expect(largeDoc.contentSize, 1000);
    });

    test('Document with emoji icon', () {
      // Arrange & Act
      const document = Document(
        id: 'doc1',
        spaceId: 'space1',
        title: 'Document with Icon',
        icon: '📄',
        createdBy: 'user1',
        lastEditedBy: 'user1',
      );

      // Assert
      expect(document.icon, '📄');
    });
  });
}
