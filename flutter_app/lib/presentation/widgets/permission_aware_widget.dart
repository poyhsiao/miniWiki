import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/value_objects/role.dart';
import 'package:miniwiki/presentation/providers/permission_provider.dart';

/// A widget that conditionally displays its child based on user permissions.
///
/// This widget is the primary mechanism for role-based UI filtering in the app.
/// It wraps any widget and only displays it if the user has the required
/// permission or role.
///
/// Example usage:
/// ```dart
/// PermissionAwareWidget(
///   permission: Permission.editDocuments,
///   child: ElevatedButton(
///     onPressed: () => _editDocument(),
///     child: Text('Edit'),
///   ),
///   fallback: Text('You cannot edit this document'),
/// )
/// ```
///
/// See also:
/// - [RoleBasedWidget] for role-specific UI
/// - [ActionBasedWidget] for action-specific UI
class PermissionAwareWidget extends ConsumerWidget {
  /// The permission required to display the child widget.
  final Permission permission;

  /// The widget to display when the user has the required permission.
  final Widget child;

  /// Optional fallback widget to display when permission is denied.
  final Widget? fallback;

  /// Whether to show the fallback or hide completely when permission is denied.
  /// If true (default), shows the fallback widget.
  /// If false, returns an empty SizedBox when permission is denied.
  final bool showFallback;

  /// Optional space ID to check permissions for (uses current space if not provided).
  final String? spaceId;

  const PermissionAwareWidget({
    required this.permission, required this.child, super.key,
    this.fallback,
    this.showFallback = true,
    this.spaceId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final permissionState = ref.watch(permissionProvider(spaceId ?? ''));

    if (permissionState.isLoading) {
      return const SizedBox.shrink();
    }

    final hasPermission = permissionState.permissions.contains(permission);

    if (hasPermission) {
      return child;
    }

    if (showFallback && fallback != null) {
      return fallback!;
    }

    return const SizedBox.shrink();
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(EnumProperty<Permission>('permission', permission));
    properties.add(DiagnosticsProperty<bool>('showFallback', showFallback));
    properties.add(StringProperty('spaceId', spaceId));
  }
}

/// A widget that displays content only for specific roles.
///
/// This is useful when you want to show different UI for different roles.
/// Unlike [PermissionAwareWidget] which checks permissions, this checks
/// for exact role matches.
///
/// Example usage:
/// ```dart
/// RoleBasedWidget(
///   allowedRoles: [Role.owner, Role.editor],
///   child: AdminPanel(),
///   fallback: Text('You do not have permission to view this panel'),
/// )
/// ```
class RoleBasedWidget extends ConsumerWidget {
  /// Roles that are allowed to see the child widget.
  final List<Role> allowedRoles;

  /// The widget to display when the user's role is in [allowedRoles].
  final Widget child;

  /// Optional fallback widget when role is not allowed.
  final Widget? fallback;

  /// Optional space ID (uses current space if not provided).
  final String? spaceId;

  const RoleBasedWidget({
    required this.allowedRoles, required this.child, super.key,
    this.fallback,
    this.spaceId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final permissionState = ref.watch(permissionProvider(spaceId ?? ''));

    if (permissionState.isLoading) {
      return const SizedBox.shrink();
    }

    final userRole = permissionState.userRole;
    if (userRole != null && allowedRoles.contains(userRole)) {
      return child;
    }

    if (fallback != null) {
      return fallback!;
    }

    return const SizedBox.shrink();
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(IterableProperty<Role>('allowedRoles', allowedRoles));
    properties.add(StringProperty('spaceId', spaceId));
  }
}

/// A widget that conditionally displays content based on action availability.
///
/// This is useful when you want to check if a user can perform a specific
/// action rather than checking a specific permission.
///
/// Example usage:
/// ```dart
/// ActionBasedWidget(
///   action: ActionType.deleteDocument,
///   child: DeleteButton(),
/// )
/// ```
class ActionBasedWidget extends ConsumerWidget {
  /// The action that must be allowed.
  final ActionType action;

  /// The widget to display when the action is allowed.
  final Widget child;

  /// Optional fallback widget when action is not allowed.
  final Widget? fallback;

  /// Whether to hide completely when action is not allowed.
  final bool hideOnDeny;

  /// Optional space ID (uses current space if not provided).
  final String? spaceId;

