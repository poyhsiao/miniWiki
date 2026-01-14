import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/domain/entities/space.dart';
import 'package:miniwiki/domain/entities/space_membership.dart';
import 'package:miniwiki/domain/repositories/space_repository.dart';
import 'package:riverpod/riverpod.dart';

class SpaceRepositoryImpl implements SpaceRepository {
  final ApiClient _apiClient;

  SpaceRepositoryImpl(this._apiClient);

  @override
  Future<List<Space>> listSpaces() async {
    final response = await _apiClient.get('/spaces');
    final data = response.data['data'] as Map<String, dynamic>;
    final spacesJson = data['spaces'] as List;

    return spacesJson
        .map((space) => Space.fromJson(space as Map<String, dynamic>))
        .toList();
  }

  @override
  Future<Space> getSpace(String spaceId) async {
    final response = await _apiClient.get('/spaces/$spaceId');
    final data = response.data['data'] as Map<String, dynamic>;
    return Space.fromJson(data);
  }

  @override
  Future<Space> createSpace({
    required String name,
    String? icon,
    String? description,
    bool isPublic = false,
  }) async {
    final requestData = <String, dynamic>{
      'name': name,
      'is_public': isPublic,
    };
    if (icon != null) requestData['icon'] = icon;
    if (description != null) requestData['description'] = description;

    final response = await _apiClient.post('/spaces', data: requestData);
    final data = response.data['data'] as Map<String, dynamic>;
    return Space.fromJson(data);
  }

  @override
  Future<Space> updateSpace(
    String spaceId, {
    String? name,
    String? icon,
    String? description,
    bool? isPublic,
  }) async {
    final requestData = <String, dynamic>{};
    if (name != null) requestData['name'] = name;
    if (icon != null) requestData['icon'] = icon;
    if (description != null) requestData['description'] = description;
    if (isPublic != null) requestData['is_public'] = isPublic;

    final response = await _apiClient.patch('/spaces/$spaceId', data: requestData);
    final data = response.data['data'] as Map<String, dynamic>;
    return Space.fromJson(data);
  }

  @override
  Future<void> deleteSpace(String spaceId) async {
    await _apiClient.delete('/spaces/$spaceId');
  }

  @override
  Future<List<SpaceMembership>> listMembers(String spaceId) async {
    final response = await _apiClient.get('/spaces/$spaceId/members');
    final data = response.data['data'] as Map<String, dynamic>;
    final membersJson = data['members'] as List;

    return membersJson
        .map((member) => SpaceMembership.fromJson(member as Map<String, dynamic>))
        .toList();
  }

  @override
  Future<SpaceMembership> addMember(
    String spaceId,
    String userId,
    String role,
  ) async {
    final response = await _apiClient.post(
      '/spaces/$spaceId/members',
      data: {
        'user_id': userId,
        'role': role,
      },
    );
    final data = response.data['data'] as Map<String, dynamic>;
    return SpaceMembership.fromJson(data);
  }

  @override
  Future<SpaceMembership> updateMemberRole(
    String spaceId,
    String userId,
    String role,
  ) async {
    final response = await _apiClient.patch(
      '/spaces/$spaceId/members/$userId',
      data: {'role': role},
    );
    final data = response.data['data'] as Map<String, dynamic>;
    return SpaceMembership.fromJson(data);
  }

  @override
  Future<void> removeMember(String spaceId, String userId) async {
    await _apiClient.delete('/spaces/$spaceId/members/$userId');
  }
}

final spaceRepositoryProvider = Provider<SpaceRepository>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return SpaceRepositoryImpl(apiClient);
});
