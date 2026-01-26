import 'dart:async';
import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/core/network/network_error.dart' as ne;
import 'package:miniwiki/domain/entities/file.dart';
import 'package:miniwiki/domain/repositories/file_repository.dart';
import 'package:path_provider/path_provider.dart';

/// File type enumeration for filtering
enum FileTypeFilter {
  all('all', 'All Files'),
  image('image', 'Images'),
  document('document', 'Documents'),
  video('video', 'Videos'),
  audio('audio', 'Audio'),
  archive('archive', 'Archives');

  final String apiValue;
  final String displayName;

  const FileTypeFilter(this.apiValue, this.displayName);
}

/// File upload configuration
class UploadConfig {
  final int maxFileSize;
  final List<String> allowedExtensions;
  final int defaultChunkSize;

  const UploadConfig({
    this.maxFileSize = 50 * 1024 * 1024, // 50MB default
    this.allowedExtensions = const [
      'jpg', 'jpeg', 'png', 'gif', 'webp', // images
      'pdf', 'doc', 'docx', 'txt', 'md', 'rtf', // documents
      'mp4', 'mov', 'avi', 'mkv', // videos
      'mp3', 'wav', 'ogg', 'm4a', // audio
      'zip', 'rar', '7z', // archives
    ],
    this.defaultChunkSize = 5 * 1024 * 1024, // 5MB chunks
  });

  bool isValidSize(int bytes) => bytes <= maxFileSize;

  bool isAllowedExtension(String fileName) {
    final ext = fileName.split('.').last.toLowerCase();
    return allowedExtensions.contains(ext);
  }
}

/// File state for provider
class FileState {
  final bool isLoading;
  final List<FileEntity> files;
  final FileEntity? selectedFile;
  final String? error;
  final Map<String, FileUploadProgress> uploadProgress;
  final double? downloadProgress;

  const FileState({
    this.isLoading = false,
    this.files = const [],
    this.selectedFile,
    this.error,
    this.uploadProgress = const {},
    this.downloadProgress,
  });

  FileState copyWith({
    bool? isLoading,
    List<FileEntity>? files,
    FileEntity? selectedFile,
    String? error,
    Map<String, FileUploadProgress>? uploadProgress,
    double? downloadProgress,
  }) =>
      FileState(
        isLoading: isLoading ?? this.isLoading,
        files: files ?? this.files,
        selectedFile: selectedFile ?? this.selectedFile,
        error: error ?? this.error,
        uploadProgress: uploadProgress ?? this.uploadProgress,
        downloadProgress: downloadProgress ?? this.downloadProgress,
      );
}

/// Service for file operations
class FileService {
  final ApiClient apiClient;
  final FileRepository fileRepository;
  final String baseUrl;
  final UploadConfig config;

  FileService({
    required this.apiClient,
    required this.fileRepository,
    required this.baseUrl,
    this.config = const UploadConfig(),
  });

  /// Upload a single file
  Future<FileEntity> uploadFile({
    required String spaceId,
    required String documentId,
    required String filePath,
    required String fileName,
    required String contentType,
    int? fileSize,
    void Function(double)? onProgress,
  }) async {
    // Validate file
    final file = File(filePath);
    final actualSize = fileSize ?? await file.length();

    if (!config.isValidSize(actualSize)) {
      throw ne.NetworkError(
        'File size exceeds maximum allowed size of ${config.maxFileSize ~/ (1024 * 1024)}MB',
        413,
      );
    }

    if (!config.isAllowedExtension(fileName)) {
      throw ne.NetworkError(
        'File type not allowed. Allowed types: ${config.allowedExtensions.join(', ')}',
        415,
      );
    }

    return fileRepository.uploadFile(
      spaceId: spaceId,
      documentId: documentId,
      filePath: filePath,
      fileName: fileName,
      contentType: contentType,
      fileSize: actualSize,
      onProgress: onProgress,
    );
  }

