use auth_service::password::{hash_password, verify_password};

#[test]
fn test_verify_password_success() {
    let password = "TestPassword123!";
    let hash = hash_password(password).expect("Failed to hash password");

    let result = verify_password(password, &hash);
    assert!(result.is_ok(), "verify_password should return Ok");
    assert_eq!(result.unwrap(), true, "Password should match");
}

#[test]
fn test_verify_password_failure() {
    let password = "TestPassword123!";
    let wrong_password = "WrongPassword456!";
    let hash = hash_password(password).expect("Failed to hash password");

    let result = verify_password(wrong_password, &hash);
    assert!(result.is_ok(), "verify_password should return Ok even for wrong password");
    assert_eq!(result.unwrap(), false, "Password should not match");
}

#[test]
fn test_verify_password_corrupted_hash() {
    let password = "TestPassword123!";
    let corrupted_hash = "not_a_valid_bcrypt_hash";

    let result = verify_password(password, corrupted_hash);
    assert!(result.is_err(), "verify_password should return Err for corrupted hash");
}

#[test]
fn test_verify_password_empty_hash() {
    let password = "TestPassword123!";
    let empty_hash = "";

    let result = verify_password(password, empty_hash);
    assert!(result.is_err(), "verify_password should return Err for empty hash");
}
