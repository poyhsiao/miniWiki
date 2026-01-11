import 'package:isar/isar.dart';

part 'sync_queue_item.g.dart';

@Collection()
class SyncQueueItem {
  Id get id => Isar.autoIncrement;

  @Index()
  String entityType = '';

  @Index()
  String entityId = '';

  String operation = '';

  Map<String, dynamic> data = {};

  int retryCount = 0;

  DateTime? nextRetryAt;

  DateTime createdAt = DateTime.now();

  int priority = 0;
}
