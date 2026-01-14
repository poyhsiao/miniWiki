import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/services/version_service.dart';
import 'package:miniwiki/core/config/providers.dart';

/// Provider for VersionService
final versionServiceProvider = Provider<VersionService>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return VersionService(versionRepository: VersionRepositoryImpl(apiClient));
});
