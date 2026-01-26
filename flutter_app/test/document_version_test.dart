import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/document_version.dart';

void main() {
  group('DocumentVersion Entity Tests', () {
    test('DocumentVersion can be created with all fields', () {
      // Arrange & Act
      final version = DocumentVersion(
        id: 'ver1',
        documentId: 'doc1',
        versionNumber: 1,
        title: 'Initial Version',
        content: <String, dynamic>{'delta': <dynamic>[]},
        createdBy: 'user1',
        vectorClock: {'clock': 1},
        createdAt: DateTime(2024),
      );

      // Assert
      expect(version.id, 'ver1');
      expect(version.documentId, 'doc1');
      expect(version.versionNumber, 1);
      expect(version.title, 'Initial Version');
      expect(version.content, <String, dynamic>{'delta': <dynamic>[]});
      expect(version.createdBy, 'user1');
      expect(version.vectorClock, {'clock': 1});
      expect(version.createdAt, DateTime(2024));
    });

    test('DocumentVersion can be created with optional fields', () {
      // Arrange & Act
      final version = DocumentVersion(
        id: 'ver1',
        documentId: 'doc1',
        versionNumber: 1,
        title: 'Version',
        content: <String, dynamic>{'delta': <dynamic>[]},
        createdBy: 'user1',
        vectorClock: {'clock': 1},
        createdAt: DateTime(2024),
        changeSummary: 'Fixed typo',
      );

      // Assert
      expect(version.changeSummary, 'Fixed typo');
    });

    test('DocumentVersion copyWith creates modified copy', () {
      // Arrange
      final original = DocumentVersion(
        id: 'ver1',
        documentId: 'doc1',
        versionNumber: 1,
        title: 'Version 1',
        content: <String, dynamic>{'delta': <dynamic>[]},
        createdBy: 'user1',
        vectorClock: {'clock': 1},
        createdAt: DateTime(2024),
      );

      // Act
      final modified = original.copyWith(
        title: 'Updated Version',
        versionNumber: 2,
      );

      // Assert
      expect(modified.id, 'ver1');
      expect(modified.title, 'Updated Version');
      expect(modified.versionNumber, 2);
      expect(modified.documentId, 'doc1'); // Unchanged
    });

    test('DocumentVersion toJson creates correct JSON', () {
      // Arrange
      final version = DocumentVersion(
        id: 'ver1',
        documentId: 'doc1',
        versionNumber: 1,
        title: 'Version 1',
        content: <String, dynamic>{'delta': <dynamic>[]},
        createdBy: 'user1',
        vectorClock: {'clock': 1},
        createdAt: DateTime(2024),
        changeSummary: 'Initial commit',
      );

      // Act
      final json = version.toJson();

      // Assert
      expect(json['id'], 'ver1');
      expect(json['document_id'], 'doc1');
      expect(json['version_number'], 1);
      expect(json['title'], 'Version 1');
      expect(json['created_by'], 'user1');
      expect(json['vector_clock'], {'clock': 1});
      expect(json['change_summary'], 'Initial commit');
    });

    test('DocumentVersion fromJson creates instance correctly', () {
      // Arrange
      final json = {
        'id': 'ver1',
        'document_id': 'doc1',
        'version_number': 1,
        'title': 'Version 1',
        'content': <String, dynamic>{'delta': <dynamic>[]},
        'created_by': 'user1',
        'vector_clock': {'clock': 1},
        'created_at': '2024-01-01T00:00:00Z',
        'change_summary': 'Initial commit',
      };

      // Act
      final version = DocumentVersion.fromJson(json);

      // Assert
      expect(version.id, 'ver1');
      expect(version.documentId, 'doc1');
      expect(version.versionNumber, 1);
      expect(version.title, 'Version 1');
      expect(version.createdBy, 'user1');
      expect(version.vectorClock, {'clock': 1});
      expect(version.changeSummary, 'Initial commit');
    });

    test('DocumentVersion fromJson handles null changeSummary', () {
      // Arrange
      final json = {
        'id': 'ver1',
        'document_id': 'doc1',
        'version_number': 1,
        'title': 'Version 1',
        'content': <String, dynamic>{'delta': <dynamic>[]},
        'created_by': 'user1',
        'vector_clock': {'clock': 1},
        'created_at': '2024-01-01T00:00:00Z',
      };

      // Act
      final version = DocumentVersion.fromJson(json);

      // Assert
      expect(version.id, 'ver1');
      expect(version.changeSummary, isNull);
    });

    test('DocumentVersion equality works correctly', () {
      // Arrange
      final version1 = DocumentVersion(
        id: 'ver1',
        documentId: 'doc1',
        versionNumber: 1,
        title: 'Version 1',
        content: <String, dynamic>{'delta': <dynamic>[]},
        createdBy: 'user1',
        vectorClock: {'clock': 1},
        createdAt: DateTime(2024),
      );

      final version2 = DocumentVersion(
        id: 'ver1',
        documentId: 'doc1',
        versionNumber: 1,
        title: 'Version 1',
        content: <String, dynamic>{'delta': <dynamic>[]},
        createdBy: 'user1',
        vectorClock: {'clock': 1},
        createdAt: DateTime(2024),
      );

      final version3 = DocumentVersion(
        id: 'ver2',
        documentId: 'doc1',
        versionNumber: 2,
        title: 'Version 2',
        content: <String, dynamic>{'delta': <dynamic>[]},
        createdBy: 'user1',
        vectorClock: {'clock': 2},
        createdAt: DateTime(2024),
      );

      // Assert
      expect(version1, equals(version2));
      expect(version1, isNot(equals(version3)));
      expect(version1.hashCode, equals(version2.hashCode));
    });

    test('DocumentVersion toString returns formatted string', () {
      // Arrange & Act
      final version = DocumentVersion(
        id: 'ver1',
        documentId: 'doc1',
        versionNumber: 1,
        title: 'Version 1',
        content: <String, dynamic>{'delta': <dynamic>[]},
        createdBy: 'user1',
        vectorClock: {'clock': 1},
        createdAt: DateTime(2024),
      );

      // Assert
      expect(
        version.toString(),
        'DocumentVersion(id: ver1, version: 1, title: Version 1)',
      );
    });
  });
}
