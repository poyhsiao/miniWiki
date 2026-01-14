import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/data/models/user_entity.dart';
import 'package:miniwiki/data/models/space_entity.dart';
import 'package:miniwiki/data/models/document_entity.dart';

final isarProvider = Provider<IsarDatabase>((ref) {
  throw UnimplementedError('IsarDatabase not initialized');
});

class IsarDatabase {
  // Isar instance - will be initialized in main.dart
  // ignore: unused_field
  final dynamic _isar;

  IsarDatabase(this._isar);

  Future<void> initialize() async {
    // Isar initialization will be done in main.dart
  }

  Future<List<UserEntity>> getUsers() async => [];

  Future<UserEntity?> getUserById(String id) async => null;

  Future<UserEntity?> getUserByEmail(String email) async => null;

  Future<void> saveUser(UserEntity user) async {}

  Future<void> deleteUser(String id) async {}

  Future<List<SpaceEntity>> getSpaces() async => [];

  Future<SpaceEntity?> getSpaceById(String id) async => null;

  Future<List<SpaceEntity>> getSpacesByOwner(String ownerId) async => [];

  Future<void> saveSpace(SpaceEntity space) async {}

  Future<void> deleteSpace(String id) async {}

  Future<List<DocumentEntity>> getDocuments() async => [];

  Future<DocumentEntity?> getDocumentById(String id) async => null;

  Future<List<DocumentEntity>> getDocumentsBySpace(String spaceId) async => [];

  Future<List<DocumentEntity>> getDocumentsByParent(String parentId) async => [];

  Future<void> saveDocument(DocumentEntity document) async {}

  Future<void> deleteDocument(String id) async {}

  Future<List<DocumentEntity>> getDirtyDocuments() async => [];

  Future<void> markDocumentSynced(String id) async {}
}
