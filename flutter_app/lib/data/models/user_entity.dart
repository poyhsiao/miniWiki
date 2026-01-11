import 'package:isar/isar.dart';

part 'user_entity.g.dart';

@Collection()
class UserEntity {
  Id get id => int.tryParse(uuid) ?? Isar.autoIncrement;

  @Index()
  String uuid = '';

  String email = '';

  String displayName = '';

  String? avatarUrl;

  String timezone = 'UTC';

  String language = 'en';

  bool isEmailVerified = false;

  DateTime? lastLoginAt;

  DateTime? createdAt;

  DateTime? updatedAt;

  bool isSynced = true;

  bool isDirty = false;

  DateTime? lastSyncedAt;
}
