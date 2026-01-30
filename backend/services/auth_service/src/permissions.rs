use serde::{Deserialize, Serialize};
use std::fmt;

/// Role enum representing user roles in a space
///
/// Each role has a specific level and associated permissions.
/// Roles are ordered by privilege level: Owner > Editor > Commenter > Viewer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Owner = 100,
    Editor = 75,
    Commenter = 50,
    Viewer = 25,
}

impl Role {
    pub fn level(&self) -> i32 {
        match self {
            Role::Owner => 100,
            Role::Editor => 75,
            Role::Commenter => 50,
            Role::Viewer => 25,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Role::Owner => "Owner",
            Role::Editor => "Editor",
            Role::Commenter => "Commenter",
            Role::Viewer => "Viewer",
        }
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        permission.allowed_roles().contains(self)
    }

    pub fn can_assign_role(&self, target_role: &Role) -> bool {
        if *self == Role::Owner {
            return true;
        }
        if *self == Role::Editor {
            return target_role.level() < self.level() && *target_role != Role::Owner;
        }
        false
    }

    pub fn can_perform_action(&self, action: &ActionType) -> bool {
        action.allowed_roles().contains(self)
    }

    pub fn from_str(role: &str) -> Option<Role> {
        match role.to_lowercase().as_str() {
            "owner" => Some(Role::Owner),
            "editor" => Some(Role::Editor),
            "commenter" => Some(Role::Commenter),
            "viewer" => Some(Role::Viewer),
            _ => None,
        }
    }
}

/// Permission enum representing granular permissions
///
/// Each permission specifies which roles are allowed to perform it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    #[serde(rename = "view_documents")]
    ViewDocuments,
    #[serde(rename = "create_documents")]
    CreateDocuments,
    #[serde(rename = "edit_documents")]
    EditDocuments,
    #[serde(rename = "delete_documents")]
    DeleteDocuments,
    #[serde(rename = "comment")]
    Comment,
    #[serde(rename = "share")]
    Share,
    #[serde(rename = "manage_members")]
    ManageMembers,
    #[serde(rename = "manage_roles")]
    ManageRoles,
    #[serde(rename = "delete_space")]
    DeleteSpace,
}

impl Permission {
    pub fn allowed_roles(&self) -> Vec<Role> {
        match self {
            Permission::ViewDocuments => vec![Role::Owner, Role::Editor, Role::Commenter, Role::Viewer],
            Permission::CreateDocuments => vec![Role::Owner, Role::Editor],
            Permission::EditDocuments => vec![Role::Owner, Role::Editor],
            Permission::DeleteDocuments => vec![Role::Owner],
            Permission::Comment => vec![Role::Owner, Role::Editor, Role::Commenter],
            Permission::Share => vec![Role::Owner, Role::Editor],
            Permission::ManageMembers => vec![Role::Owner, Role::Editor],
            Permission::ManageRoles => vec![Role::Owner],
            Permission::DeleteSpace => vec![Role::Owner],
        }
    }

