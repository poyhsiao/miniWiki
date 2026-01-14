import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/space.dart';
import 'package:miniwiki/domain/entities/space_membership.dart';
import 'package:miniwiki/services/space_service.dart';
import 'package:miniwiki/data/repositories/space_repository_impl.dart';

class SpaceProvider extends StateNotifier<SpaceState> {
  final SpaceService spaceService;

  SpaceProvider({required this.spaceService}) : super(SpaceState()) {
    loadSpaces();
  }

  Future<void> loadSpaces() async {
    state = state.copyWith(isLoading: true);
    try {
      final spaces = await spaceService.listSpaces();
      state = state.copyWith(spaces: spaces, isLoading: false);
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
    }
  }

  Future<void> createSpace({
    required String name,
    String? icon,
    String? description,
    bool isPublic = false,
  }) async {
    state = state.copyWith(isLoading: true);
    try {
      final space = await spaceService.createSpace(
        name: name,
        icon: icon,
        description: description,
        isPublic: isPublic,
      );
      state = state.copyWith(
        spaces: [...state.spaces, space],
        isLoading: false,
      );
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      rethrow;
    }
  }

  Future<void> deleteSpace(String spaceId) async {
    state = state.copyWith(isLoading: true);
    try {
      await spaceService.deleteSpace(spaceId);
      state = state.copyWith(
        spaces: state.spaces.where((s) => s.id != spaceId).toList(),
        isLoading: false,
      );
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      rethrow;
    }
  }

  Future<void> loadMembers(String spaceId) async {
    state = state.copyWith(isLoadingMembers: true);
    try {
      final members = await spaceService.listMembers(spaceId);
      state = state.copyWith(members: members, isLoadingMembers: false);
    } catch (e) {
      state = state.copyWith(isLoadingMembers: false, error: e.toString());
    }
  }

  Future<void> updateSpace(
    String spaceId, {
    String? name,
    String? icon,
    String? description,
    bool? isPublic,
  }) async {
    state = state.copyWith(isLoading: true);
    try {
      final space = await spaceService.updateSpace(
        spaceId,
        name: name,
        icon: icon,
        description: description,
        isPublic: isPublic,
      );
      state = state.copyWith(
        spaces: state.spaces.map((s) => s.id == spaceId ? space : s).toList(),
        selectedSpace: state.selectedSpace?.id == spaceId ? space : state.selectedSpace,
        isLoading: false,
      );
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      rethrow;
    }
  }

  Future<void> addMember(
    String spaceId,
    String userId,
    String role,
  ) async {
    try {
      final member = await spaceService.addMember(spaceId, userId, role);
      state = state.copyWith(
        members: [...state.members, member],
      );
    } catch (e) {
      state = state.copyWith(error: e.toString());
      rethrow;
    }
  }

  Future<void> updateMemberRole(
    String spaceId,
    String userId,
    String role,
  ) async {
    try {
      final member = await spaceService.updateMemberRole(spaceId, userId, role);
      state = state.copyWith(
        members: state.members.map((m) => m.userId == userId ? member : m).toList(),
      );
    } catch (e) {
      state = state.copyWith(error: e.toString());
      rethrow;
    }
  }

  Future<void> removeMember(String spaceId, String userId) async {
    try {
      await spaceService.removeMember(spaceId, userId);
      state = state.copyWith(
        members: state.members.where((m) => m.userId != userId).toList(),
      );
    } catch (e) {
      state = state.copyWith(error: e.toString());
      rethrow;
    }
  }

  void selectSpace(Space? space) {
    state = state.copyWith(selectedSpace: space);
  }

  void clearError() {
    state = state.copyWith();
  }
}

class SpaceState {
  final List<Space> spaces;
  final Space? selectedSpace;
  final List<SpaceMembership> members;
  final bool isLoading;
  final bool isLoadingMembers;
  final String? error;

  SpaceState({
    this.spaces = const [],
    this.selectedSpace,
    this.members = const [],
    this.isLoading = false,
    this.isLoadingMembers = false,
    this.error,
  });

  SpaceState copyWith({
    List<Space>? spaces,
    Space? selectedSpace,
    List<SpaceMembership>? members,
    bool? isLoading,
    bool? isLoadingMembers,
    String? error,
  }) => SpaceState(
      spaces: spaces ?? this.spaces,
      selectedSpace: selectedSpace ?? this.selectedSpace,
      members: members ?? this.members,
      isLoading: isLoading ?? this.isLoading,
      isLoadingMembers: isLoadingMembers ?? this.isLoadingMembers,
      error: error,
    );
}

final spaceProvider = StateNotifierProvider<SpaceProvider, SpaceState>((ref) {
  final repository = ref.watch(spaceRepositoryProvider);
  final service = SpaceService(spaceRepository: repository);
  return SpaceProvider(spaceService: service);
});
