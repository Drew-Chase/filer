#[cfg(test)]
mod tests {
    use crate::auth::auth_data::{CreateUserRequest, LoginRequest, LoginResponse, UpdateUserRequest, User};
    use crate::auth::permission_flags::PermissionFlags;
    use enumflags2::BitFlags;
    use sqlx::Row;
    use sqlx::sqlite::SqliteRow;

    #[test]
    fn test_create_user_request() {
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            permissions: vec!["Read".to_string(), "Write".to_string()],
        };

        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "password123");
        assert_eq!(request.permissions, vec!["Read".to_string(), "Write".to_string()]);
    }

    #[test]
    fn test_login_request() {
        // Test with remember flag set to true
        let request_with_remember = LoginRequest { username: "testuser".to_string(), password: "password123".to_string(), remember: Some(true) };

        assert_eq!(request_with_remember.username, "testuser");
        assert_eq!(request_with_remember.password, "password123");
        assert_eq!(request_with_remember.remember, Some(true));

        // Test with remember flag not set
        let request_without_remember = LoginRequest { username: "testuser".to_string(), password: "password123".to_string(), remember: None };

        assert_eq!(request_without_remember.username, "testuser");
        assert_eq!(request_without_remember.password, "password123");
        assert_eq!(request_without_remember.remember, None);
    }

    #[test]
    fn test_login_response() {
        let response = LoginResponse { token: "jwt-token-123".to_string(), username: "testuser".to_string() };

        assert_eq!(response.token, "jwt-token-123");
        assert_eq!(response.username, "testuser");
    }

    #[test]
    fn test_update_user_request() {
        // Test with both fields set
        let request_full = UpdateUserRequest {
            password: Some("newpassword123".to_string()),
            permissions: Some(vec!["Read".to_string(), "Write".to_string(), "Delete".to_string()]),
        };

        assert_eq!(request_full.password, Some("newpassword123".to_string()));
        assert_eq!(request_full.permissions, Some(vec!["Read".to_string(), "Write".to_string(), "Delete".to_string()]));

        // Test with only password set
        let request_password_only = UpdateUserRequest { password: Some("newpassword123".to_string()), permissions: None };

        assert_eq!(request_password_only.password, Some("newpassword123".to_string()));
        assert_eq!(request_password_only.permissions, None);

        // Test with only permissions set
        let request_permissions_only = UpdateUserRequest { password: None, permissions: Some(vec!["Read".to_string(), "Write".to_string()]) };

        assert_eq!(request_permissions_only.password, None);
        assert_eq!(request_permissions_only.permissions, Some(vec!["Read".to_string(), "Write".to_string()]));
    }

    #[test]
    fn test_user() {
        let permissions = BitFlags::from_bits_truncate((PermissionFlags::Read as u8) | (PermissionFlags::Write as u8));

        let user = User { id: 1, username: "testuser".to_string(), password: "hashedpassword123".to_string(), permissions };

        assert_eq!(user.id, 1);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.password, "hashedpassword123");
        assert!(user.permissions.contains(PermissionFlags::Read));
        assert!(user.permissions.contains(PermissionFlags::Write));
        assert!(!user.permissions.contains(PermissionFlags::Delete));
        assert!(!user.permissions.contains(PermissionFlags::Create));
        assert!(!user.permissions.contains(PermissionFlags::Upload));
        assert!(!user.permissions.contains(PermissionFlags::Download));
    }
}

#[cfg(test)]
mod permission_tests {
    use crate::auth::permission_flags::PermissionFlags;
    use enumflags2::BitFlags;

    #[test]
    fn test_permission_flags_default() {
        let default_permission = PermissionFlags::default();
        assert_eq!(default_permission, PermissionFlags::Read);
    }

    #[test]
    fn test_permission_flags_all() {
        let all_permissions = PermissionFlags::all();

        // Check that all permission bits are set
        assert_eq!(all_permissions & (PermissionFlags::Read as u8), PermissionFlags::Read as u8);
        assert_eq!(all_permissions & (PermissionFlags::Write as u8), PermissionFlags::Write as u8);
        assert_eq!(all_permissions & (PermissionFlags::Delete as u8), PermissionFlags::Delete as u8);
        assert_eq!(all_permissions & (PermissionFlags::Create as u8), PermissionFlags::Create as u8);
        assert_eq!(all_permissions & (PermissionFlags::Upload as u8), PermissionFlags::Upload as u8);
        assert_eq!(all_permissions & (PermissionFlags::Download as u8), PermissionFlags::Download as u8);
    }

    #[test]
    fn test_permission_flags_from_strings_valid() {
        let permissions = vec!["Read".to_string(), "Write".to_string(), "Delete".to_string()];

        let flags = PermissionFlags::from_strings(&permissions).unwrap();

        assert!(flags.contains(PermissionFlags::Read));
        assert!(flags.contains(PermissionFlags::Write));
        assert!(flags.contains(PermissionFlags::Delete));
        assert!(!flags.contains(PermissionFlags::Create));
        assert!(!flags.contains(PermissionFlags::Upload));
        assert!(!flags.contains(PermissionFlags::Download));
    }

    #[test]
    fn test_permission_flags_from_strings_empty() {
        let permissions: Vec<String> = vec![];
        let flags = PermissionFlags::from_strings(&permissions).unwrap();

        assert_eq!(flags, BitFlags::empty());
    }

    #[test]
    fn test_permission_flags_from_strings_invalid() {
        let permissions = vec!["Read".to_string(), "InvalidPermission".to_string()];

        let result = PermissionFlags::from_strings(&permissions);
        assert!(result.is_err());
    }

    #[test]
    fn test_permission_flags_bitwise_operations() {
        // Test bitwise OR
        let read_write = PermissionFlags::Read | PermissionFlags::Write;
        assert!(read_write.contains(PermissionFlags::Read));
        assert!(read_write.contains(PermissionFlags::Write));
        assert!(!read_write.contains(PermissionFlags::Delete));

        // Test bitwise AND
        let read_write_delete = PermissionFlags::Read | PermissionFlags::Write | PermissionFlags::Delete;
        let read_write_mask = PermissionFlags::Read | PermissionFlags::Write;
        let result = read_write_delete & read_write_mask;

        assert!(result.contains(PermissionFlags::Read));
        assert!(result.contains(PermissionFlags::Write));
        assert!(!result.contains(PermissionFlags::Delete));

        // Test bitwise XOR
        let read_write = PermissionFlags::Read | PermissionFlags::Write;
        let write_delete = PermissionFlags::Write | PermissionFlags::Delete;
        let result = read_write ^ write_delete;

        assert!(result.contains(PermissionFlags::Read));
        assert!(!result.contains(PermissionFlags::Write)); // Both have Write, so it's removed by XOR
        assert!(result.contains(PermissionFlags::Delete));
    }
}