    pub fn from_string(permission: &str) -> Option<Permission> {
        match permission.to_lowercase().as_str() {
            "view" | "view_documents" => Some(Permission::ViewDocuments),
            "create" | "create_documents" => Some(Permission::CreateDocuments),
            "edit" | "edit_documents" => Some(Permission::EditDocuments),
            "delete" | "delete_documents" => Some(Permission::DeleteDocuments),
            "comment" => Some(Permission::Comment),
            "share" => Some(Permission::Share),
            "manage_members" => Some(Permission::ManageMembers),
            "manage_roles" => Some(Permission::ManageRoles),
            "delete_space" => Some(Permission::DeleteSpace),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::ViewDocuments => "view_documents",
            Permission::CreateDocuments => "create_documents",
            Permission::EditDocuments => "edit_documents",
            Permission::DeleteDocuments => "delete_documents",
            Permission::Comment => "comment",
            Permission::Share => "share",
            Permission::ManageMembers => "manage_members",
            Permission::ManageRoles => "manage_roles",
            Permission::DeleteSpace => "delete_space",
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// ActionType enum representing high-level user actions
///
/// Actions are composed of one or more permissions and represent
/// common user workflows in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    #[serde(rename = "view_document")]
    ViewDocument,
    #[serde(rename = "create_document")]
    CreateDocument,
    #[serde(rename = "edit_document")]
    EditDocument,
    #[serde(rename = "delete_document")]
    DeleteDocument,
    #[serde(rename = "comment")]
    Comment,
    #[serde(rename = "share")]
    Share,
    #[serde(rename = "manage_members")]
    ManageMembers,
    #[serde(rename = "manage_roles")]
    ManageRoles,
    #[serde(rename = "delete_space")]
    DeleteSpace,
    #[serde(rename = "invite_member")]
    InviteMember,
    #[serde(rename = "remove_member")]
    RemoveMember,
    #[serde(rename = "view_members")]
    ViewMembers,
    #[serde(rename = "export_document")]
    ExportDocument,
    #[serde(rename = "view_version_history")]
    ViewVersionHistory,
    #[serde(rename = "restore_version")]
    RestoreVersion,
}

impl ActionType {
    pub fn allowed_roles(&self) -> Vec<Role> {
        match self {
            ActionType::ViewDocument => vec![Role::Owner, Role::Editor, Role::Commenter, Role::Viewer],
            ActionType::CreateDocument => vec![Role::Owner, Role::Editor],
            ActionType::EditDocument => vec![Role::Owner, Role::Editor],
            ActionType::DeleteDocument => vec![Role::Owner],
            ActionType::Comment => vec![Role::Owner, Role::Editor, Role::Commenter],
            ActionType::Share => vec![Role::Owner, Role::Editor],
            ActionType::ManageMembers => vec![Role::Owner, Role::Editor],
            ActionType::ManageRoles => vec![Role::Owner],
            ActionType::DeleteSpace => vec![Role::Owner],
            ActionType::InviteMember => vec![Role::Owner, Role::Editor],
            ActionType::RemoveMember => vec![Role::Owner, Role::Editor],
            ActionType::ViewMembers => vec![Role::Owner, Role::Editor, Role::Commenter, Role::Viewer],
            ActionType::ExportDocument => vec![Role::Owner, Role::Editor, Role::Commenter, Role::Viewer],
            ActionType::ViewVersionHistory => vec![Role::Owner, Role::Editor, Role::Commenter, Role::Viewer],
            ActionType::RestoreVersion => vec![Role::Owner, Role::Editor],
        }
    }

    pub fn required_permissions(&self) -> Vec<Permission> {
        match self {
            ActionType::ViewDocument => vec![Permission::ViewDocuments],
            ActionType::CreateDocument => vec![Permission::CreateDocuments],
            ActionType::EditDocument => vec![Permission::EditDocuments],
            ActionType::DeleteDocument => vec![Permission::DeleteDocuments],
            ActionType::Comment => vec![Permission::Comment],
            ActionType::Share => vec![Permission::Share],
            ActionType::ManageMembers => vec![Permission::ManageMembers],
            ActionType::ManageRoles => vec![Permission::ManageRoles],
            ActionType::DeleteSpace => vec![Permission::DeleteSpace],
            ActionType::InviteMember => vec![Permission::ManageMembers],
            ActionType::RemoveMember => vec![Permission::ManageMembers],
            ActionType::ViewMembers => vec![Permission::ViewDocuments],
            ActionType::ExportDocument => vec![Permission::ViewDocuments],
            ActionType::ViewVersionHistory => vec![Permission::ViewDocuments],
            ActionType::RestoreVersion => vec![Permission::EditDocuments],
        }
    }
}

/// RBAC configuration constants
pub struct RbacConfig;

impl RbacConfig {
    pub const MAX_MEMBERS_PER_SPACE: i32 = 1000;
    pub const MAX_INVITES_PER_DAY: i32 = 50;
    pub const DEFAULT_ROLE: Role = Role::Viewer;

    pub fn get_permissions_for_role(role: &Role) -> Vec<Permission> {
        match role {
            Role::Owner => vec![
                Permission::ViewDocuments,
                Permission::CreateDocuments,
                Permission::EditDocuments,
                Permission::DeleteDocuments,
                Permission::Comment,
                Permission::Share,
                Permission::ManageMembers,
                Permission::ManageRoles,
                Permission::DeleteSpace,
            ],
            Role::Editor => vec![
                Permission::ViewDocuments,
                Permission::CreateDocuments,
                Permission::EditDocuments,
                Permission::Comment,
                Permission::Share,
                Permission::ManageMembers,
            ],
            Role::Commenter => vec![Permission::ViewDocuments, Permission::Comment],
            Role::Viewer => vec![Permission::ViewDocuments],
        }
    }

    pub fn can_role_be_modified_by(target_role: &Role, modifier_role: &Role) -> bool {
        if *target_role == Role::Owner {
            return false;
        }
        modifier_role.level() > target_role.level()
    }

    pub fn is_valid_role_transition(from_role: &Role, to_role: &Role) -> bool {
        if *to_role == Role::Owner {
            return false;
        }
        from_role.level() >= to_role.level()
    }

    pub fn get_assignable_roles(assigner_role: &Role) -> Vec<Role> {
        vec![Role::Owner, Role::Editor, Role::Commenter, Role::Viewer]
            .into_iter()
            .filter(|role| assigner_role.can_assign_role(role))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_levels() {
        assert!(Role::Owner.level() > Role::Editor.level());
        assert!(Role::Editor.level() > Role::Commenter.level());
        assert!(Role::Commenter.level() > Role::Viewer.level());
    }

    #[test]
    fn test_owner_has_all_permissions() {
        assert!(Role::Owner.has_permission(&Permission::ViewDocuments));
        assert!(Role::Owner.has_permission(&Permission::EditDocuments));
        assert!(Role::Owner.has_permission(&Permission::DeleteDocuments));
        assert!(Role::Owner.has_permission(&Permission::CreateDocuments));
    }

    #[test]
    fn test_editor_permissions() {
        assert!(Role::Editor.has_permission(&Permission::ViewDocuments));
        assert!(Role::Editor.has_permission(&Permission::EditDocuments));
        assert!(!Role::Editor.has_permission(&Permission::DeleteDocuments));
        assert!(Role::Editor.has_permission(&Permission::CreateDocuments));
    }

    #[test]
    fn test_role_assignment() {
        assert!(Role::Owner.can_assign_role(&Role::Editor));
        assert!(Role::Owner.can_assign_role(&Role::Commenter));
        assert!(Role::Owner.can_assign_role(&Role::Viewer));
        assert!(!Role::Editor.can_assign_role(&Role::Owner));
        assert!(!Role::Editor.can_assign_role(&Role::Editor));
        assert!(Role::Editor.can_assign_role(&Role::Commenter));
        assert!(Role::Editor.can_assign_role(&Role::Viewer));
        assert!(!Role::Commenter.can_assign_role(&Role::Editor));
        assert!(!Role::Viewer.can_assign_role(&Role::Commenter));
    }

    #[test]
    fn test_action_permissions() {
        assert!(ActionType::ViewDocument.allowed_roles().contains(&Role::Viewer));
        assert!(ActionType::DeleteDocument.allowed_roles().contains(&Role::Owner));
        assert!(!ActionType::DeleteDocument.allowed_roles().contains(&Role::Editor));
    }
}
