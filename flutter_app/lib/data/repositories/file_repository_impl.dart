import 'dart:io';

import 'package:dio/dio.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/core/network/network_error.dart' as ne;
import 'package:miniwiki/domain/entities/file.dart';
import 'package:miniwiki/domain/repositories/file_repository.dart';
import 'package:riverpod/riverpod.dart';

/// Implementation of FileRepository
class FileRepositoryImpl implements FileRepository {
  final ApiClient apiClient;
  final String baseUrl;

  FileRepositoryImpl({
    required this.apiClient,
    required this.baseUrl,
  });

  @override
  Future<FileEntity> uploadFile({
    required String spaceId,
    required String documentId,
    required String filePath,
    required String fileName,
    required String contentType,
    int? fileSize,
    void Function(double)? onProgress,
  }) async {
    final formData = FormData.fromMap({
      'file': await MultipartFile.fromFile(
        filePath,
        filename: fileName,
        contentType: DioMediaType.parse(contentType),
      ),
      'space_id': spaceId,
      'document_id': documentId,
      'file_name': fileName,
    });

    final response = await apiClient.dio.post<Map<String, dynamic>>(
      '$baseUrl/api/v1/files/upload',
      data: formData,
      onSendProgress: (count, total) {
        if (onProgress != null) {
          onProgress(count / total);
        }
      },
    );

    if (response.statusCode != 201) {
      throw ne.NetworkError(
        'Upload failed: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    return FileEntity.fromJson(response.data as Map<String, dynamic>);
  }

  @override
  Future<String> getPresignedUploadUrl({
    required String spaceId,
    required String fileName,
    required String contentType,
    required int contentLength,
  }) async {
    final response = await apiClient.dio.post<Map<String, dynamic>>(
      '$baseUrl/api/v1/files/upload/presigned-url',
      data: {
        'space_id': spaceId,
        'file_name': fileName,
        'content_type': contentType,
        'content_length': contentLength,
      },
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Failed to get presigned URL: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    return (response.data as Map<String, dynamic>)['url'] as String;
  }

  @override
  Future<String> downloadFile({
    required String fileId,
    required String savePath,
    void Function(double)? onProgress,
  }) async {
    final response = await apiClient.dio.get<List<int>>(
      '$baseUrl/api/v1/files/$fileId/download',
      options: Options(responseType: ResponseType.bytes),
      onReceiveProgress: (count, total) {
        if (onProgress != null) {
          onProgress(count / total);
        }
      },
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Download failed: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    final file = File(savePath);
    await file.writeAsBytes(response.data as List<int>);
    return savePath;
  }

  @override
  Future<String> getPresignedDownloadUrl(String fileId) async {
    final response = await apiClient.dio.get<Map<String, dynamic>>(
      '$baseUrl/api/v1/files/$fileId/download/presigned-url',
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Failed to get download URL: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    return (response.data as Map<String, dynamic>)['url'] as String;
  }

  @override
  Future<FileEntity> getFile(String fileId) async {
    final response = await apiClient.dio.get<Map<String, dynamic>>(
      '$baseUrl/api/v1/files/$fileId',
    );

    if (response.statusCode == 404) {
      throw ne.NetworkError('File not found', 404);
    }

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Failed to get file: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    final fileData =
        (response.data as Map<String, dynamic>)['file'] as Map<String, dynamic>;
    return FileEntity.fromJson(fileData);
  }

  @override
  Future<List<FileEntity>> listFiles({
    required String spaceId,
    String? documentId,
    int limit = 50,
    int offset = 0,
  }) async {
    final queryParams = <String, dynamic>{
      'limit': limit,
      'offset': offset,
    };
    if (documentId != null) {
      queryParams['document_id'] = documentId;
    }

    final response = await apiClient.dio.get<Map<String, dynamic>>(
      '$baseUrl/api/v1/files/spaces/$spaceId/files',
      queryParameters: queryParams,
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Failed to list files: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    final data = response.data as Map<String, dynamic>;
    final files = (data['files'] as List)
        .map((e) => FileEntity.fromJson(e as Map<String, dynamic>))
        .toList();
    return files;
  }

  @override
  Future<void> deleteFile(String fileId) async {
    final response = await apiClient.dio.delete<dynamic>(
      '$baseUrl/api/v1/files/$fileId',
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Delete failed: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }
  }

  @override
  Future<FileEntity> restoreFile(String fileId) async {
    final response = await apiClient.dio.post<Map<String, dynamic>>(
      '$baseUrl/api/v1/files/$fileId/restore',
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Restore failed: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    return FileEntity.fromJson(response.data as Map<String, dynamic>);
  }

  @override
  Future<void> permanentDeleteFile(String fileId) async {
    final response = await apiClient.dio.delete<dynamic>(
      '$baseUrl/api/v1/files/$fileId/permanent-delete',
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Permanent delete failed: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }
  }

  @override
  Future<ChunkedUploadInit> initChunkedUpload({
    required String spaceId,
    required String fileName,
    required String contentType,
    required int totalSize,
    int chunkSize = 5 * 1024 * 1024,
  }) async {
    final response = await apiClient.dio.post<Map<String, dynamic>>(
      '$baseUrl/api/v1/files/upload/chunked/init',
      data: {
        'space_id': spaceId,
        'file_name': fileName,
        'content_type': contentType,
        'total_size': totalSize,
        'chunk_size': chunkSize,
      },
    );

    if (response.statusCode != 201) {
      throw ne.NetworkError(
        'Failed to init chunked upload: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    return ChunkedUploadInit.fromJson(response.data as Map<String, dynamic>);
  }

  @override
  Future<ChunkUploadResult> uploadChunk({
    required String uploadId,
    required int chunkNumber,
    required List<int> chunkData,
  }) async {
    final response = await apiClient.dio.put<Map<String, dynamic>>(
      '$baseUrl/api/v1/files/upload/chunked/$uploadId',
      data: chunkData,
      options: Options(
        headers: {
          'Content-Type': 'application/octet-stream',
        },
      ),
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Failed to upload chunk: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    return ChunkUploadResult.fromJson(response.data as Map<String, dynamic>);
  }

  @override
  Future<FileEntity> completeChunkedUpload({
    required String uploadId,
    required List<ChunkInfo> chunks,
    String? checksum,
  }) async {
    final response = await apiClient.dio.post<Map<String, dynamic>>(
      '$baseUrl/api/v1/files/upload/chunked/$uploadId',
      data: {
        'chunks': chunks.map((e) => e.toJson()).toList(),
        'checksum': checksum,
      },
    );

    if (response.statusCode != 201) {
      throw ne.NetworkError(
        'Failed to complete upload: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    return FileEntity.fromJson(response.data as Map<String, dynamic>);
  }

  @override
  Future<void> cancelChunkedUpload(String uploadId) async {
    final response = await apiClient.dio.delete<dynamic>(
      '$baseUrl/api/v1/files/upload/chunked/$uploadId',
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Failed to cancel upload: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }
  }

  @override
  Future<BulkDeleteResult> bulkDeleteFiles(List<String> fileIds) async {
    final response = await apiClient.dio.post<Map<String, dynamic>>(
      '$baseUrl/api/v1/files/bulk/delete',
      data: {'file_ids': fileIds},
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Bulk delete failed: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    return BulkDeleteResult.fromJson(response.data as Map<String, dynamic>);
  }
}

/// Provider for FileRepository
final fileRepositoryProvider = Provider<FileRepository>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  const baseUrl = ''; // Will be loaded from config
  return FileRepositoryImpl(apiClient: apiClient, baseUrl: baseUrl);
});
