import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Simple local storage service using SharedPreferences
/// Web-compatible alternative to Isar for offline storage
class LocalStorageService {
  static LocalStorageService? _instance;
  static SharedPreferences? _prefs;

  LocalStorageService._internal();

  static Future<LocalStorageService> getInstance() async {
    _instance ??= LocalStorageService._internal();
    _prefs ??= await SharedPreferences.getInstance();
    return _instance!;
  }

  /// Get a value by key
  String? getString(String key) => _prefs?.getString(key);

  /// Set a value by key
  Future<bool> setString(String key, String value) async {
    return await _prefs?.setString(key, value) ?? false;
  }

  /// Get a list of values by key prefix
  List<String> getValuesByPrefix(String prefix) {
    final keys = _prefs?.getKeys() ?? {};
    return keys
        .where((k) => k.startsWith(prefix))
        .map((k) => _prefs!.getString(k))
        .where((v) => v != null)
        .cast<String>()
        .toList();
  }

  /// Get all keys matching a pattern
  List<String> getKeysByPrefix(String prefix) {
    final keys = _prefs?.getKeys() ?? {};
    return keys.where((k) => k.startsWith(prefix)).toList();
  }

  /// Remove a key
  Future<bool> remove(String key) async {
    return await _prefs?.remove(key) ?? false;
  }

  /// Remove all keys with a prefix
  Future<void> removeByPrefix(String prefix) async {
    final keys = getKeysByPrefix(prefix);
    for (final key in keys) {
      await remove(key);
    }
  }

  /// Check if key exists
  bool containsKey(String key) => _prefs?.containsKey(key) ?? false;

  /// Clear all data
  Future<bool> clear() async {
    return await _prefs?.clear() ?? false;
  }
}

/// Global storage instance (initialized at app startup)
LocalStorageService? _globalStorage;

/// Initialize local storage - call this at app startup
Future<LocalStorageService> initLocalStorage() async {
  _globalStorage ??= await LocalStorageService.getInstance();
  return _globalStorage!;
}

/// Provider for LocalStorageService (synchronous after initialization)
final localStorageServiceProvider = Provider<LocalStorageService>((ref) {
  if (_globalStorage == null) {
    throw StateError(
        'LocalStorageService not initialized. Call initLocalStorage() at app startup.');
  }
  return _globalStorage!;
});

/// Document cache service using local storage
class DocumentCacheService {
  static const String _docPrefix = 'doc_';
  static const String _cachePrefix = 'cache_';

  final LocalStorageService _storage;

  DocumentCacheService(this._storage);

  /// Cache a document
  Future<void> cacheDocument(String docId, Map<String, dynamic> data) async {
    final json = jsonEncode(data);
    await _storage.setString('$_docPrefix$docId', json);
  }

  /// Get a cached document
  Map<String, dynamic>? getDocument(String docId) {
    final json = _storage.getString('$_docPrefix$docId');
    if (json == null) return null;
    try {
      return jsonDecode(json) as Map<String, dynamic>;
    } catch (_) {
      return null;
    }
  }

  /// Remove a cached document
  Future<void> removeDocument(String docId) async {
    await _storage.remove('$_docPrefix$docId');
  }

  /// Get all cached document IDs
  List<String> getCachedDocIds() {
    return _storage
        .getKeysByPrefix(_docPrefix)
        .map((key) => key.replaceFirst(_docPrefix, ''))
        .toList();
  }

  /// Clear all cached documents
  Future<void> clearCache() async {
    await _storage.removeByPrefix(_docPrefix);
    await _storage.removeByPrefix(_cachePrefix);
  }

  /// Cache document content for offline
  Future<void> cacheContent(String docId, String content) async {
    await _storage.setString('$_cachePrefix$docId', content);
  }

  /// Get cached content
  String? getCachedContent(String docId) {
    return _storage.getString('$_cachePrefix$docId');
  }
}

/// Sync queue service using local storage
class SyncQueueService {
  static const String _queuePrefix = 'sync_queue_';
  static const String _failedPrefix = 'sync_failed_';

  final LocalStorageService _storage;

  SyncQueueService(this._storage);

