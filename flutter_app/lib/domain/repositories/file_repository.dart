import 'package:miniwiki/domain/entities/file.dart';

/// Repository interface for file operations
abstract class FileRepository {
  /// Upload a file
  Future<FileEntity> uploadFile({
    required String spaceId,
    required String documentId,
    required String filePath,
    required String fileName,
    required String contentType,
    int? fileSize,
    void Function(double)? onProgress,
  });

  /// Get presigned upload URL
  Future<String> getPresignedUploadUrl({
    required String spaceId,
    required String fileName,
    required String contentType,
    required int contentLength,
  });

  /// Download a file
  Future<String> downloadFile({
    required String fileId,
    required String savePath,
    void Function(double)? onProgress,
  });

  /// Get presigned download URL
  Future<String> getPresignedDownloadUrl(String fileId);

  /// Get file metadata
  Future<FileEntity> getFile(String fileId);

  /// List files in a space
  Future<List<FileEntity>> listFiles({
    required String spaceId,
    String? documentId,
    int limit = 50,
    int offset = 0,
  });

  /// Delete a file (soft delete)
  Future<void> deleteFile(String fileId);

  /// Restore a deleted file
  Future<FileEntity> restoreFile(String fileId);

  /// Permanently delete a file
  Future<void> permanentDeleteFile(String fileId);

  /// Initialize chunked upload
  Future<ChunkedUploadInit> initChunkedUpload({
    required String spaceId,
    required String fileName,
    required String contentType,
    required int totalSize,
    int chunkSize = 5 * 1024 * 1024, // 5MB default
  });

  /// Upload a chunk
  Future<ChunkUploadResult> uploadChunk({
    required String uploadId,
    required int chunkNumber,
    required List<int> chunkData,
  });

  /// Complete chunked upload
  Future<FileEntity> completeChunkedUpload({
    required String uploadId,
    required List<ChunkInfo> chunks,
    String? checksum,
  });

  /// Cancel chunked upload
  Future<void> cancelChunkedUpload(String uploadId);

  /// Bulk delete files
  Future<BulkDeleteResult> bulkDeleteFiles(List<String> fileIds);
}

/// Chunked upload initialization response
class ChunkedUploadInit {
  final String uploadId;
  final String uploadUrl;
  final int chunkSize;
  final int totalChunks;

  const ChunkedUploadInit({
    required this.uploadId,
    required this.uploadUrl,
    required this.chunkSize,
    required this.totalChunks,
  });

  factory ChunkedUploadInit.fromJson(Map<String, dynamic> json) => ChunkedUploadInit(
      uploadId: json['upload_id'] as String,
      uploadUrl: json['upload_url'] as String,
      chunkSize: json['chunk_size'] as int,
      totalChunks: json['total_chunks'] as int,
    );
}

/// Chunk upload result
class ChunkUploadResult {
  final int chunkNumber;
  final int uploadedBytes;
  final int chunksUploaded;
  final int totalChunks;

  const ChunkUploadResult({
    required this.chunkNumber,
    required this.uploadedBytes,
    required this.chunksUploaded,
    required this.totalChunks,
  });

  factory ChunkUploadResult.fromJson(Map<String, dynamic> json) => ChunkUploadResult(
      chunkNumber: json['chunk_number'] as int,
      uploadedBytes: json['uploaded_bytes'] as int,
      chunksUploaded: json['chunks_uploaded'] as int,
      totalChunks: json['total_chunks'] as int,
    );
}

/// Chunk information for completion
class ChunkInfo {
  final int chunkNumber;
  final String etag;

  const ChunkInfo({
    required this.chunkNumber,
    required this.etag,
  });

  Map<String, dynamic> toJson() => {
      'chunk_number': chunkNumber,
      'etag': etag,
    };
}

/// Bulk delete result
class BulkDeleteResult {
  final List<String> deleted;
  final List<FailedDelete> failed;

  const BulkDeleteResult({
    required this.deleted,
    required this.failed,
  });

  factory BulkDeleteResult.fromJson(Map<String, dynamic> json) => BulkDeleteResult(
      deleted: List<String>.from(json['deleted'] as List),
      failed: (json['failed'] as List)
          .map((e) => FailedDelete.fromJson(e as Map<String, dynamic>))
          .toList(),
    );
}

/// Failed delete item
class FailedDelete {
  final String fileId;
  final String reason;

  const FailedDelete({
    required this.fileId,
    required this.reason,
  });

  factory FailedDelete.fromJson(Map<String, dynamic> json) => FailedDelete(
      fileId: json['file_id'] as String,
      reason: json['reason'] as String,
    );
}
