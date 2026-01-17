import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

part 'file.g.dart';

/// File entity for attachments
@JsonSerializable()
class FileEntity extends Equatable {
  final String id;
  final String spaceId;
  @JsonKey(fromJson: _fromNull, toJson: _toNull)
  final String? documentId;
  final String uploadedBy;
  final String fileName;
  final String fileType;
  final int fileSize;
  final String downloadUrl;
  @JsonKey(fromJson: _dateTimeFromJson, toJson: _dateTimeToJson)
  final DateTime? createdAt;
  final bool isDeleted;
  @JsonKey(fromJson: _dateTimeFromJson, toJson: _dateTimeToJson)
  final DateTime? deletedAt;

  const FileEntity({
    required this.id,
    required this.spaceId,
    required this.uploadedBy,
    required this.fileName,
    required this.fileType,
    required this.fileSize,
    required this.downloadUrl,
    this.documentId,
    this.createdAt,
    this.isDeleted = false,
    this.deletedAt,
  });

  factory FileEntity.fromJson(Map<String, dynamic> json) =>
      _$FileEntityFromJson(json);

  Map<String, dynamic> toJson() => _$FileEntityToJson(this);

  static String? _fromNull(Object? json) {
    if (json == null) return null;
    if (json is String) return json;
    throw FormatException(
        '_fromNull expected String or null, got ${json.runtimeType}');
  }

  static dynamic _toNull(String? value) => value;
  static DateTime? _dateTimeFromJson(Object? json) {
    if (json == null) return null;
    if (json is String) return DateTime.tryParse(json);
    if (json is num) return DateTime.fromMillisecondsSinceEpoch((json).toInt());
    return null;
  }

  static String? _dateTimeToJson(DateTime? value) => value?.toIso8601String();

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

  /// Get icon based on file type
  String get icon {
    if (isImage) return '🖼️';
    if (isPdf) return '📄';
    if (isVideo) return '🎬';
    if (isAudio) return '🎵';
    if (fileType == 'application/zip') return '📦';
    if (fileType.contains('word') || fileName.endsWith('.doc')) return '📝';
    if (fileType.contains('excel') || fileName.endsWith('.xls')) return '📊';
    return '📎';
  }

  FileEntity copyWith({
    String? id,
    String? spaceId,
    String? documentId,
    String? uploadedBy,
    String? fileName,
    String? fileType,
    int? fileSize,
    String? downloadUrl,
    DateTime? createdAt,
    bool? isDeleted,
    DateTime? deletedAt,
  }) =>
      FileEntity(
        id: id ?? this.id,
        spaceId: spaceId ?? this.spaceId,
        documentId: documentId ?? this.documentId,
        uploadedBy: uploadedBy ?? this.uploadedBy,
        fileName: fileName ?? this.fileName,
        fileType: fileType ?? this.fileType,
        fileSize: fileSize ?? this.fileSize,
        downloadUrl: downloadUrl ?? this.downloadUrl,
        createdAt: createdAt ?? this.createdAt,
        isDeleted: isDeleted ?? this.isDeleted,
        deletedAt: deletedAt ?? this.deletedAt,
      );

  @override
  List<Object?> get props => [
        id,
        spaceId,
        documentId,
        uploadedBy,
        fileName,
        fileType,
        fileSize,
        downloadUrl,
        createdAt,
        isDeleted,
        deletedAt,
      ];
}

/// File upload progress
class FileUploadProgress {
  final String fileId;
  final String fileName;
  final int bytesUploaded;
  final int totalBytes;
  final double progress;
  final FileUploadStatus status;
  final String? error;

  const FileUploadProgress({
    required this.fileId,
    required this.fileName,
    required this.bytesUploaded,
    required this.totalBytes,
    required this.progress,
    required this.status,
    this.error,
  });

  factory FileUploadProgress.initial(String fileName, int totalBytes) =>
      FileUploadProgress(
        fileId: '',
        fileName: fileName,
        bytesUploaded: 0,
        totalBytes: totalBytes,
        progress: 0,
        status: FileUploadStatus.pending,
      );

  FileUploadProgress copyWith({
    String? fileId,
    String? fileName,
    int? bytesUploaded,
    int? totalBytes,
    double? progress,
    FileUploadStatus? status,
    String? error,
  }) =>
      FileUploadProgress(
        fileId: fileId ?? this.fileId,
        fileName: fileName ?? this.fileName,
        bytesUploaded: bytesUploaded ?? this.bytesUploaded,
        totalBytes: totalBytes ?? this.totalBytes,
        progress: progress ?? this.progress,
        status: status ?? this.status,
        error: error ?? this.error,
      );
}

enum FileUploadStatus {
  pending,
  uploading,
  completed,
  failed,
  cancelled,
}