  /// Add item to sync queue
  Future<void> addItem(String entityType, String entityId, String operation,
      Map<String, dynamic> data) async {
    final item = {
      'entityType': entityType,
      'entityId': entityId,
      'operation': operation,
      'data': data,
      'createdAt': DateTime.now().toIso8601String(),
      'retryCount': 0,
    };
    final json = jsonEncode(item);
    await _storage.setString('$_queuePrefix${entityType}_$entityId', json);
  }

  /// Get all queue items
  List<Map<String, dynamic>> getQueueItems() {
    final items = _storage.getValuesByPrefix(_queuePrefix);
    return items
        .map((json) {
          try {
            return jsonDecode(json) as Map<String, dynamic>;
          } catch (_) {
            return <String, dynamic>{};
          }
        })
        .where((item) => item.isNotEmpty)
        .toList();
  }

  /// Remove item from queue
  Future<void> removeItem(String entityType, String entityId) async {
    await _storage.remove('$_queuePrefix${entityType}_$entityId');
  }

  /// Clear queue
  Future<void> clearQueue() async {
    await _storage.removeByPrefix(_queuePrefix);
  }

  /// Get queue size
  int getQueueSize() {
    return getQueueItems().length;
  }

  // ============================================
  // FAILED QUEUE OPERATIONS
  // ============================================

  /// Add item to failed queue
  Future<void> addFailedItem(String entityType, String entityId,
      String operation, Map<String, dynamic> data, String error) async {
    final item = {
      'entityType': entityType,
      'entityId': entityId,
      'operation': operation,
      'data': data,
      'error': error,
      'failedAt': DateTime.now().toIso8601String(),
    };
    final json = jsonEncode(item);
    await _storage.setString('$_failedPrefix${entityType}_$entityId', json);
  }

  /// Get all failed items
  List<Map<String, dynamic>> getFailedItems() {
    final items = _storage.getValuesByPrefix(_failedPrefix);
    return items
        .map((json) {
          try {
            return jsonDecode(json) as Map<String, dynamic>;
          } catch (_) {
            return <String, dynamic>{};
          }
        })
        .where((item) => item.isNotEmpty)
        .toList();
  }

  /// Remove item from failed queue
  Future<void> removeFailedItem(String entityType, String entityId) async {
    await _storage.remove('$_failedPrefix${entityType}_$entityId');
  }

  /// Clear failed queue
  Future<void> clearFailedQueue() async {
    await _storage.removeByPrefix(_failedPrefix);
  }

  /// Get failed queue size
  int getFailedQueueSize() {
    return getFailedItems().length;
  }
}

/// User cache service
class UserCacheService {
  static const String _userPrefix = 'user_';

  final LocalStorageService _storage;

  UserCacheService(this._storage);

  /// Cache current user
  Future<void> cacheUser(Map<String, dynamic> userData) async {
    final json = jsonEncode(userData);
    await _storage.setString(_userPrefix, json);
  }

  /// Get cached user
  Map<String, dynamic>? getCachedUser() {
    final json = _storage.getString(_userPrefix);
    if (json == null) return null;
    try {
      return jsonDecode(json) as Map<String, dynamic>;
    } catch (_) {
      return null;
    }
  }

  /// Clear user cache
  Future<void> clearCache() async {
    await _storage.remove(_userPrefix);
  }
}

/// Space cache service
class SpaceCacheService {
  static const String _spacePrefix = 'space_';

  final LocalStorageService _storage;

  SpaceCacheService(this._storage);

  /// Cache a space
  Future<void> cacheSpace(String spaceId, Map<String, dynamic> data) async {
    final json = jsonEncode(data);
    await _storage.setString('$_spacePrefix$spaceId', json);
  }

  /// Get a cached space
  Map<String, dynamic>? getSpace(String spaceId) {
    final json = _storage.getString('$_spacePrefix$spaceId');
    if (json == null) return null;
    try {
      return jsonDecode(json) as Map<String, dynamic>;
    } catch (_) {
      return null;
    }
  }

  /// Get all cached space IDs
  List<String> getCachedSpaceIds() {
    return _storage
        .getKeysByPrefix(_spacePrefix)
        .map((key) => key.replaceFirst(_spacePrefix, ''))
        .toList();
  }

  /// Clear space cache
  Future<void> clearCache() async {
    await _storage.removeByPrefix(_spacePrefix);
  }
}
