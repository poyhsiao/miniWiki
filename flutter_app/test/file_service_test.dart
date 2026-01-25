import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/domain/entities/file.dart';
import 'package:miniwiki/domain/repositories/file_repository.dart';
import 'package:miniwiki/services/file_service.dart';

// Mock implementation of FileRepository for testing
class MockFileRepository implements FileRepository {
  final Map<String, FileEntity> _files = {};
  int _idCounter = 1;

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
    final id = 'file_$_idCounter';
    _idCounter++;

    final file = FileEntity(
      id: id,
      spaceId: spaceId,
      documentId: documentId,
      uploadedBy: 'user1',
      fileName: fileName,
      fileType: contentType,
      fileSize: fileSize ?? 1000,
      downloadUrl: 'https://example.com/download/$id',
      createdAt: DateTime.now(),
    );

    _files[id] = file;
    return file;
  }

  @override
  Future<String> getPresignedUploadUrl({
    required String spaceId,
    required String fileName,
    required String contentType,
    required int contentLength,
  }) async =>
      'https://example.com/presigned/upload/$fileName';

  @override
  Future<String> downloadFile({
    required String fileId,
    required String savePath,
    void Function(double)? onProgress,
  }) async =>
      savePath;

  @override
  Future<String> getPresignedDownloadUrl(String fileId) async =>
      'https://example.com/download/$fileId';

  @override
  Future<FileEntity> getFile(String fileId) async {
    if (!_files.containsKey(fileId)) {
      throw Exception('File not found: $fileId');
    }
    return _files[fileId]!;
  }

  @override
  Future<List<FileEntity>> listFiles({
    required String spaceId,
    String? documentId,
    int limit = 50,
    int offset = 0,
  }) async {
    return _files.values
        .where((f) => f.spaceId == spaceId)
        .where((f) => documentId == null || f.documentId == documentId)
        .skip(offset)
        .take(limit)
        .toList();
  }

  @override
  Future<void> deleteFile(String fileId) async {
    final file = _files[fileId];
    if (file != null) {
      _files[fileId] = file.copyWith(isDeleted: true, deletedAt: DateTime.now());
    }
  }

  @override
  Future<FileEntity> restoreFile(String fileId) async {
    final file = _files[fileId];
    if (file != null) {
      final restored = file.copyWith(isDeleted: false, deletedAt: null);
      _files[fileId] = restored;
      return restored;
    }
    throw Exception('File not found');
  }

  @override
  Future<void> permanentDeleteFile(String fileId) async {
    _files.remove(fileId);
  }

  @override
  Future<ChunkedUploadInit> initChunkedUpload({
    required String spaceId,
    required String fileName,
    required String contentType,
    required int totalSize,
    int chunkSize = 5 * 1024 * 1024,
  }) async =>
      ChunkedUploadInit(
        uploadId: 'upload_${fileName}_$totalSize',
        uploadUrl: 'https://example.com/chunked/$fileName',
        chunkSize: chunkSize,
        totalChunks: (totalSize / chunkSize).ceil(),
      );

  @override
  Future<ChunkUploadResult> uploadChunk({
    required String uploadId,
    required int chunkNumber,
    required List<int> chunkData,
  }) async =>
      ChunkUploadResult(
        chunkNumber: chunkNumber,
        uploadedBytes: chunkData.length,
        chunksUploaded: chunkNumber + 1,
        totalChunks: 10,
      );

  @override
  Future<FileEntity> completeChunkedUpload({
    required String uploadId,
    required List<ChunkInfo> chunks,
    String? checksum,
  }) async {
    final id = 'file_$_idCounter';
    _idCounter++;

    final file = FileEntity(
      id: id,
      spaceId: 'space1',
      uploadedBy: 'user1',
      fileName: 'chunked_file.bin',
      fileType: 'application/octet-stream',
      fileSize: chunks.length * 5 * 1024 * 1024,
      downloadUrl: 'https://example.com/download/$id',
      createdAt: DateTime.now(),
    );

    _files[id] = file;
    return file;
  }

  @override
  Future<void> cancelChunkedUpload(String uploadId) async {
    // Mock cancel - do nothing
  }

  @override
  Future<BulkDeleteResult> bulkDeleteFiles(List<String> fileIds) async {
    final deleted = <String>[];
    final failed = <FailedDelete>[];

    for (final id in fileIds) {
      if (_files.containsKey(id)) {
        _files.remove(id);
        deleted.add(id);
      } else {
        failed.add(FailedDelete(fileId: id, reason: 'Not found'));
      }
    }

    return BulkDeleteResult(deleted: deleted, failed: failed);
  }
}

