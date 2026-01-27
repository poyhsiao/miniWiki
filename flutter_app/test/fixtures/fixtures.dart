import 'package:miniwiki/domain/entities/document.dart';
import 'package:miniwiki/domain/entities/file.dart' as entity;
import 'package:miniwiki/domain/entities/comment.dart';
import 'package:miniwiki/domain/entities/space.dart';
import 'package:miniwiki/domain/entities/document_version.dart';
import 'package:miniwiki/domain/entities/share_link.dart';
import 'package:miniwiki/domain/entities/space_membership.dart';
import 'package:miniwiki/domain/value_objects/role.dart';

/// 測試數據夾具工廠
class TestFixtures {
  // ===== Document Fixtures =====

  static Document createTestDocument({
    String? id,
    String? title,
    String? content,
    String? spaceId,
    String? ownerId,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) {
    return Document(
      id: id ?? 'test-doc-${DateTime.now().millisecondsSinceEpoch}',
      title: title ?? 'Test Document',
      content: content ?? '# Test Content\n\nThis is a test document.',
      spaceId: spaceId ?? 'test-space-id',
      ownerId: ownerId ?? 'test-user-id',
      createdAt: createdAt ?? DateTime.now().subtract(const Duration(days: 1)),
      updatedAt: updatedAt ?? DateTime.now(),
    );
  }

  static List<Document> createTestDocumentList(int count) {
    return List.generate(
      count,
      (index) => createTestDocument(
        title: 'Test Document $index',
        content: 'Content for document $index',
      ),
    );
  }

  // ===== File Fixtures =====

  static entity.FileEntity createTestFile({
    String? id,
    String? fileName,
    String? contentType,
    int? fileSize,
    String? spaceId,
    String? documentId,
    String? uploaderId,
  }) {
    return entity.FileEntity(
      id: id ?? 'test-file-${DateTime.now().millisecondsSinceEpoch}',
      fileName: fileName ?? 'test-file.pdf',
      contentType: contentType ?? 'application/pdf',
      fileSize: fileSize ?? 1024000,
      spaceId: spaceId ?? 'test-space-id',
      documentId: documentId ?? 'test-doc-id',
      uploaderId: uploaderId ?? 'test-user-id',
      uploadedAt: DateTime.now(),
      fileUrl: 'https://example.com/files/test-file.pdf',
    );
  }

  static List<entity.FileEntity> createTestFileList(int count) {
    return List.generate(
      count,
      (index) => createTestFile(
        fileName: 'test-file-$index.pdf',
      ),
    );
  }

  // ===== Comment Fixtures =====

  static Comment createTestComment({
    String? id,
    String? content,
    String? documentId,
    String? authorId,
    String? authorName,
    DateTime? createdAt,
  }) {
    return Comment(
      id: id ?? 'test-comment-${DateTime.now().millisecondsSinceEpoch}',
      content: content ?? 'This is a test comment',
      documentId: documentId ?? 'test-doc-id',
      authorId: authorId ?? 'test-user-id',
      authorName: authorName ?? 'Test User',
      createdAt: createdAt ?? DateTime.now(),
      resolved: false,
    );
  }

  static List<Comment> createTestCommentList(int count) {
    return List.generate(
      count,
      (index) => createTestComment(
        content: 'Test comment $index',
      ),
    );
  }

  // ===== Space Fixtures =====

  static Space createTestSpace({
    String? id,
    String? name,
    String? description,
    String? ownerId,
  }) {
    return Space(
      id: id ?? 'test-space-${DateTime.now().millisecondsSinceEpoch}',
      name: name ?? 'Test Space',
      description: description ?? 'A test space for testing',
      ownerId: ownerId ?? 'test-user-id',
      createdAt: DateTime.now().subtract(const Duration(days: 7)),
      memberCount: 5,
      documentCount: 10,
    );
  }

  static List<Space> createTestSpaceList(int count) {
    return List.generate(
      count,
      (index) => createTestSpace(
        name: 'Test Space $index',
      ),
    );
  }

  // ===== Document Version Fixtures =====

  static DocumentVersion createTestDocumentVersion({
    String? id,
    String? documentId,
    int? versionNumber,
    String? title,
    Map<String, dynamic>? content,
    String? changeSummary,
  }) {
    return DocumentVersion(
      id: id ?? 'test-version-${DateTime.now().millisecondsSinceEpoch}',
      documentId: documentId ?? 'test-doc-id',
      versionNumber: versionNumber ?? 1,
      title: title ?? 'Test Document v1',
      content: content ?? {'delta': []},
      createdAt: DateTime.now(),
      createdBy: 'test-user-id',
      creatorName: 'Test User',
      changeSummary: changeSummary ?? 'Initial version',
    );
  }

  static List<DocumentVersion> createTestDocumentVersionList(int count) {
    return List.generate(
      count,
      (index) => createTestDocumentVersion(
        versionNumber: index + 1,
        title: 'Test Document v${index + 1}',
      ),
    );
  }

  // ===== Share Link Fixtures =====

  static ShareLink createTestShareLink({
    String? id,
    String? documentId,
    String? token,
    Role? permission,
    DateTime? expiresAt,
  }) {
    return ShareLink(
      id: id ?? 'test-share-${DateTime.now().millisecondsSinceEpoch}',
      documentId: documentId ?? 'test-doc-id',
      token: token ?? 'test-share-token',
      permission: permission ?? Role.viewer,
      createdAt: DateTime.now(),
      expiresAt: expiresAt ?? DateTime.now().add(const Duration(days: 7)),
      createdBy: 'test-user-id',
    );
  }

  // ===== Space Membership Fixtures =====

  static SpaceMembership createTestMembership({
    String? id,
    String? spaceId,
    String? userId,
    String? userName,
    Role? role,
  }) {
    return SpaceMembership(
      id: id ?? 'test-membership-${DateTime.now().millisecondsSinceEpoch}',
      spaceId: spaceId ?? 'test-space-id',
      userId: userId ?? 'test-user-id',
      userName: userName ?? 'Test User',
      role: role ?? Role.editor,
      joinedAt: DateTime.now(),
    );
  }

  static List<SpaceMembership> createTestMembershipList(int count) {
    return List.generate(
      count,
      (index) => createTestMembership(
        userName: 'Test User $index',
        role: Role.values[index % Role.values.length],
      ),
    );
  }

  // ===== API Response Fixtures =====

  static Map<String, dynamic> createSuccessResponse(dynamic data) {
    return {
      'status': 'success',
      'data': data,
    };
  }

  static Map<String, dynamic> createErrorResponse(String message) {
    return {
      'status': 'error',
      'message': message,
    };
  }

  static Map<String, dynamic> createPaginatedResponse(
    List<dynamic> items, {
    int total = 0,
    int offset = 0,
    int limit = 20,
  }) {
    return {
      'status': 'success',
      'data': {
        'items': items,
        'pagination': {
          'total': total,
          'offset': offset,
          'limit': limit,
        },
      },
    };
  }
}

/// 測試場景數據生成器
class TestScenarioData {
  /// 生成一個完整的文檔編輯場景
  static Map<String, dynamic> createDocumentEditScenario() {
    final doc = TestFixtures.createTestDocument();
    final comment1 = TestFixtures.createTestComment(
      documentId: doc.id,
      content: 'First comment',
    );
    final comment2 = TestFixtures.createTestComment(
      documentId: doc.id,
      content: 'Second comment',
    );

    return {
      'document': doc,
      'comments': [comment1, comment2],
      'versions': TestFixtures.createTestDocumentVersionList(3),
    };
  }

  /// 生成一個完整的空間場景
  static Map<String, dynamic> createSpaceScenario() {
    final space = TestFixtures.createTestSpace();
    return {
      'space': space,
      'documents': TestFixtures.createTestDocumentList(5),
      'members': TestFixtures.createTestMembershipList(3),
    };
  }

  /// 生成文件上傳場景
  static Map<String, dynamic> createFileUploadScenario() {
    final doc = TestFixtures.createTestDocument();
    final file = TestFixtures.createTestFile(documentId: doc.id);
    return {
      'document': doc,
      'file': file,
      'uploadProgress': [0.0, 0.25, 0.5, 0.75, 1.0],
    };
  }
}
