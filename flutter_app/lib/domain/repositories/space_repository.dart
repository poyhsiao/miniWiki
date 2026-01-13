import 'package:miniwiki/domain/entities/space.dart';
import 'package:miniwiki/domain/entities/space_membership.dart';

abstract class SpaceRepository {
  Future<List<Space>> listSpaces();
  Future<Space> getSpace(String spaceId);
  Future<Space> createSpace({
    required String name,
    String? icon,
    String? description,
    bool isPublic = false,
  });
  Future<Space> updateSpace(
    String spaceId, {
    String? name,
    String? icon,
    String? description,
    bool? isPublic,
  });
  Future<void> deleteSpace(String spaceId);
  Future<List<SpaceMembership>> listMembers(String spaceId);
  Future<SpaceMembership> addMember(
    String spaceId,
    String userId,
    String role,
  );
  Future<SpaceMembership> updateMemberRole(
    String spaceId,
    String userId,
    String role,
  );
  Future<void> removeMember(String spaceId, String userId);
}
