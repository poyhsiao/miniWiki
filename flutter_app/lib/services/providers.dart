import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/services/version_service.dart';
import 'package:miniwiki/core/config/providers.dart';
import 'package:miniwiki/core/network/api_client.dart';

/// Provider for VersionService
final versionServiceProvider = Provider<VersionService>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return VersionService(versionRepository: VersionRepositoryImpl(apiClient));
});
