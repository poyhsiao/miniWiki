import 'dart:io';
import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/data/models/user_entity.dart';
import 'package:miniwiki/data/models/space_entity.dart';
import 'package:miniwiki/data/models/document_entity.dart';

final isarProvider = Provider<IsarDatabase>((ref) {
  throw UnimplementedError('IsarDatabase not initialized');
});

class IsarDatabase {
  final dynamic _isar;

  IsarDatabase(this._isar);

  Future<void> initialize() async {
    // Isar initialization will be done in main.dart
  }

  Future<List<UserEntity>> getUsers() async {
    return [];
  }

  Future<UserEntity?> getUserById(String id) async {
    return null;
  }

  Future<UserEntity?> getUserByEmail(String email) async {
    return null;
  }

  Future<void> saveUser(UserEntity user) async {}

  Future<void> deleteUser(String id) async {}

  Future<List<SpaceEntity>> getSpaces() async {
    return [];
  }

  Future<SpaceEntity?> getSpaceById(String id) async {
    return null;
  }

  Future<List<SpaceEntity>> getSpacesByOwner(String ownerId) async {
    return [];
  }

  Future<void> saveSpace(SpaceEntity space) async {}

  Future<void> deleteSpace(String id) async {}

  Future<List<DocumentEntity>> getDocuments() async {
    return [];
  }

  Future<DocumentEntity?> getDocumentById(String id) async {
    return null;
  }

  Future<List<DocumentEntity>> getDocumentsBySpace(String spaceId) async {
    return [];
  }

  Future<List<DocumentEntity>> getDocumentsByParent(String parentId) async {
    return [];
  }

  Future<void> saveDocument(DocumentEntity document) async {}

  Future<void> deleteDocument(String id) async {}

  Future<List<DocumentEntity>> getDirtyDocuments() async {
    return [];
  }

  Future<void> markDocumentSynced(String id) async {}
}
