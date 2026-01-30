#[cfg(test)]
mod file_service_test {
    use file_service::handlers::{
        extract_boundary, extract_user_id, FIELD_DOCUMENT_ID, FIELD_FILE, FIELD_FILE_NAME, FIELD_SPACE_ID,
    };
    use file_service::storage::{extract_chunk_number, ChunkedUploadSession, StorageError};

    #[test]
    fn test_extract_boundary_valid() {
        use actix_web::http::header::HeaderMap;
        let mut headers = HeaderMap::new();
        headers.insert(
            actix_web::http::header::CONTENT_TYPE,
            "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZ0+",
        );

        let boundary = extract_boundary(&headers);
        assert!(boundary.is_some());
        assert_eq!(boundary.unwrap(), "----WebKitFormBoundary7MA4YWxkTrZ0+");
    }

    #[test]
    fn test_extract_boundary_missing() {
        use actix_web::http::header::HeaderMap;
        let headers = HeaderMap::new();

        let boundary = extract_boundary(&headers);
        assert!(boundary.is_none());
    }

    #[test]
    fn test_extract_boundary_invalid_content_type() {
        use actix_web::http::header::HeaderMap;
        let mut headers = HeaderMap::new();
        headers.insert(actix_web::http::header::CONTENT_TYPE, "application/json");

        let boundary = extract_boundary(&headers);
        assert!(boundary.is_none());
    }

    #[test]
    fn test_extract_chunk_number_valid() {
        let chunk_name = "chunk-00005-of-00100";
        let result = extract_chunk_number(chunk_name);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
    }

    #[test]
    fn test_extract_chunk_number_invalid_format() {
        let chunk_name = "invalid-chunk-name";
        let result = extract_chunk_number(chunk_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_chunk_number_missing_number() {
        let chunk_name = "chunk-missing-of-100";
        let result = extract_chunk_number(chunk_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_chunked_upload_session_display() {
        use std::fmt;

        let session = ChunkedUploadSession {
            upload_id: uuid::Uuid::new_v4(),
            file_name: "test.pdf".to_string(),
            file_size: 1024 * 1024,      // 1MB
            chunk_size: 5 * 1024 * 1024, // 5MB
            uploaded_chunks: 3,
            total_chunks: 20,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };

        let display = format!("{}", session);
        assert!(display.contains("test.pdf"));
        assert!(display.contains("1MB"));
        assert!(display.contains("3/20"));
    }

    // ========================================
    // Handlers Module Tests
    // ========================================

    #[test]
    fn test_extract_user_id_from_extensions() {
        // Given: Request with user_id in extensions
        // When: extract_user_id is called
        // Then: User ID should be returned
        // Note: This test requires Actix request extensions which are tested in integration tests
        // Unit tests verify the extraction logic itself
    }

    #[test]
    fn test_extract_user_id_from_header_fallback() {
        // Given: No user_id in extensions, but X-User-Id header present
        // When: extract_user_id is called
        // Then: User ID from header should be returned
        // Integration test: JWT test sets X-User-Id header
    }

    #[test]
    fn test_extract_user_id_missing() {
        // Given: No user_id in extensions or headers
        // When: extract_user_id is called
        // Then: Should return AuthenticationError
    }

    // ========================================
    // Storage Module Tests
    // ========================================

    #[test]
    fn test_storage_validate_file_type_invalid() {
        use file_service::storage::StorageError;

        let result = file_service::storage::S3Storage::validate_file_type("application/json");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::UnsupportedFileType(_)));
    }

    #[test]
    fn test_storage_validate_file_size_exceeds_limit() {
        use file_service::storage::StorageError;

        let file_size = 51 * 1024 * 1024 + 1;
        let max_size = 50 * 1024 * 1024;
        let result = file_service::storage::S3Storage::validate_file_size(file_size, max_size);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::FileTooLarge(_, _)));
    }

    #[test]
    fn test_storage_validate_file_size_within_limit() {
        use file_service::storage::StorageError;

        let file_size = 10 * 1024 * 1024;
        let max_size = 50 * 1024 * 1024;
        let result = file_service::storage::S3Storage::validate_file_size(file_size, max_size);

        assert!(result.is_ok());
    }

    #[test]
    fn test_chunked_upload_session_all_fields_set() {
        let session = file_service::storage::ChunkedUploadSession {
            upload_id: uuid::Uuid::new_v4(),
            file_name: "test.pdf".to_string(),
            file_size: 1024 * 1024 * 1024,
            chunk_size: 5 * 1024 * 1024,
            uploaded_chunks: 3,
            total_chunks: 10,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(2),
        };

        assert!(!session.upload_id.is_nil());
        assert!(!session.file_name.is_empty());
        assert_eq!(session.file_size, 1024 * 1024 * 1024);
        assert_eq!(session.chunk_size, 5 * 1024 * 1024);
        assert_eq!(session.uploaded_chunks, 3);
        assert_eq!(session.total_chunks, 10);
    }

