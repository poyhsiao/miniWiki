import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/services/version_service.dart';
import 'package:miniwiki/services/comment_service.dart';
import 'package:miniwiki/core/config/providers.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/domain/repositories/comment_repository.dart';
import 'package:miniwiki/data/repositories/comment_repository_impl.dart';

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
