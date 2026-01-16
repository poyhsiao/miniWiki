import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/data/models/document_entity.dart';

void main() {
  group('DocumentEntity Tests', () {
    test('create document entity with default values', () {
      final doc = DocumentEntity()
        ..uuid = 'test-uuid'
        ..spaceId = 'space-uuid'
        ..title = 'Test Document';

      expect(doc.uuid, 'test-uuid');
      expect(doc.spaceId, 'space-uuid');
      expect(doc.title, 'Test Document');
      expect(doc.isSynced, true);
      expect(doc.isDirty, false);
      expect(doc.isArchived, false);
    });

    test('document entity content is stored correctly', () {
      final content = {'type': 'Y.Doc', 'update': 'base64data'};
      final doc = DocumentEntity()
        ..uuid = 'test-uuid'
        ..spaceId = 'space-uuid'
        ..title = 'Test'
        ..content = content;

      expect(doc.content!['type'], 'Y.Doc');
      expect(doc.content!['update'], 'base64data');
    });

    test('document entity with parent hierarchy', () {
      final doc = DocumentEntity()
        ..uuid = 'child-uuid'
        ..spaceId = 'space-uuid'
        ..parentId = 'parent-uuid'
        ..title = 'Child Document';

      expect(doc.parentId, 'parent-uuid');
    });

    test('document entity dirty flag management', () {
      final doc = DocumentEntity()
        ..uuid = 'test-uuid'
        ..spaceId = 'space-uuid'
        ..title = 'Test';

      // Initially synced
      expect(doc.isSynced, true);
      expect(doc.isDirty, false);

      // Mark as dirty
      doc.isDirty = true;
      doc.isSynced = false;

      expect(doc.isSynced, false);
      expect(doc.isDirty, true);
    });

    test('document entity archived state', () {
      final doc = DocumentEntity()
        ..uuid = 'test-uuid'
        ..spaceId = 'space-uuid'
        ..title = 'Test';

      expect(doc.isArchived, false);

      doc.isArchived = true;

      expect(doc.isArchived, true);
    });
  });

  group('DocumentEntity - CRUD Operations Simulation', () {
    test('simulate document creation flow', () {
      // Create
      final doc = DocumentEntity()
        ..uuid = 'new-doc'
        ..spaceId = 'space-1'
        ..title = 'New Document'
        ..content = {'type': 'Y.Doc'}
        ..createdBy = 'user-1'
        ..lastEditedBy = 'user-1'
        ..isSynced = true
        ..isDirty = false;

      expect(doc.isSynced, true);
      expect(doc.isDirty, false);
    });

    test('simulate document edit and sync', () {
      final doc = DocumentEntity()
        ..uuid = 'doc-1'
        ..spaceId = 'space-1'
        ..title = 'Original'
        ..content = {'text': 'Hello'}
        ..isSynced = true;

      // User edits - must reassign the full map because content getter returns a new Map each time
      doc.title = 'Modified';
      final newContent = Map<String, dynamic>.from(doc.content!);
      newContent['text'] = 'Hello World';
      doc.content = newContent;
      doc.isDirty = true;
      doc.isSynced = false;

      // Verify the content was updated
      expect(doc.content!['text'], 'Hello World');

      // Simulate sync
      doc.isDirty = false;
      doc.isSynced = true;

      expect(doc.title, 'Modified');
      expect(doc.isSynced, true);
      expect(doc.isDirty, false);
    });

    test('simulate document deletion', () {
      final doc = DocumentEntity()
        ..uuid = 'doc-1'
        ..spaceId = 'space-1'
        ..title = 'To Delete'
        ..isArchived = false;

      // Soft delete
      doc.isArchived = true;

      expect(doc.isArchived, true);
    });

    test('simulate document hierarchy', () {
      final parent = DocumentEntity()
        ..uuid = 'parent-1'
        ..spaceId = 'space-1'
        ..title = 'Parent';

      final child = DocumentEntity()
        ..uuid = 'child-1'
        ..spaceId = 'space-1'
        ..parentId = parent.uuid
        ..title = 'Child';

      expect(child.parentId, parent.uuid);
      expect(parent.parentId, isNull);
    });
  });
}
