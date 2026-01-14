import 'package:isar/isar.dart';

part 'space_entity.g.dart';

@Collection()
class SpaceEntity {
  Id id = Isar.autoIncrement;

  @Index()
  String uuid = '';

  String name = '';

  String? icon;

  String? description;

  bool isPublic = false;

  @Index()
  String ownerId = '';

  DateTime? createdAt;

  DateTime? updatedAt;

  bool isSynced = true;

  bool isDirty = false;

  DateTime? lastSyncedAt;
}
