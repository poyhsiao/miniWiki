import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/value_objects/role.dart';
import 'package:miniwiki/services/rbac_service.dart';

class _Undefined {
  const _Undefined();
}

const _undefined = _Undefined();

final permissionProvider =
    StateNotifierProvider.family<PermissionNotifier, PermissionState, String>(
  (ref, spaceId) => PermissionNotifier(
    rbacService: ref.read(rbacServiceProvider),
    spaceId: spaceId,
  ),
);

final rbacServiceProvider = Provider<RbacService>(
  (ref) {
    throw UnimplementedError(
        'rbacServiceProvider must be overridden by the app');
  },
);

class PermissionState {
  final bool isLoading;
  final Role? userRole;
  final Set<Permission> permissions;
  final Set<ActionType> allowedActions;
  final String? error;

  const PermissionState({
    this.isLoading = false,
    this.userRole = null,
    this.permissions = const {},
    this.allowedActions = const {},
    this.error = null,
  });

  PermissionState copyWith({
    bool? isLoading,
    Object? userRole = _undefined,
    Set<Permission>? permissions,
    Set<ActionType>? allowedActions,
    Object? error = _undefined,
  }) {
    return PermissionState(
      isLoading: isLoading ?? this.isLoading,
      userRole: userRole == _undefined ? this.userRole : (userRole as Role?),
      permissions: permissions ?? this.permissions,
      allowedActions: allowedActions ?? this.allowedActions,
      error: error == _undefined ? this.error : (error as String?),
    );
  }

  bool get canEdit => allowedActions.contains(ActionType.editDocument);
  bool get canCreate => allowedActions.contains(ActionType.createDocument);
  bool get canDelete => allowedActions.contains(ActionType.deleteDocument);
  bool get canComment => allowedActions.contains(ActionType.comment);
  bool get canShare => allowedActions.contains(ActionType.share);
  bool get canManageMembers =>
      allowedActions.contains(ActionType.manageMembers);
  bool get canManageRoles => allowedActions.contains(ActionType.manageRoles);
  bool get isOwner => userRole == Role.owner;
  bool get isEditor => userRole == Role.editor || userRole == Role.owner;
  bool get isViewer => userRole == Role.viewer;
  bool get isCommenter => userRole == Role.commenter;
  bool get canManage => canManageMembers || canManageRoles;
}

class PermissionNotifier extends StateNotifier<PermissionState> {
  final RbacService rbacService;
  final String spaceId;
  String? _currentUserId;

  PermissionNotifier({
    required this.rbacService,
    required this.spaceId,
  }) : super(const PermissionState());

  Future<void> refreshPermissions(String userId) async {
    if (_currentUserId == userId && state.userRole != null) {
      return;
    }

    _currentUserId = userId;
    state = state.copyWith(isLoading: true, error: null);

    try {
      final role = await rbacService.getUserRole(spaceId, userId);
      if (role == null) {
        state = state.copyWith(
          isLoading: false,
          userRole: null,
          permissions: const {},
          allowedActions: const {},
        );
        return;
      }

      final permissions = await rbacService.getUserPermissions(spaceId, userId);
      final actions = await rbacService.getUserActions(spaceId, userId);

      state = state.copyWith(
        isLoading: false,
        userRole: role,
        permissions: permissions,
        allowedActions: actions,
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.toString(),
      );
    }
  }

  Future<bool> checkPermission(Permission permission) async {
    if (_currentUserId == null) {
      return false;
    }
    return rbacService.hasPermissionForSpace(
        spaceId, _currentUserId!, permission);
  }

  Future<bool> checkAction(ActionType action) async {
    if (_currentUserId == null) {
      return false;
    }
    return rbacService.canPerformAction(spaceId, _currentUserId!, action);
  }

  bool canEditDocument(String documentId) {
    return state.canEdit;
  }

  bool canDeleteDocument(String documentId) {
    return state.canDelete;
  }

  bool canCommentOnDocument(String documentId) {
    return state.canComment;
  }

  bool canShareDocument(String documentId) {
    return state.canShare;
  }

  bool canManageSpaceMembers() {
    return state.canManageMembers;
  }

  bool canManageSpaceRoles() {
    return state.canManageRoles;
  }

  bool canDeleteSpace() {
    return state.allowedActions.contains(ActionType.deleteSpace);
  }

  void clear() {
    _currentUserId = null;
    state = const PermissionState();
  }
}

final class CurrentSpacePermissionProvider
    extends StateNotifier<PermissionState> {
  final RbacService rbacService;
  final String spaceId;
  final String userId;

  CurrentSpacePermissionProvider({
    required this.rbacService,
    required this.spaceId,
    required this.userId,
  }) : super(const PermissionState()) {
    _loadPermissions();
  }

  Future<void> _loadPermissions() async {
    state = state.copyWith(isLoading: true);

    try {
      final role = await rbacService.getUserRole(spaceId, userId);
      if (role == null) {
        state = state.copyWith(isLoading: false);
        return;
      }

      final permissions = await rbacService.getUserPermissions(spaceId, userId);
      final actions = await rbacService.getUserActions(spaceId, userId);

      state = state.copyWith(
        isLoading: false,
        userRole: role,
        permissions: permissions,
        allowedActions: actions,
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.toString(),
      );
    }
  }

  Future<void> refresh() async {
    await _loadPermissions();
  }

  bool get canEdit => state.canEdit;
  bool get canCreate => state.canCreate;
  bool get canDelete => state.canDelete;
  bool get canComment => state.canComment;
  bool get canShare => state.canShare;
  bool get canManageMembers => state.canManageMembers;
  bool get canManageRoles => state.canManageRoles;
  bool get isOwner => state.isOwner;
  bool get isEditor => state.isEditor;
  bool get isViewer => state.isViewer;
  bool get isCommenter => state.isCommenter;
}

StateNotifierProviderFamily<CurrentSpacePermissionProvider, PermissionState,
        String> currentSpacePermissionProvider =
    StateNotifierProvider.family<CurrentSpacePermissionProvider,
        PermissionState, String>((ref, String spaceId) {
  final rbacService = ref.read(rbacServiceProvider);
  final userId = ref.watch(currentUserIdProvider);

  if (userId == null) {
    throw StateError('User not authenticated');
  }

  return CurrentSpacePermissionProvider(
    rbacService: rbacService,
    spaceId: spaceId,
    userId: userId,
  );
});

final currentUserIdProvider = StateProvider<String?>((ref) => null);