  /// Upload file with chunked transfer for large files
  Future<FileEntity> uploadFileChunked({
    required String spaceId,
    required String documentId,
    required String filePath,
    required String fileName,
    required String contentType,
    int? fileSize,
    void Function(double)? onProgress,
  }) async {
    final file = File(filePath);
    final actualSize = fileSize ?? await file.length();

    // For small files, use regular upload
    if (actualSize <= config.defaultChunkSize) {
      return uploadFile(
        spaceId: spaceId,
        documentId: documentId,
        filePath: filePath,
        fileName: fileName,
        contentType: contentType,
        fileSize: actualSize,
        onProgress: onProgress,
      );
    }

    // Initialize chunked upload
    final init = await fileRepository.initChunkedUpload(
      spaceId: spaceId,
      fileName: fileName,
      contentType: contentType,
      totalSize: actualSize,
      chunkSize: config.defaultChunkSize,
    );

    final chunks = <ChunkInfo>[];
    var chunkNumber = 0;
    var totalRead = 0;

    // Use RandomAccessFile for memory-efficient chunked reading
    RandomAccessFile? raf;
    try {
      raf = await file.open();
      final chunkSize = config.defaultChunkSize;

      while (true) {
        final chunkData = await raf.read(chunkSize);
        if (chunkData.isEmpty) {
          break; // EOF reached
        }

        await fileRepository.uploadChunk(
          uploadId: init.uploadId,
          chunkNumber: chunkNumber,
          chunkData: chunkData,
        );

        // Use chunk number as etag since backend doesn't return it
        chunks.add(ChunkInfo(
          chunkNumber: chunkNumber,
          etag: 'chunk_$chunkNumber',
        ));

        chunkNumber++;
        totalRead += chunkData.length;

        // Report progress
        if (onProgress != null) {
          onProgress(totalRead / actualSize);
        }
      }

      // Complete the upload
      return await fileRepository.completeChunkedUpload(
        uploadId: init.uploadId,
        chunks: chunks,
      );
    } catch (e) {
      // Cancel on error
      await fileRepository.cancelChunkedUpload(init.uploadId);
      rethrow;
    } finally {
      // Ensure RandomAccessFile is closed
      await raf?.close();
    }
  }

  /// Download a file
  Future<String> downloadFile({
    required String fileId,
    String? savePath,
    void Function(double)? onProgress,
  }) async {
    // Get suggested save path if not provided
    final directory = await getDownloadsDirectory();
    final fileEntity = await fileRepository.getFile(fileId);
    final basePath =
        directory?.path ?? (await getApplicationDocumentsDirectory()).path;
    final suggestedPath = '$basePath/${fileEntity.fileName}';
    final finalPath = savePath ?? suggestedPath;

    return fileRepository.downloadFile(
      fileId: fileId,
      savePath: finalPath,
      onProgress: onProgress,
    );
  }

  /// Get presigned upload URL for direct-to-S3 upload
  Future<String> getPresignedUploadUrl({
    required String spaceId,
    required String fileName,
    required String contentType,
    required int contentLength,
  }) async =>
      fileRepository.getPresignedUploadUrl(
        spaceId: spaceId,
        fileName: fileName,
        contentType: contentType,
        contentLength: contentLength,
      );

  /// Get presigned download URL for private files
  Future<String> getPresignedDownloadUrl(String fileId) async =>
      fileRepository.getPresignedDownloadUrl(fileId);

  /// List files in a space
  Future<List<FileEntity>> listFiles({
    required String spaceId,
    String? documentId,
    int limit = 50,
    int offset = 0,
  }) async =>
      fileRepository.listFiles(
        spaceId: spaceId,
        documentId: documentId,
        limit: limit,
        offset: offset,
      );

  /// Get a single file by ID
  Future<FileEntity> getFile(String fileId) async =>
      fileRepository.getFile(fileId);

  /// Delete a file (soft delete)
  Future<void> deleteFile(String fileId) async =>
      fileRepository.deleteFile(fileId);

  /// Restore a deleted file
  Future<FileEntity> restoreFile(String fileId) async =>
      fileRepository.restoreFile(fileId);

  /// Permanently delete a file
  Future<void> permanentDeleteFile(String fileId) async =>
      fileRepository.permanentDeleteFile(fileId);

  /// Bulk delete files
  Future<BulkDeleteResult> bulkDeleteFiles(List<String> fileIds) async =>
      fileRepository.bulkDeleteFiles(fileIds);