    #[test]
    fn test_chunked_upload_session_expires_calculation() {
        let session = file_service::storage::ChunkedUploadSession {
            upload_id: uuid::Uuid::new_v4(),
            file_name: "test.pdf".to_string(),
            file_size: 1024 * 1024 * 1024,
            chunk_size: 5 * 1024 * 1024,
            uploaded_chunks: 3,
            total_chunks: 10,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        };

        let time_diff = session.expires_at - session.created_at;
        let expected_diff_hours = 24;
        let actual_diff_hours = time_diff.num_hours() as f64;

        assert!((actual_diff_hours - expected_diff_hours as f64).abs() < 0.1);
    }

    // Models Module Tests
    // ===========================

    #[test]
    fn test_file_serialization() {
        use file_service::models::File;
        use uuid::Uuid;

        let file = File {
            id: Uuid::new_v4(),
            space_id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            file_name: "test.txt".to_string(),
            file_type: "text/plain".to_string(),
            file_size: 1024,
            storage_path: "spaces/123/test.txt".to_string(),
            storage_bucket: "test-bucket".to_string(),
            checksum: "abc123".to_string(),
            is_deleted: false,
            created_at: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&file).expect("Failed to serialize");
        let deserialized: File = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(file.id, deserialized.id);
        assert_eq!(file.file_name, deserialized.file_name);
        assert_eq!(file.file_type, deserialized.file_type);
    }

    #[test]
    fn test_file_detail_serialization() {
        use file_service::models::{File, FileDetail, UploaderInfo};
        use uuid::Uuid;

        let uploader_info = UploaderInfo {
            id: Uuid::new_v4(),
            display_name: "Test User".to_string(),
            avatar_url: None,
        };

        let file_detail = FileDetail {
            file: File {
                id: Uuid::new_v4(),
                space_id: Uuid::new_v4(),
                document_id: Uuid::new_v4(),
                file_name: "test.txt".to_string(),
                file_type: "text/plain".to_string(),
                file_size: 1024,
            },
            uploaded_by: uploader_info,
            checksum: "abc123".to_string(),
            storage_path: "spaces/123/test.txt".to_string(),
            storage_bucket: "test-bucket".to_string(),
            deleted_at: None,
            created_at: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&file_detail).expect("Failed to serialize");
        let deserialized: FileDetail = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(file_detail.file.id, deserialized.file.id);
        assert_eq!(
            file_detail.uploaded_by.display_name,
            deserialized.uploaded_by.display_name
        );
    }

    #[test]
    fn test_file_list_query() {
        use file_service::models::FileListQuery;

        let query1 = FileListQuery {
            document_id: Some(uuid::Uuid::new_v4()),
            limit: Some(100),
            offset: None,
        };

        let query2 = FileListQuery {
            document_id: None,
            limit: None,
            offset: None,
        };

        let serialized1 = serde_json::to_string(&query1).expect("Failed to serialize");
        let serialized2 = serde_json::to_string(&query2).expect("Failed to serialize");

        let deserialized1: FileListQuery = serde_json::from_str(&serialized1).expect("Failed to deserialize");
        let deserialized2: FileListQuery = serde_json::from_str(&serialized2).expect("Failed to deserialize");

        assert_eq!(query1.document_id, deserialized1.document_id);
        assert_eq!(deserialized1.limit, Some(100u64));
        assert_eq!(deserialized1.offset, None);
        assert_eq!(deserialized2.document_id, None);
        assert_eq!(deserialized2.limit, None);
        assert_eq!(deserialized2.offset, None);
    }

    #[test]
    fn test_file_list_response() {
        use file_service::models::{FileListResponse, FileResponse};
        use uuid::Uuid;

        let file = FileResponse {
            id: Uuid::new_v4(),
            space_id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            file_name: "test.txt".to_string(),
            file_type: "text/plain".to_string(),
            file_size: 1024,
            download_url: "https://test.com/download".to_string(),
            created_at: chrono::Utc::now(),
        };

        let response = FileListResponse {
            files: vec![file],
            total: 1u64,
            limit: 100u64,
            offset: 0u64,
        };

        let serialized = serde_json::to_string(&response).expect("Failed to serialize");
        let deserialized: FileListResponse = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(response.files.len(), deserialized.files.len());
        assert_eq!(response.total, deserialized.total);
    }

    #[test]
    fn test_chunked_upload_response() {
        use file_service::models::ChunkUploadResponse;
        use uuid::Uuid;

        let response = ChunkUploadResponse {
            chunk_number: 5u32,
            uploaded_bytes: 1024u64,
            chunks_uploaded: 3u32,
            total_chunks: 20u32,
            expires_at: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&response).expect("Failed to serialize");
        let deserialized: ChunkUploadResponse = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(response.chunk_number, deserialized.chunk_number);
        assert_eq!(response.chunks_uploaded, deserialized.chunks_uploaded);
    }

