import 'dart:async';

import 'package:flutter/services.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/core/network/network_error.dart' as ne;
import 'package:miniwiki/domain/entities/share_link.dart';
import 'package:miniwiki/domain/repositories/share_repository.dart';
import 'package:riverpod/riverpod.dart';

/// Implementation of ShareRepository for share link operations.
///
/// This class handles all share link related API calls to the backend
/// and provides clipboard integration for sharing URLs.
class ShareRepositoryImpl implements ShareRepository {
  final ApiClient apiClient;
  final String baseUrl;

  ShareRepositoryImpl({
    required this.apiClient,
    required this.baseUrl,
  });

  @override
  Future<ShareLink> createShareLink(CreateShareLinkRequest request) async {
    final response = await apiClient.dio.post<Map<String, dynamic>>(
      '$baseUrl/api/v1/documents/${request.documentId}/share',
      data: request.toJson(),
    );

    if (response.statusCode != 201) {
      throw ne.NetworkError(
        'Failed to create share link: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    return ShareLink.fromJson(response.data as Map<String, dynamic>);
  }

  @override
  Future<List<ShareLink>> getShareLinks(String documentId) async {
    final response = await apiClient.dio.get<List<dynamic>>(
      '$baseUrl/api/v1/documents/$documentId/share',
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Failed to get share links: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    final data = response.data as List<dynamic>;
    return data
        .map((item) => ShareLink.fromJson(item as Map<String, dynamic>))
        .toList();
  }

  @override
  Future<ShareLink?> getShareLinkByToken(String token) async {
    final response = await apiClient.dio.get<Map<String, dynamic>>(
      '$baseUrl/api/v1/share/$token',
    );

    if (response.statusCode == 404) {
      return null;
    }

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Failed to get share link: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    final data = response.data as Map<String, dynamic>;

    // Check if access code is required
    if (data['requires_access_code'] == true) {
      // Return a partial share link with limited info
      return ShareLink(
        id: data['id'] as String,
        documentId: data['document_id'] as String,
        documentTitle: data['document_title'] as String,
        token: token,
        requiresAccessCode: true,
        expiresAt: data['expires_at'] != null
            ? DateTime.parse(data['expires_at'] as String)
            : null,
        permission: data['permission'] as String,
        isActive: true,
        createdAt: DateTime.now(),
        accessCount: 0,
        createdBy: 'Unknown',
      );
    }

    return ShareLink.fromJson(data);
  }

  @override
  Future<ShareLinkVerification> verifyAccessCode(
    String token,
    String accessCode,
  ) async {
    final response = await apiClient.dio.post<Map<String, dynamic>>(
      '$baseUrl/api/v1/share/$token/verify',
      data: {'accessCode': accessCode},
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Invalid access code',
        response.statusCode ?? 401,
      );
    }

    return ShareLinkVerification.fromJson(
        response.data as Map<String, dynamic>);
  }

  @override
  Future<void> deleteShareLink(String documentId, String token) async {
    final response = await apiClient.dio.delete<dynamic>(
      '$baseUrl/api/v1/documents/$documentId/share/$token',
    );

    if (response.statusCode != 204) {
      throw ne.NetworkError(
        'Failed to delete share link: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }
  }

  @override
  Future<bool> copyShareLinkToClipboard(
    ShareLink shareLink,
    String baseUrl,
  ) async {
    try {
      final url = shareLink.getShareUrl(baseUrl);
      await Clipboard.setData(ClipboardData(text: url));
      return true;
    } catch (e) {
      return false;
    }
  }
}

/// Provider for ShareRepositoryImpl
final shareRepositoryProvider = Provider<ShareRepository>((ref) {
  final apiClient = ref.watch(apiClientProvider);

  // Read and validate API_BASE_URL from environment
  const envBaseUrl = String.fromEnvironment('API_BASE_URL');

  // Validate: must be non-empty or use development default
  final baseUrl = envBaseUrl.isEmpty
      ? 'http://localhost:3000'  // Development default
      : envBaseUrl;

  // Optional: Assert it's a valid URL format
  if (!baseUrl.startsWith('http://') && !baseUrl.startsWith('https://')) {
    throw ArgumentError(
      'API_BASE_URL must be a valid HTTP(S) URL, got: $baseUrl'
    );
  }

  return ShareRepositoryImpl(
    apiClient: apiClient,
    baseUrl: baseUrl,
  );
});