FileService createFileService(MockFileRepository mockRepository) {
  final dio = Dio();
  final apiClient = ApiClient(dio);
  return FileService(
    apiClient: apiClient,
    fileRepository: mockRepository,
    baseUrl: 'https://example.com',
  );
}

void main() {
  group('FileService - FileTypeFilter', () {
    test('FileTypeFilter has correct values', () {
      expect(FileTypeFilter.all.apiValue, 'all');
      expect(FileTypeFilter.all.displayName, 'All Files');

      expect(FileTypeFilter.image.apiValue, 'image');
      expect(FileTypeFilter.image.displayName, 'Images');

      expect(FileTypeFilter.document.apiValue, 'document');
      expect(FileTypeFilter.document.displayName, 'Documents');

      expect(FileTypeFilter.video.apiValue, 'video');
      expect(FileTypeFilter.video.displayName, 'Videos');

      expect(FileTypeFilter.audio.apiValue, 'audio');
      expect(FileTypeFilter.audio.displayName, 'Audio');

      expect(FileTypeFilter.archive.apiValue, 'archive');
      expect(FileTypeFilter.archive.displayName, 'Archives');
    });
  });

  group('FileService - UploadConfig', () {
    test('UploadConfig default values', () {
      const config = UploadConfig();

      expect(config.maxFileSize, 50 * 1024 * 1024);
      expect(config.defaultChunkSize, 5 * 1024 * 1024);
      expect(config.allowedExtensions.length, greaterThan(10));
    });

    test('UploadConfig isValidSize returns correct result', () {
      const config = UploadConfig(maxFileSize: 1000);

      expect(config.isValidSize(500), true);
      expect(config.isValidSize(1000), true);
      expect(config.isValidSize(1500), false);
    });

    test('UploadConfig isAllowedExtension works correctly', () {
      const config = UploadConfig();

      expect(config.isAllowedExtension('test.jpg'), true);
      expect(config.isAllowedExtension('test.pdf'), true);
      expect(config.isAllowedExtension('test.exe'), false);
      expect(config.isAllowedExtension('test'), false);
    });
  });

  group('FileService - FileState', () {
    test('FileState can be created with default values', () {
      const state = FileState();

      expect(state.isLoading, false);
      expect(state.files, isEmpty);
      expect(state.selectedFile, isNull);
      expect(state.error, isNull);
      expect(state.uploadProgress, isEmpty);
      expect(state.downloadProgress, isNull);
    });

    test('FileState can be created with all fields', () {
      final file = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'test.txt',
        fileType: 'text/plain',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file1',
      );

      final state = FileState(
        isLoading: true,
        files: [file],
        selectedFile: file,
        error: 'Test error',
        uploadProgress: {'file1': FileUploadProgress.initial('test.txt', 1000)},
        downloadProgress: 0.5,
      );

      expect(state.isLoading, true);
      expect(state.files.length, 1);
      expect(state.selectedFile?.id, 'file1');
      expect(state.error, 'Test error');
      expect(state.uploadProgress.isNotEmpty, true);
      expect(state.downloadProgress, 0.5);
    });

    test('FileState copyWith updates specified fields', () {
      final state = FileState(
        isLoading: true,
        error: 'Error',
      );

      final updated = state.copyWith(isLoading: false);

      expect(updated.isLoading, false);
      expect(updated.error, 'Error');
      expect(state.isLoading, true); // Original unchanged
    });
  });

  group('FileService - formatFileSize', () {
    late FileService service;
    late MockFileRepository mockRepository;

    setUp(() {
      mockRepository = MockFileRepository();
      service = createFileService(mockRepository);
    });

    test('formatFileSize formats bytes', () {
      expect(service.formatFileSize(0), '0 B');
      expect(service.formatFileSize(512), '512 B');
      expect(service.formatFileSize(1023), '1023 B');
    });

    test('formatFileSize formats kilobytes', () {
      expect(service.formatFileSize(1024), '1.0 KB');
      expect(service.formatFileSize(5120), '5.0 KB');
      expect(service.formatFileSize(1024 * 1023), '1023.0 KB');
    });

    test('formatFileSize formats megabytes', () {
      expect(service.formatFileSize(1024 * 1024), '1.0 MB');
      expect(service.formatFileSize(5 * 1024 * 1024), '5.0 MB');
    });

    test('formatFileSize formats gigabytes', () {
      expect(service.formatFileSize(1024 * 1024 * 1024), '1.0 GB');
      expect(service.formatFileSize(2 * 1024 * 1024 * 1024), '2.0 GB');
    });
  });

  group('FileService - getContentType', () {
    late FileService service;
    late MockFileRepository mockRepository;

    setUp(() {
      mockRepository = MockFileRepository();
      service = createFileService(mockRepository);
    });

    test('getContentType returns correct MIME types for images', () {
      expect(service.getContentType('test.jpg'), 'image/jpeg');
      expect(service.getContentType('test.jpeg'), 'image/jpeg');
      expect(service.getContentType('test.png'), 'image/png');
      expect(service.getContentType('test.gif'), 'image/gif');
      expect(service.getContentType('test.webp'), 'image/webp');
    });

    test('getContentType returns correct MIME types for documents', () {
      expect(service.getContentType('test.pdf'), 'application/pdf');
      expect(service.getContentType('test.doc'), 'application/msword');
      expect(service.getContentType('test.txt'), 'text/plain');
      expect(service.getContentType('test.md'), 'text/markdown');
    });

    test('getContentType returns correct MIME types for videos', () {
      expect(service.getContentType('test.mp4'), 'video/mp4');
      expect(service.getContentType('test.mov'), 'video/quicktime');
      expect(service.getContentType('test.avi'), 'video/x-msvideo');
    });

    test('getContentType returns correct MIME types for audio', () {
      expect(service.getContentType('test.mp3'), 'audio/mpeg');
      expect(service.getContentType('test.wav'), 'audio/wav');
      expect(service.getContentType('test.ogg'), 'audio/ogg');
    });

    test('getContentType returns octet-stream for unknown types', () {
      expect(service.getContentType('test.xyz'), 'application/octet-stream');
    });
  });

  group('FileService - listFiles', () {
    late FileService service;
    late MockFileRepository mockRepository;

    setUp(() async {
      mockRepository = MockFileRepository();
      service = createFileService(mockRepository);

      await mockRepository.uploadFile(
        spaceId: 'space1',
        documentId: 'doc1',
        filePath: '/path/file1.txt',
        fileName: 'file1.txt',
        contentType: 'text/plain',
        fileSize: 1000,
      );

      await mockRepository.uploadFile(
        spaceId: 'space1',
        documentId: 'doc1',
        filePath: '/path/file2.txt',
        fileName: 'file2.txt',
        contentType: 'text/plain',
        fileSize: 2000,
      );

      await mockRepository.uploadFile(
        spaceId: 'space2',
        documentId: 'doc2',
        filePath: '/path/file3.txt',
        fileName: 'file3.txt',
        contentType: 'text/plain',
        fileSize: 3000,
      );
    });

    test('listFiles returns files for space', () async {
      final files = await service.listFiles(spaceId: 'space1');

      expect(files.length, 2);
    });

    test('listFiles filters by documentId', () async {
      final files = await service.listFiles(
        spaceId: 'space1',
        documentId: 'doc1',
      );

      expect(files.length, 2);
    });

    test('listFiles respects limit parameter', () async {
      final files = await service.listFiles(
        spaceId: 'space1',
        limit: 1,
      );

      expect(files.length, 1);
    });

    test('listFiles respects offset parameter', () async {
      final files = await service.listFiles(
        spaceId: 'space1',
        offset: 1,
      );

      expect(files.length, 1);
    });
  });

  group('FileService - getFile', () {
    late FileService service;
    late MockFileRepository mockRepository;

    setUp(() async {
      mockRepository = MockFileRepository();
      service = createFileService(mockRepository);

      // Create a test file
      await mockRepository.uploadFile(
        spaceId: 'space1',
        documentId: 'doc1',
        filePath: '/path/test.txt',
        fileName: 'test.txt',
        contentType: 'text/plain',
        fileSize: 1000,
      );
    });

    test('getFile returns file by ID', () async {
      final file = await service.getFile('file_1');

      expect(file.id, 'file_1');
      expect(file.fileName, 'test.txt');
    });

    test('getFile throws exception when file not found', () async {
      expect(
        () => service.getFile('nonexistent'),
        throwsA(isA<Exception>()),
      );
    });
  });

  group('FileService - delete operations', () {
    late FileService service;
    late MockFileRepository mockRepository;

    setUp(() async {
      mockRepository = MockFileRepository();
      service = createFileService(mockRepository);

      await mockRepository.uploadFile(
        spaceId: 'space1',
        documentId: 'doc1',
        filePath: '/path/file1.txt',
        fileName: 'file1.txt',
        contentType: 'text/plain',
        fileSize: 1000,
      );
    });

    test('deleteFile soft deletes file', () async {
      await service.deleteFile('file_1');

      final file = await service.getFile('file_1');
      expect(file.isDeleted, true);
    });

    test('permanentDeleteFile removes file completely', () async {
      await mockRepository.uploadFile(
        spaceId: 'space1',
        documentId: 'doc1',
        filePath: '/path/file2.txt',
        fileName: 'file2.txt',
        contentType: 'text/plain',
        fileSize: 1000,
      );

      await service.permanentDeleteFile('file_2');

      final files = await service.listFiles(spaceId: 'space1');
      expect(files.any((f) => f.id == 'file_2'), false);
    });

    test('bulkDeleteFiles deletes multiple files', () async {
      final result = await service.bulkDeleteFiles(['file_1']);

      expect(result.deleted.length, 1);
      expect(result.failed.length, 0);
    });
  });

  group('FileService - presigned URLs', () {
    late FileService service;
    late MockFileRepository mockRepository;

    setUp(() {
      mockRepository = MockFileRepository();
      service = createFileService(mockRepository);
    });

    test('getPresignedUploadUrl returns valid URL', () async {
      final url = await service.getPresignedUploadUrl(
        spaceId: 'space1',
        fileName: 'test.txt',
        contentType: 'text/plain',
        contentLength: 1000,
      );

      expect(url, contains('https://example.com/presigned/upload/'));
      expect(url, contains('test.txt'));
    });

    test('getPresignedDownloadUrl returns valid URL', () async {
      final url = await service.getPresignedDownloadUrl('file123');

      expect(url, 'https://example.com/download/file123');
    });
  });

  group('FileEntity - Entity Operations', () {
    test('FileEntity can be created with required fields', () {
      final file = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'test.txt',
        fileType: 'text/plain',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file1',
      );

      expect(file.id, 'file1');
      expect(file.fileName, 'test.txt');
      expect(file.isDeleted, false);
      expect(file.documentId, isNull);
    });

    test('FileEntity copyWith updates specified fields', () {
      final file = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'test.txt',
        fileType: 'text/plain',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file1',
      );

      final updated = file.copyWith(fileName: 'updated.txt', isDeleted: true);

      expect(updated.fileName, 'updated.txt');
      expect(updated.isDeleted, true);
      expect(updated.id, 'file1'); // Original unchanged
    });

    test('FileEntity formattedSize works correctly', () {
      final smallFile = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'small.txt',
        fileType: 'text/plain',
        fileSize: 512,
        downloadUrl: 'https://example.com/file1',
      );

      final mediumFile = FileEntity(
        id: 'file2',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'medium.pdf',
        fileType: 'application/pdf',
        fileSize: 1024 * 1024,
        downloadUrl: 'https://example.com/file2',
      );

      expect(smallFile.formattedSize, '512 B');
      expect(mediumFile.formattedSize, '1.0 MB');
    });

    test('FileEntity extension works correctly', () {
      final file = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'document.pdf',
        fileType: 'application/pdf',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file1',
      );

      expect(file.extension, 'pdf');
    });

    test('FileEntity type checks work correctly', () {
      final image = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'image.jpg',
        fileType: 'image/jpeg',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file1',
      );

      final pdf = FileEntity(
        id: 'file2',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'document.pdf',
        fileType: 'application/pdf',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file2',
      );

      expect(image.isImage, true);
      expect(image.isPdf, false);
      expect(pdf.isPdf, true);
      expect(pdf.isImage, false);
    });

    test('FileEntity icon returns correct emoji', () {
      final image = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'image.jpg',
        fileType: 'image/jpeg',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file1',
      );

      final pdf = FileEntity(
        id: 'file2',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'document.pdf',
        fileType: 'application/pdf',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file2',
      );

      expect(image.icon, '🖼️');
      expect(pdf.icon, '📄');
    });
  });

  group('FileUploadProgress - Progress Tracking', () {
    test('FileUploadProgress initial factory creates initial state', () {
      final progress = FileUploadProgress.initial('test.txt', 5000);

      expect(progress.fileId, '');
      expect(progress.fileName, 'test.txt');
      expect(progress.bytesUploaded, 0);
      expect(progress.totalBytes, 5000);
      expect(progress.progress, 0);
      expect(progress.status, FileUploadStatus.pending);
    });

    test('FileUploadProgress copyWith updates fields', () {
      final progress = FileUploadProgress.initial('test.txt', 5000);

      final updated = progress.copyWith(
        fileId: 'file123',
        bytesUploaded: 2500,
        progress: 0.5,
        status: FileUploadStatus.uploading,
      );

      expect(updated.fileId, 'file123');
      expect(updated.bytesUploaded, 2500);
      expect(updated.progress, 0.5);
      expect(updated.status, FileUploadStatus.uploading);
      expect(progress.fileId, ''); // Original unchanged
    });
  });

  group('FileUploadStatus - Status Enum', () {
    test('FileUploadStatus has all expected values', () {
      expect(FileUploadStatus.pending, isNotNull);
      expect(FileUploadStatus.uploading, isNotNull);
      expect(FileUploadStatus.completed, isNotNull);
      expect(FileUploadStatus.failed, isNotNull);
      expect(FileUploadStatus.cancelled, isNotNull);
    });
  });

  group('ChunkedUploadInit - Upload Initialization', () {
    test('ChunkedUploadInit can be created', () {
      const init = ChunkedUploadInit(
        uploadId: 'upload123',
        uploadUrl: 'https://example.com/upload',
        chunkSize: 5 * 1024 * 1024,
        totalChunks: 10,
      );

      expect(init.uploadId, 'upload123');
      expect(init.totalChunks, 10);
    });

    test('ChunkedUploadInit fromJson parses correctly', () {
      final json = {
        'upload_id': 'upload123',
        'upload_url': 'https://example.com/upload',
        'chunk_size': 5242880,
        'total_chunks': 10,
      };

      final init = ChunkedUploadInit.fromJson(json);

      expect(init.uploadId, 'upload123');
      expect(init.uploadUrl, 'https://example.com/upload');
      expect(init.chunkSize, 5242880);
      expect(init.totalChunks, 10);
    });
  });

  group('ChunkUploadResult - Chunk Upload', () {
    test('ChunkUploadResult can be created', () {
      const result = ChunkUploadResult(
        chunkNumber: 0,
        uploadedBytes: 5242880,
        chunksUploaded: 1,
        totalChunks: 10,
      );

      expect(result.chunkNumber, 0);
      expect(result.chunksUploaded, 1);
    });

    test('ChunkUploadResult fromJson parses correctly', () {
      final json = {
        'chunk_number': 0,
        'uploaded_bytes': 5242880,
        'chunks_uploaded': 1,
        'total_chunks': 10,
      };

      final result = ChunkUploadResult.fromJson(json);

      expect(result.chunkNumber, 0);
      expect(result.uploadedBytes, 5242880);
      expect(result.chunksUploaded, 1);
      expect(result.totalChunks, 10);
    });
  });

  group('ChunkInfo - Chunk Completion', () {
    test('ChunkInfo can be created', () {
      const info = ChunkInfo(
        chunkNumber: 0,
        etag: 'etag123',
      );

      expect(info.chunkNumber, 0);
      expect(info.etag, 'etag123');
    });

    test('ChunkInfo toJson serializes correctly', () {
      const info = ChunkInfo(
        chunkNumber: 0,
        etag: 'etag123',
      );

      final json = info.toJson();

      expect(json['chunk_number'], 0);
      expect(json['etag'], 'etag123');
    });
  });

  group('BulkDeleteResult - Bulk Operations', () {
    test('BulkDeleteResult can be created', () {
      const result = BulkDeleteResult(
        deleted: ['file1', 'file2'],
        failed: [],
      );

      expect(result.deleted.length, 2);
      expect(result.failed.length, 0);
    });

    test('BulkDeleteResult fromJson parses correctly', () {
      final json = {
        'deleted': ['file1', 'file2'],
        'failed': [
          {'file_id': 'file3', 'reason': 'Not found'}
        ],
      };

      final result = BulkDeleteResult.fromJson(json);

      expect(result.deleted.length, 2);
      expect(result.failed.length, 1);
      expect(result.failed[0].fileId, 'file3');
      expect(result.failed[0].reason, 'Not found');
    });
  });

  group('FailedDelete - Error Handling', () {
    test('FailedDelete can be created', () {
      const failed = FailedDelete(
        fileId: 'file123',
        reason: 'Not found',
      );

      expect(failed.fileId, 'file123');
      expect(failed.reason, 'Not found');
    });

    test('FailedDelete fromJson parses correctly', () {
      final json = {
        'file_id': 'file123',
        'reason': 'Not found',
      };

      final failed = FailedDelete.fromJson(json);

      expect(failed.fileId, 'file123');
      expect(failed.reason, 'Not found');
    });
  });

  group('FileService - Integration Scenarios', () {
    late FileService service;
    late MockFileRepository mockRepository;

    setUp(() async {
      mockRepository = MockFileRepository();
      service = createFileService(mockRepository);
    });

    test('Full file lifecycle', () async {
      // Upload
      final uploaded = await mockRepository.uploadFile(
        spaceId: 'space1',
        documentId: 'doc1',
        filePath: '/path/file.txt',
        fileName: 'lifecycle.txt',
        contentType: 'text/plain',
        fileSize: 1000,
      );

      expect(uploaded.id, isNotNull);

      // Get file
      final retrieved = await service.getFile(uploaded.id);
      expect(retrieved.id, uploaded.id);

      // List files
      final files = await service.listFiles(spaceId: 'space1');
      expect(files.any((f) => f.id == uploaded.id), true);

      // Delete file
      await service.deleteFile(uploaded.id);

      // Verify soft delete
      final deleted = await service.getFile(uploaded.id);
      expect(deleted.isDeleted, true);

      // Restore file
      final restored = await service.restoreFile(uploaded.id);
      expect(restored.isDeleted, false);
    });

    test('Multiple spaces have separate files', () async {
      await mockRepository.uploadFile(
        spaceId: 'space1',
        documentId: 'doc1',
        filePath: '/path/file1.txt',
        fileName: 'file1.txt',
        contentType: 'text/plain',
        fileSize: 1000,
      );

      await mockRepository.uploadFile(
        spaceId: 'space2',
        documentId: 'doc1',
        filePath: '/path/file2.txt',
        fileName: 'file2.txt',
        contentType: 'text/plain',
        fileSize: 1000,
      );

      final space1Files = await service.listFiles(spaceId: 'space1');
      final space2Files = await service.listFiles(spaceId: 'space2');

      expect(space1Files.length, 1);
      expect(space2Files.length, 1);
      expect(space1Files[0].fileName, 'file1.txt');
      expect(space2Files[0].fileName, 'file2.txt');
    });
  });

  group('FileUtils - Extension Methods', () {
    test('FileUtils extension provides formattedSize', () {
      final file = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'test.txt',
        fileType: 'text/plain',
        fileSize: 1024 * 1024,
        downloadUrl: 'https://example.com/file1',
      );

      expect(file.formattedSize, '1.0 MB');
    });

    test('FileUtils extension provides extension', () {
      final file = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'document.pdf',
        fileType: 'application/pdf',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file1',
      );

      expect(file.extension, 'pdf');
    });

    test('FileUtils extension provides type checks', () {
      final image = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'photo.jpg',
        fileType: 'image/jpeg',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file1',
      );

      final video = FileEntity(
        id: 'file2',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'movie.mp4',
        fileType: 'video/mp4',
        fileSize: 5000000,
        downloadUrl: 'https://example.com/file2',
      );

      expect(image.isImage, true);
      expect(image.isVideo, false);
      expect(video.isVideo, true);
      expect(video.isImage, false);
    });

    test('FileUtils extension provides icon', () {
      final image = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'photo.jpg',
        fileType: 'image/jpeg',
        fileSize: 1000,
        downloadUrl: 'https://example.com/file1',
      );

      final video = FileEntity(
        id: 'file2',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'movie.mp4',
        fileType: 'video/mp4',
        fileSize: 5000000,
        downloadUrl: 'https://example.com/file2',
      );

      expect(image.icon, '🖼️');
      expect(video.icon, '🎬');
    });
  });
}