  const ActionBasedWidget({
    required this.action, required this.child, super.key,
    this.fallback,
    this.hideOnDeny = false,
    this.spaceId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final permissionState = ref.watch(permissionProvider(spaceId ?? ''));

    if (permissionState.isLoading) {
      return hideOnDeny ? const SizedBox.shrink() : const SizedBox.shrink();
    }

    final canPerform = permissionState.allowedActions.contains(action);

    if (canPerform) {
      return child;
    }

    if (hideOnDeny) {
      return const SizedBox.shrink();
    }

    if (fallback != null) {
      return fallback!;
    }

    return const SizedBox.shrink();
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(EnumProperty<ActionType>('action', action));
    properties.add(DiagnosticsProperty<bool>('hideOnDeny', hideOnDeny));
    properties.add(StringProperty('spaceId', spaceId));
  }
}

/// A widget that shows different content based on the user's role.
///
/// This is useful for displaying role-specific UI hints or instructions.
///
/// Example:
/// ```dart
/// RoleIndicator(
///   builder: (role) {
///     switch (role) {
///       case Role.owner:
///         return Chip(label: Text('Owner'));
///       case Role.editor:
///         return Chip(label: Text('Editor'));
///       default:
///         return SizedBox.shrink();
///     }
///   },
/// )
/// ```
class RoleIndicator extends ConsumerWidget {
  /// Builder function that receives the current role and returns a widget.
  final Widget Function(Role? role) builder;

  /// Optional space ID (uses current space if not provided).
  final String? spaceId;

  const RoleIndicator({
    required this.builder, super.key,
    this.spaceId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final permissionState = ref.watch(permissionProvider(spaceId ?? ''));
    return builder(permissionState.userRole);
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(ObjectFlagProperty<Widget Function(Role? role)>.has('builder', builder));
    properties.add(StringProperty('spaceId', spaceId));
  }
}

/// A wrapper that shows a lock icon and disables interaction when permission is denied.
///
/// This is useful for form fields, buttons, or other interactive elements
/// that should be visibly disabled rather than hidden.
class PermissionLockedWidget extends StatelessWidget {
  /// The permission required for this widget.
  final Permission permission;

  /// The child widget to wrap.
  final Widget child;

  /// Message to show in the tooltip when locked.
  final String? lockMessage;

  /// Optional space ID (uses current space if not provided).
  final String? spaceId;

  const PermissionLockedWidget({
    required this.permission, required this.child, super.key,
    this.lockMessage,
    this.spaceId,
  });

  @override
  Widget build(BuildContext context) => Consumer(
      builder: (context, ref, _) {
        final permissionState = ref.watch(permissionProvider(spaceId ?? ''));
        final hasPermission = permissionState.permissions.contains(permission);

        return Tooltip(
          message: hasPermission
              ? lockMessage ?? ''
              : lockMessage ?? 'You do not have permission for this action',
          child: AbsorbPointer(
            absorbing: !hasPermission,
            child: Opacity(
              opacity: hasPermission ? 1.0 : 0.5,
              child: child,
            ),
          ),
        );
      },
    );

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(EnumProperty<Permission>('permission', permission));
    properties.add(StringProperty('lockMessage', lockMessage));
    properties.add(StringProperty('spaceId', spaceId));
  }
}

/// A read-only text field that shows when editing is not permitted.
///
/// This provides a consistent UI pattern for displaying content that
/// the user cannot edit.
class PermissionReadOnlyField extends StatelessWidget {
  /// The label for this field.
  final String label;

  /// The current value of the field.
  final String value;

  /// Whether the user can edit this field.
  final bool canEdit;

  /// Optional icon to display.
  final Widget? icon;

  /// Message shown when user cannot edit.
  final String? editMessage;

  const PermissionReadOnlyField({
    required this.label, required this.value, super.key,
    this.canEdit = false,
    this.icon,
    this.editMessage = 'You do not have permission to edit this field',
  });

