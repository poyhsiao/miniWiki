import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/core/config/providers.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/repositories/share_repository_impl.dart';
import 'package:miniwiki/data/repositories/version_repository_impl.dart';
import 'package:miniwiki/services/comment_service.dart';
import 'package:miniwiki/services/file_service.dart';
import 'package:miniwiki/services/share_service.dart';
import 'package:miniwiki/services/version_service.dart';

/// Provider for VersionService
final versionServiceProvider = Provider<VersionService>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return VersionService(versionRepository: VersionRepositoryImpl(apiClient));
});

/// Provider for CommentService
final commentServiceProvider = Provider<CommentService>((ref) {
  final repository = ref.watch(commentRepositoryProvider);
  return CommentService(repository);
});

/// Provider for FileService
final fileServiceProvider = Provider<FileService>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  final repository = ref.watch(fileRepositoryProvider);
  final baseUrl = ref.watch(appConfigProvider);
  return FileService(
    apiClient: apiClient,
    fileRepository: repository,
    baseUrl: baseUrl,
  );
});

/// Provider for ShareService
final shareServiceProvider = Provider<ShareService>((ref) {
  final repository = ref.watch(shareRepositoryProvider);
  final baseUrl = ref.watch(appConfigProvider);
  return ShareService(repository, baseUrl);
});
