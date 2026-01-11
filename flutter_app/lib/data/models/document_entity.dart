import 'package:isar/isar.dart';

part 'document_entity.g.dart';

@Collection()
class DocumentEntity {
  Id get id => int.tryParse(uuid) ?? Isar.autoIncrement;

  @Index()
  String uuid = '';

  @Index()
  String spaceId = '';

  String? parentId;

  String title = '';

  String? icon;

  Map<String, dynamic> content = {};

  int contentSize = 0;

  bool isArchived = false;

  String createdBy = '';

  String lastEditedBy = '';

  DateTime? createdAt;

  DateTime? updatedAt;

  bool isSynced = true;

  bool isDirty = false;

  DateTime? lastSyncedAt;

  @Index()
  DateTime? localUpdatedAt;
}
