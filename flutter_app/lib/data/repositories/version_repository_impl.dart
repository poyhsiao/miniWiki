import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/domain/entities/document_version.dart';
import 'package:miniwiki/domain/repositories/version_repository.dart';

/// Implementation of VersionRepository that handles version operations
/// via the API
class VersionRepositoryImpl implements VersionRepository {
  final ApiClient _apiClient;

  VersionRepositoryImpl(this._apiClient);

  @override
  Future<List<DocumentVersion>> listVersions(
    String documentId, {
    int limit = 20,
    int offset = 0,
  }) async {
    try {
      final response = await _apiClient.get(
        '/documents/$documentId/versions',
        queryParams: {
          'limit': limit,
          'offset': offset,
        },
      );

      final versionsData = response.data['data']['versions'] as List;

      final versions = versionsData
          .map((v) => DocumentVersion.fromJson(v as Map<String, dynamic>))
          .toList();

      return versions;
    } catch (e) {
      throw Exception('Failed to fetch versions: $e');
    }
  }

  @override
  Future<DocumentVersion> getVersion(
    String documentId,
    int versionNumber,
  ) async {
    try {
      final response = await _apiClient.get(
        '/documents/$documentId/versions/$versionNumber',
      );

      return DocumentVersion.fromJson(
          response.data['data'] as Map<String, dynamic>);
    } catch (e) {
      throw Exception('Failed to fetch version $versionNumber: $e');
    }
  }

  @override
  Future<DocumentVersion> createVersion({
    required String documentId,
    required String title,
    required Map<String, dynamic> content,
    String? changeSummary,
    Map<String, dynamic>? vectorClock,
  }) async {
    try {
      final response = await _apiClient.post(
        '/documents/$documentId/versions',
        data: {
          'title': title,
          'content': content,
          'change_summary': changeSummary,
          'vector_clock': vectorClock,
        },
      );

      return DocumentVersion.fromJson(
          response.data['data'] as Map<String, dynamic>);
    } catch (e) {
      throw Exception('Failed to create version: $e');
    }
  }

  @override
  Future<DocumentVersion> restoreVersion(
    String documentId,
    int versionNumber,
  ) async {
    try {
      final response = await _apiClient.post(
        '/documents/$documentId/versions/$versionNumber/restore',
      );

      return DocumentVersion.fromJson(
          response.data['data'] as Map<String, dynamic>);
    } catch (e) {
      throw Exception('Failed to restore to version $versionNumber: $e');
    }
  }

  @override
  Future<Map<String, dynamic>> compareVersions(
    String documentId,
    int fromVersion,
    int toVersion,
  ) async {
    try {
      final response = await _apiClient.get(
        '/documents/$documentId/versions/diff',
        queryParams: {
          'from': fromVersion,
          'to': toVersion,
        },
      );

      final diffData = response.data['data'] as Map<String, dynamic>;
      return {
        'added': diffData['added'] as List? ?? [],
        'removed': diffData['removed'] as List? ?? [],
        'modified': diffData['modified'] as List? ?? [],
      };
    } catch (e) {
      throw Exception('Failed to compare versions: $e');
    }
  }

  @override
  Future<int> getVersionCount(String documentId) async {
    try {
      // Get all versions with a large limit to count them
      final versions = await listVersions(documentId, limit: 1000);
      return versions.length;
    } catch (e) {
      throw Exception('Failed to get version count: $e');
    }
  }

  @override
  Future<DocumentVersion> getCurrentVersion(String documentId) async {
    try {
      // The API returns versions in descending order, so the first one is the latest
      final versions = await listVersions(documentId, limit: 1);
      if (versions.isEmpty) {
        throw Exception('No versions found for document');
      }
      return versions.first;
    } catch (e) {
      throw Exception('Failed to get current version: $e');
    }
  }
}
