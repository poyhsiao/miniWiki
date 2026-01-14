use sqlx::{types::text, types::Uuid};

#[cfg(test)]
mod tests {
    use crate::permissions::{Role, Permission, ActionType};

    #[test]
    fn test_role_levels() {
        assert!(Role::Owner.level() > Role::Editor.level());
        assert!(Role::Editor.level() > Role::Commenter.level());
        assert!(Role::Commenter.level() > Role::Viewer.level());
    }

    #[test]
    fn test_owner_has_all_permissions() {
        let owner_role = Role::Owner;
        assert!(owner_role.has_permission(&Permission::ViewDocuments));
        assert!(owner_role.has_permission(&Permission::EditDocuments));
        assert!(owner_role.has_permission(&Permission::DeleteDocuments));
        assert!(owner_role.has_permission(&Permission::CreateDocuments));
        assert!(owner_role.has_permission(&Permission::Comment));
        assert!(owner_role.has_permission(&Permission::Share));
        assert!(owner_role.has_permission(&Permission::ManageMembers));
        assert!(owner_role.has_permission(&Permission::ManageRoles));
        assert!(owner_role.has_permission(&Permission::DeleteSpace));
    }

    #[test]
    fn test_editor_permissions() {
        let editor_role = Role::Editor;
        assert!(editor_role.has_permission(&Permission::ViewDocuments));
        assert!(editor_role.has_permission(&Permission::EditDocuments));
        assert!(!editor_role.has_permission(&Permission::DeleteDocuments));
        assert!(editor_role.has_permission(&Permission::CreateDocuments));
        assert!(editor_role.has_permission(&Permission::Comment));
        assert!(!editor_role.has_permission(&Permission::Share));
        assert!(!editor_role.has_permission(&Permission::ManageMembers));
        assert!(!editor_role.has_permission(&Permission::ManageRoles));
        assert!(!editor_role.has_permission(&Permission::DeleteSpace));
    }

    #[test]
    fn test_commenter_permissions() {
        let commenter_role = Role::Commenter;
        assert!(commenter_role.has_permission(&Permission::ViewDocuments));
        assert!(!commenter_role.has_permission(&Permission::EditDocuments));
        assert!(!commenter_role.has_permission(&Permission::DeleteDocuments));
        assert!(!commenter_role.has_permission(&Permission::CreateDocuments));
        assert!(commenter_role.has_permission(&Permission::Comment));
        assert!(!commenter_role.has_permission(&Permission::Share));
        assert!(!commenter_role.has_permission(&Permission::ManageMembers));
        assert!(!commenter_role.has_permission(&Permission::ManageRoles));
        assert!(!commenter_role.has_permission(&Permission::DeleteSpace));
    }

    #[test]
    fn test_viewer_permissions() {
        let viewer_role = Role::Viewer;
        assert!(viewer_role.has_permission(&Permission::ViewDocuments));
        assert!(!viewer_role.has_permission(&Permission::EditDocuments));
        assert!(!viewer_role.has_permission(&Permission::DeleteDocuments));
        assert!(!viewer_role.has_permission(&Permission::CreateDocuments));
        assert!(!viewer_role.has_permission(&Permission::Comment));
        assert!(!viewer_role.has_permission(&Permission::Share));
        assert!(!viewer_role.has_permission(&Permission::ManageMembers));
        assert!(!viewer_role.has_permission(&Permission::ManageRoles));
        assert!(!viewer_role.has_permission(&Permission::DeleteSpace));
    }

    #[test]
    fn test_role_assignment() {
        // Owner can assign any role
        assert!(Role::Owner.can_assign_role(&Role::Owner));
        assert!(Role::Owner.can_assign_role(&Role::Editor));
        assert!(Role::Owner.can_assign_role(&Role::Commenter));
        assert!(Role::Owner.can_assign_role(&Role::Viewer));

        // Editor can assign Commenter and Viewer but not Owner or Editor
        assert!(!Role::Editor.can_assign_role(&Role::Owner));
        assert!(!Role::Editor.can_assign_role(&Role::Editor));
        assert!(Role::Editor.can_assign_role(&Role::Commenter));
        assert!(Role::Editor.can_assign_role(&Role::Viewer));

        // Commenter and Viewer cannot assign any roles
        assert!(!Role::Commenter.can_assign_role(&Role::Owner));
        assert!(!Role::Commenter.can_assign_role(&Role::Editor));
        assert!(!Role::Commenter.can_assign_role(&Role::Commenter));
        assert!(!Role::Commenter.can_assign_role(&Role::Viewer));

        assert!(!Role::Viewer.can_assign_role(&Role::Owner));
        assert!(!Role::Viewer.can_assign_role(&Role::Editor));
        assert!(!Role::Viewer.can_assign_role(&Role::Commenter));
        assert!(!Role::Viewer.can_assign_role(&Role::Viewer));
    }

    #[test]
    fn test_action_permissions() {
        // Owner can delete documents
        assert!(ActionType::DeleteDocument
            .allowed_roles()
            .contains(&Role::Owner));
        assert!(ActionType::DeleteDocument
            .allowed_roles()
            .contains(&Role::Editor));

        // Viewer can view documents
        assert!(ActionType::ViewDocument
            .allowed_roles()
            .contains(&Role::Viewer));
        assert!(ActionType::DeleteDocument
            .allowed_roles()
            .contains(&Role::Viewer));
    }

    #[test]
    fn test_permission_from_string() {
        // Test valid permission strings
        assert_eq!(
            Permission::from_string("view_documents"),
            Some(Permission::ViewDocuments)
        );
        assert_eq!(
            Permission::from_string("view"),
            Some(Permission::ViewDocuments)
        );
        assert_eq!(
            Permission::from_string("edit"),
            Some(Permission::EditDocuments)
        );
        assert_eq!(
            Permission::from_string("delete"),
            Some(Permission::DeleteDocuments)
        );
        assert_eq!(
            Permission::from_string("comment"),
            Some(Permission::Comment)
        );

        // Test invalid permission string
        assert_eq!(Permission::from_string("invalid_permission"), None);
    }

    #[test]
    fn test_role_from_string() {
        // Test valid role strings
        assert_eq!(
            serde_json::from_str::<Role>("owner"),
            Role::Owner
        );
        assert_eq!(
            serde_json::from_str::<Role>("editor"),
            Role::Editor
        );
        assert_eq!(
            serde_json::from_str::<Role>("commenter"),
            Role::Commenter
        );
        assert_eq!(
            serde_json::from_str::<Role>("viewer"),
            Role::Viewer
        );

        // Test invalid role string
        assert!(serde_json::from_str::<Role>("invalid_role").is_err());
    }

    #[test]
    fn test_permission_required_actions() {
        // Each action maps to correct required permissions
        let create_actions = ActionType::CreateDocument.required_permissions();
        assert!(create_actions.contains(&Permission::CreateDocuments));
        assert_eq!(create_actions.len(), 1);

        let edit_actions = ActionType::EditDocument.required_permissions();
        assert!(edit_actions.contains(&Permission::EditDocuments));
        assert_eq!(edit_actions.len(), 1);

        let delete_actions = ActionType::DeleteDocument.required_permissions();
        assert!(delete_actions.contains(&Permission::DeleteDocuments));
        assert_eq!(delete_actions.len(), 1);

        let view_actions = ActionType::ViewDocument.required_permissions();
        assert!(view_actions.contains(&Permission::ViewDocuments));
        assert_eq!(view_actions.len(), 1);
    }
}