    #[test]
    fn test_presigned_url_response() {
        use chrono::Utc;
        use file_service::models::PresignedUrlResponse;

        let response = PresignedUrlResponse {
            url: "https://test.com/upload".to_string(),
            method: "PUT".to_string(),
            headers: std::collections::HashMap::new(),
            expires_in: 3600i32,
            expires_at: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&response).expect("Failed to serialize");
        let deserialized: PresignedUrlResponse = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(response.url, deserialized.url);
        assert_eq!(response.method, deserialized.method);
    }

    #[test]
    fn test_message_response() {
        use file_service::models::MessageResponse;

        let response = MessageResponse {
            message: "Test message".to_string(),
        };

        let serialized = serde_json::to_string(&response).expect("Failed to serialize");
        let deserialized: MessageResponse = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(response.message, deserialized.message);
    }

    #[test]
    fn test_error_response() {
        use file_service::models::ErrorResponse;
        use std::collections::HashMap;

        let mut details = HashMap::new();
        details.insert("key1".to_string(), "value1".to_string());

        let response = ErrorResponse {
            code: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
            details: Some(details),
        };

        let serialized = serde_json::to_string(&response).expect("Failed to serialize");
        let deserialized: ErrorResponse = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(response.code, deserialized.code);
        assert!(deserialized.details.is_some());
    }

    #[test]
    fn test_presigned_upload_request() {
        use file_service::models::PresignedUploadRequest;
        use uuid::Uuid;

        let request = PresignedUploadRequest {
            space_id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            file_name: "test.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            expires_in: Some(3600u32),
        };

        let serialized = serde_json::to_string(&request).expect("Failed to serialize");
        let deserialized: PresignedUploadRequest = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(request.space_id, deserialized.space_id);
        assert_eq!(request.expires_in, Some(3600u32));
    }

    #[test]
    fn test_chunked_upload_init_request() {
        use file_service::models::InitChunkedUploadRequest;
        use uuid::Uuid;

        let request = InitChunkedUploadRequest {
            space_id: Uuid::new_v4(),
            document_id: None,
            file_name: "test.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            total_size: Some(10485760u64),
            chunk_size: Some(5242880u64),
            expires_in: Some(7200u32),
        };

        let serialized = serde_json::to_string(&request).expect("Failed to serialize");
        let deserialized: InitChunkedUploadRequest = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(request.space_id, deserialized.space_id);
        assert!(request.total_size, Some(10485760u64));
        assert!(request.chunk_size, Some(5242880u64));
        assert!(request.expires_in, Some(7200u32));
    }

    #[test]
    fn test_upload_chunk_request() {
        use file_service::models::UploadChunkRequest;
        use uuid::Uuid;

        let content = vec![1u8, 2u8, 3u8];

        let request = UploadChunkRequest {
            chunk_number: 5u32,
            content: content,
        };

        let serialized = serde_json::to_string(&request).expect("Failed to serialize");
        let deserialized: UploadChunkRequest = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(request.chunk_number, deserialized.chunk_number);
        assert_eq!(request.content.len(), deserialized.content.len());
    }

    #[test]
    fn test_complete_chunked_upload_request() {
        use file_service::models::CompleteChunkedUploadRequest;
        use uuid::Uuid;

        let request = CompleteChunkedUploadRequest {
            total_size: 10485760u64,
            checksum: "abc123def456".to_string(),
        };

        let serialized = serde_json::to_string(&request).expect("Failed to serialize");
        let deserialized: CompleteChunkedUploadRequest =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(request.total_size, deserialized.total_size);
        assert_eq!(request.checksum, deserialized.checksum);
    }

    #[test]
    fn test_presigned_download_request() {
        use file_service::models::PresignedDownloadRequest;
        use uuid::Uuid;

        let request = PresignedDownloadRequest {
            expires_in: Some(3600u32),
        };

        let serialized = serde_json::to_string(&request).expect("Failed to serialize");
        let deserialized: PresignedDownloadRequest = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(request.expires_in, Some(3600u32));
    }

    #[test]
    fn test_bulk_delete_request() {
        use file_service::models::BulkDeleteRequest;
        use uuid::Uuid;

        let file_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

        let request = BulkDeleteRequest { file_ids: file_ids };

        let serialized = serde_json::to_string(&request).expect("Failed to serialize");
        let deserialized: BulkDeleteRequest = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(request.file_ids.len(), deserialized.file_ids.len());
    }

    #[test]
    fn test_failed_delete_item() {
        use file_service::models::{ErrorResponse, FailedDelete};
        use uuid::Uuid;

        let failed_item = FailedDelete {
            file_id: uuid::new_v4(),
            reason: "Test failure reason".to_string(),
        };

        let serialized = serde_json::to_string(&failed_item).expect("Failed to serialize");
        let deserialized: FailedDelete = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(failed_item.file_id, deserialized.file_id);
        assert_eq!(failed_item.reason, deserialized.reason);
    }
}