  @override
  Widget build(BuildContext context) => Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            if (icon != null) icon!,
            const SizedBox(width: 8),
            Text(
              label,
              style: Theme.of(context).textTheme.labelMedium?.copyWith(
                    color: canEdit
                        ? Theme.of(context).colorScheme.onSurface
                        : Theme.of(context).colorScheme.outline,
                  ),
            ),
            if (!canEdit) ...[
              const SizedBox(width: 8),
              Icon(
                Icons.lock_outline,
                size: 14,
                color: Theme.of(context).colorScheme.outline,
              ),
            ],
          ],
        ),
        const SizedBox(height: 4),
        TextFormField(
          initialValue: value,
          readOnly: true,
          maxLines: null,
          decoration: InputDecoration(
            filled: !canEdit,
            fillColor: canEdit
                ? null
                : Theme.of(context).colorScheme.surfaceContainerHighest.withValues(alpha: 0.5),
            border: const OutlineInputBorder(),
            helperText: canEdit ? null : editMessage,
            helperMaxLines: 2,
          ),
        ),
      ],
    );

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('label', label));
    properties.add(StringProperty('value', value));
    properties.add(DiagnosticsProperty<bool>('canEdit', canEdit));
    properties.add(StringProperty('editMessage', editMessage));
  }
}

/// A elevated button that shows a dialog when clicked and user lacks permission.
///
/// This provides a better UX than simply disabling the button by
/// explaining why the action is not available.
class PermissionGuardedButton extends ConsumerWidget {
  /// The permission required for this action.
  final Permission permission;

  /// The callback to execute when clicked.
  final VoidCallback? onPressed;

  /// The button child.
  final Widget child;

  /// Button style (uses elevated style by default).
  final ButtonStyle? style;

  /// Whether the button is loading.
  final bool isLoading;

  /// Message to show in the permission denied dialog.
  final String? deniedMessage;

  /// Title for the permission denied dialog.
  final String? dialogTitle;

  /// Optional space ID (uses current space if not provided).
  final String? spaceId;

  const PermissionGuardedButton({
    required this.permission, required this.child, super.key,
    this.onPressed,
    this.style,
    this.isLoading = false,
    this.deniedMessage,
    this.dialogTitle,
    this.spaceId,
  });

  Future<void> _showDeniedDialog(BuildContext context) async {
    await showDialog(
      context: context,
      builder: (BuildContext context) => AlertDialog(
        title: Text(dialogTitle ?? 'Permission Denied'),
        content: Text(deniedMessage ??
            'You do not have permission to perform this action. '
                'Please contact the space owner if you believe this is an error.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final permissionState = ref.watch(permissionProvider(spaceId ?? ''));
    final hasPermission = permissionState.permissions.contains(permission);

    return ElevatedButton(
      style: style,
      onPressed: isLoading
          ? null
          : hasPermission
              ? onPressed
              : () => _showDeniedDialog(context),
      child: child,
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(EnumProperty<Permission>('permission', permission));
    properties.add(ObjectFlagProperty<VoidCallback?>.has('onPressed', onPressed));
    properties.add(DiagnosticsProperty<ButtonStyle?>('style', style));
    properties.add(DiagnosticsProperty<bool>('isLoading', isLoading));
    properties.add(StringProperty('deniedMessage', deniedMessage));
    properties.add(StringProperty('dialogTitle', dialogTitle));
    properties.add(StringProperty('spaceId', spaceId));
  }
}

/// Extension on Widget for quick permission-based wrapping.
extension PermissionAwareExtension on Widget {
  /// Wrap this widget to only show if user has the specified permission.
  Widget withPermission(
    Permission permission, {
    Widget? fallback,
    String? spaceId,
  }) => PermissionAwareWidget(
      permission: permission,
      fallback: fallback,
      spaceId: spaceId,
      child: this,
    );

  /// Wrap this widget to only show for specific roles.
  Widget withRole(
    List<Role> allowedRoles, {
    Widget? fallback,
    String? spaceId,
  }) => RoleBasedWidget(
      allowedRoles: allowedRoles,
      fallback: fallback,
      spaceId: spaceId,
      child: this,
    );

  /// Wrap this widget to only show if user can perform the action.
  Widget withAction(
    ActionType action, {
    Widget? fallback,
    bool hideOnDeny = false,
    String? spaceId,
  }) => ActionBasedWidget(
      action: action,
      fallback: fallback,
      hideOnDeny: hideOnDeny,
      spaceId: spaceId,
      child: this,
    );

  /// Wrap this widget to lock it when permission is denied.
  Widget lockedWith(
    Permission permission, {
    String? lockMessage,
    String? spaceId,
  }) => PermissionLockedWidget(
      permission: permission,
      lockMessage: lockMessage,
      spaceId: spaceId,
      child: this,
    );
}