  /// Pick files from device
  Future<List<PlatformFile>> pickFiles({
    FileTypeFilter type = FileTypeFilter.all,
    bool allowMultiple = true,
    int maxFileSize = 50 * 1024 * 1024,
  }) async {
    FileType fileType;
    switch (type) {
      case FileTypeFilter.image:
        fileType = FileType.image;
        break;
      case FileTypeFilter.video:
        fileType = FileType.video;
        break;
      case FileTypeFilter.audio:
        fileType = FileType.audio;
        break;
      case FileTypeFilter.document:
      case FileTypeFilter.archive:
      case FileTypeFilter.all:
        fileType = FileType.any;
        break;
    }

    final result = await FilePicker.platform.pickFiles(
      type: fileType,
      allowMultiple: allowMultiple,
      allowedExtensions:
          type == FileTypeFilter.all ? config.allowedExtensions : null,
    );

    if (result == null) return [];

    return result.files.where((f) => f.size <= maxFileSize).toList();
  }

  /// Get content type from file extension
  String getContentType(String fileName) {
    final ext = fileName.split('.').last.toLowerCase();
    switch (ext) {
      case 'jpg':
      case 'jpeg':
        return 'image/jpeg';
      case 'png':
        return 'image/png';
      case 'gif':
        return 'image/gif';
      case 'webp':
        return 'image/webp';
      case 'pdf':
        return 'application/pdf';
      case 'doc':
        return 'application/msword';
      case 'docx':
        return 'application/vnd.openxmlformats-officedocument.wordprocessingml.document';
      case 'txt':
        return 'text/plain';
      case 'md':
        return 'text/markdown';
      case 'rtf':
        return 'application/rtf';
      case 'mp4':
        return 'video/mp4';
      case 'mov':
        return 'video/quicktime';
      case 'avi':
        return 'video/x-msvideo';
      case 'mkv':
        return 'video/x-matroska';
      case 'mp3':
        return 'audio/mpeg';
      case 'wav':
        return 'audio/wav';
      case 'ogg':
        return 'audio/ogg';
      case 'm4a':
        return 'audio/mp4';
      case 'zip':
        return 'application/zip';
      case 'rar':
        return 'application/x-rar-compressed';
      case '7z':
        return 'application/x-7z-compressed';
      default:
        return 'application/octet-stream';
    }
  }

  /// Format file size for display
  String formatFileSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
  }
}

/// File utilities extension
extension FileUtils on FileEntity {
  /// Get human-readable file size
  String get formattedSize {
    if (fileSize < 1024) return '$fileSize B';
    if (fileSize < 1024 * 1024) {
      return '${(fileSize / 1024).toStringAsFixed(1)} KB';
    }
    if (fileSize < 1024 * 1024 * 1024) {
      return '${(fileSize / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(fileSize / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
  }

  /// Get file extension
  String get extension {
    final parts = fileName.split('.');
    return parts.length > 1 ? parts.last.toLowerCase() : '';
  }

  /// Check if this is an image file
  bool get isImage => fileType.startsWith('image/');

  /// Check if this is a PDF file
  bool get isPdf => fileType == 'application/pdf';

  /// Check if this is a video file
  bool get isVideo => fileType.startsWith('video/');

  /// Check if this is an audio file
  bool get isAudio => fileType.startsWith('audio/');

  /// Check if this is a document
  bool get isDocument =>
      fileType.startsWith('text/') ||
      fileType == 'application/pdf' ||
      fileType.contains('word') ||
      fileType.contains('excel') ||
      fileType.contains('document');

  /// Get icon based on file type
  String get icon {
    if (isImage) return '🖼️';
    if (isPdf) return '📄';
    if (isVideo) return '🎬';
    if (isAudio) return '🎵';
    if (fileType == 'application/zip') return '📦';
    if (fileType.contains('word') ||
        extension == 'doc' ||
        extension == 'docx') {
      return '📝';
    }
    if (fileType.contains('excel') ||
        extension == 'xls' ||
        extension == 'xlsx') {
      return '📊';
    }
    return '📎';
  }
}
