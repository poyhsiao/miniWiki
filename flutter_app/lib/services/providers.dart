import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/services/version_service.dart';
import 'package:miniwiki/services/comment_service.dart';
import 'package:miniwiki/services/file_service.dart';
import 'package:miniwiki/core/config/providers.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/domain/repositories/comment_repository.dart';
import 'package:miniwiki/domain/repositories/file_repository.dart';
import 'package:miniwiki/data/repositories/comment_repository_impl.dart';
import 'package:miniwiki/data/repositories/file_repository_impl.dart';

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
  const baseUrl = ''; // Will be loaded from config
  return FileService(
    apiClient: apiClient,
    fileRepository: repository,
    baseUrl: baseUrl,
  );
});
