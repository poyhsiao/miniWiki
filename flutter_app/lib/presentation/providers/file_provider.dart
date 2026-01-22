import 'package:file_picker/file_picker.dart';
import 'package:miniwiki/core/config/providers.dart';
import 'package:miniwiki/domain/entities/file.dart';
import 'package:miniwiki/services/file_service.dart';
import 'package:riverpod/riverpod.dart';

/// State for file list
class FileListState {
  final List<FileEntity> files;
  final int total;
  final bool isLoading;
  final String? error;
  final String spaceId;
  final String? documentId;
  final bool hasMore;

  const FileListState({
    required this.spaceId,
    this.files = const [],
    this.total = 0,
    this.isLoading = false,
    this.error,
    this.documentId,
    this.hasMore = false,
  });

  FileListState copyWith({
    List<FileEntity>? files,
    int? total,
    bool? isLoading,
    String? error,
    bool? clearError,
    String? spaceId,
    String? documentId,
    bool? hasMore,
  }) =>
      FileListState(
        spaceId: spaceId ?? this.spaceId,
        files: files ?? this.files,
        total: total ?? this.total,
        isLoading: isLoading ?? this.isLoading,
        error: clearError ?? false ? null : (error ?? this.error),
        documentId: documentId ?? this.documentId,
        hasMore: hasMore ?? this.hasMore,
      );
}

/// State for file upload
class FileUploadState {
  final Map<String, FileUploadProgress> uploads;
  final bool isUploading;

  const FileUploadState({
    this.uploads = const {},
    this.isUploading = false,
  });

  FileUploadState copyWith({
    Map<String, FileUploadProgress>? uploads,
    bool? isUploading,
  }) =>
      FileUploadState(
        uploads: uploads ?? this.uploads,
        isUploading: isUploading ?? this.isUploading,
      );

  double get overallProgress {
    if (uploads.isEmpty) return 0;
    final totalProgress =
        uploads.values.fold(0.0, (sum, upload) => sum + upload.progress);
    return totalProgress / uploads.length;
  }
}

/// Provider for file list notifier
class FileListNotifier extends StateNotifier<FileListState> {
  final FileService _service;
  final String spaceId;

  FileListNotifier(this._service, this.spaceId)
      : super(const FileListState(spaceId: ''));

  String? documentId;

  Future<void> init(String spaceId, {String? documentId}) async {
    state = state.copyWith(
        spaceId: spaceId, documentId: documentId, isLoading: true);
    await loadFiles();
  }

  Future<void> loadFiles({int limit = 50, bool append = false}) async {
    if (state.spaceId.isEmpty) return;

    state = state.copyWith(isLoading: true, clearError: true);

    try {
      final files = await _service.listFiles(
        spaceId: state.spaceId,
        documentId: state.documentId,
        limit: limit,
        offset: append ? state.files.length : 0,
      );

      final newFiles = append ? [...state.files, ...files] : files;
      final hasMore = files.length >= limit;

      state = state.copyWith(
        files: newFiles,
        total: append ? state.total + files.length : files.length,
        isLoading: false,
        hasMore: hasMore,
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.toString(),
      );
    }
  }

  Future<void> loadMore() async {
    if (!state.hasMore || state.isLoading) return;
    await loadFiles(append: true);
  }

  void addFile(FileEntity file) {
    state = state.copyWith(
      files: [file, ...state.files],
      total: state.total + 1,
    );
  }

  void updateFile(FileEntity updatedFile) {
    state = state.copyWith(
      files: state.files
          .map((f) => f.id == updatedFile.id ? updatedFile : f)
          .toList(),
    );
  }

  void removeFile(String fileId) {
    state = state.copyWith(
      files: state.files.where((f) => f.id != fileId).toList(),
      total: state.total - 1,
    );
  }

  void clearFiles() {
    state = const FileListState(spaceId: '');
  }
}

/// Provider for file upload notifier
class FileUploadNotifier extends StateNotifier<FileUploadState> {
  final FileService _service;
  final String spaceId;
  final String documentId;

  FileUploadNotifier(this._service, this.spaceId, this.documentId)
      : super(const FileUploadState());

  Future<void> uploadFile(PlatformFile platformFile) async {
    final fileId = platformFile.name;

    // Check if already uploading
    if (state.uploads.containsKey(fileId)) return;

    // Initialize upload progress
    final progress = FileUploadProgress.initial(
      platformFile.name,
      platformFile.size,
    );

    state = state.copyWith(
      uploads: {...state.uploads, fileId: progress},
      isUploading: true,
    );

    try {
      // Update to uploading status
      state = state.copyWith(
        uploads: {
          ...state.uploads,
          fileId: state.uploads[fileId]!
              .copyWith(status: FileUploadStatus.uploading),
        },
      );

      // Upload the file
      final entity = await _service.uploadFile(
        spaceId: spaceId,
        documentId: documentId,
        filePath: platformFile.path!,
        fileName: platformFile.name,
        contentType: _service.getContentType(platformFile.name),
        fileSize: platformFile.size,
        onProgress: (progress) {
          state = state.copyWith(
            uploads: {
              ...state.uploads,
              fileId: state.uploads[fileId]!.copyWith(
                progress: progress,
                bytesUploaded: (platformFile.size * progress).round(),
              ),
            },
          );
        },
      );

      // Update to completed status
      state = state.copyWith(
        uploads: {
          ...state.uploads,
          fileId: state.uploads[fileId]!.copyWith(
            status: FileUploadStatus.completed,
            progress: 1.0,
            bytesUploaded: platformFile.size,
            fileId: entity.id,
          ),
        },
      );
    } catch (e) {
      // Update to failed status
      state = state.copyWith(
        uploads: {
          ...state.uploads,
          fileId: state.uploads[fileId]!.copyWith(
            status: FileUploadStatus.failed,
            error: e.toString(),
          ),
        },
      );
    }
  }

  Future<void> uploadFiles(List<PlatformFile> files) async {
    for (final file in files) {
      if (file.path != null) {
        await uploadFile(file);
      }
    }
  }

  void removeUpload(String fileName) {
    final uploads = Map<String, FileUploadProgress>.from(state.uploads);
    uploads.remove(fileName);
    state = state.copyWith(
      uploads: uploads,
      isUploading: uploads.isNotEmpty,
    );
  }

  void clearCompleted() {
    final uploads = Map<String, FileUploadProgress>.from(state.uploads);
    uploads.removeWhere((_, p) => p.status == FileUploadStatus.completed);
    state = state.copyWith(
      uploads: uploads,
      isUploading: uploads.isNotEmpty,
    );
  }

  void clearAll() {
    state = const FileUploadState();
  }
}

/// Provider for file list notifier
final fileListNotifierProvider =
    StateNotifierProvider.family<FileListNotifier, FileListState, String>(
  (ref, spaceId) {
    final service = ref.watch(fileServiceProvider);
    return FileListNotifier(service, spaceId);
  },
);

/// Provider for file upload notifier (requires spaceId and documentId)
final fileUploadNotifierProvider =
    StateNotifierProvider.family<FileUploadNotifier, FileUploadState, String>(
  (ref, key) {
    // The key is composed of 'spaceId:documentId'
    final parts = key.split(':');
    final spaceId = parts[0];
    final documentId = parts.length > 1 ? parts[1] : '';
    final service = ref.watch(fileServiceProvider);
    return FileUploadNotifier(service, spaceId, documentId);
  },
);
