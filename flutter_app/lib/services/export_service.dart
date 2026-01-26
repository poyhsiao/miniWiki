import 'dart:async';
import 'dart:io';

import 'package:dio/dio.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/core/network/network_error.dart' as ne;
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Export format enumeration
enum ExportFormat {
  markdown('markdown', 'Markdown', 'text/markdown', '.md'),
  html('html', 'HTML', 'text/html', '.html'),
  pdf('pdf', 'PDF', 'application/pdf', '.pdf'),
  json('json', 'JSON', 'application/json', '.json');

  final String apiValue;
  final String displayName;
  final String mimeType;
  final String extension;

  const ExportFormat(
    this.apiValue,
    this.displayName,
    this.mimeType,
    this.extension,
  );

  static ExportFormat? fromString(String value) {
    switch (value.toLowerCase()) {
      case 'markdown':
      case 'md':
        return ExportFormat.markdown;
      case 'html':
      case 'htm':
        return ExportFormat.html;
      case 'pdf':
        return ExportFormat.pdf;
      case 'json':
        return ExportFormat.json;
      default:
        return null;
    }
  }

  static List<ExportFormat> get availableFormats => [
        ExportFormat.markdown,
        ExportFormat.html,
        ExportFormat.pdf,
        ExportFormat.json,
      ];
}

/// Export result data
class ExportResult {
  final String documentId;
  final ExportFormat format;
  final String fileName;
  final int fileSize;
  final String contentType;
  final DateTime exportedAt;
  final String? localFilePath;

  const ExportResult({
    required this.documentId,
    required this.format,
    required this.fileName,
    required this.fileSize,
    required this.contentType,
    required this.exportedAt,
    this.localFilePath,
  });

  factory ExportResult.fromJson(Map<String, dynamic> json) => ExportResult(
        documentId: json['document_id'] as String,
        format: ExportFormat.fromString(json['format'] as String) ??
            ExportFormat.json,
        fileName: json['file_name'] as String,
        fileSize: json['file_size'] as int,
        contentType: json['content_type'] as String,
        exportedAt: DateTime.parse(json['exported_at'] as String),
        localFilePath: json['local_file_path'] as String?,
      );

  Map<String, dynamic> toJson() => {
        'document_id': documentId,
        'format': format.apiValue,
        'file_name': fileName,
        'file_size': fileSize,
        'content_type': contentType,
        'exported_at': exportedAt.toIso8601String(),
        'local_file_path': localFilePath,
      };
}

/// Export state for provider
class ExportState {
  final bool isExporting;
  final ExportResult? lastExport;
  final String? error;
  final double? downloadProgress;
  final List<ExportResult> exportHistory;

  const ExportState({
    this.isExporting = false,
    this.lastExport,
    this.error,
    this.downloadProgress,
    this.exportHistory = const [],
  });

  ExportState copyWith({
    bool? isExporting,
    ExportResult? lastExport,
    String? error,
    double? downloadProgress,
    List<ExportResult>? exportHistory,
  }) =>
      ExportState(
        isExporting: isExporting ?? this.isExporting,
        lastExport: lastExport ?? this.lastExport,
        error: error ?? this.error,
        downloadProgress: downloadProgress ?? this.downloadProgress,
        exportHistory: exportHistory ?? this.exportHistory,
      );
}

/// Service for document export operations
class ExportService {
  final ApiClient apiClient;
  final String baseUrl;

  ExportService({
    required this.apiClient,
    required this.baseUrl,
  });

  /// Export a document in the specified format
  ///
  /// [documentId] The ID of the document to export
  /// [format] The export format
  /// [downloadToDevice] Whether to download the file to the device
  Future<ExportResult> exportDocument({
    required String documentId,
    required ExportFormat format,
    bool downloadToDevice = true,
    void Function(double)? onDownloadProgress,
  }) async {
    // Validate parameters
    if (documentId.isEmpty) {
      throw ArgumentError('Document ID cannot be empty');
    }

    // Call the export API
    final response = await apiClient.dio.get<dynamic>(
      '$baseUrl/api/v1/documents/$documentId/export',
      queryParameters: {'format': format.apiValue},
      options: Options(
        responseType: ResponseType.bytes,
        headers: {
          'Accept': format.mimeType,
        },
      ),
      onReceiveProgress: onDownloadProgress != null
          ? (count, total) {
              onDownloadProgress(count / total);
            }
          : null,
    );

    if (response.statusCode != 200) {
      throw ne.NetworkError(
        'Export failed: ${response.statusMessage}',
        response.statusCode ?? 0,
      );
    }

    // Get content-disposition header for filename
    final contentDisposition = response.headers.value('content-disposition');
    var fileName = 'document_$documentId${format.extension}';
    if (contentDisposition != null) {
      final match =
          RegExp(r'filename="?([^";\n]+)"?').firstMatch(contentDisposition);
      if (match != null) {
        fileName = match.group(1)!;
      }
    }

    // Download to device if requested
    String? localFilePath;
    if (downloadToDevice && response.data != null) {
      localFilePath = await _saveToFile(
        documentId,
        fileName,
        response.data as List<int>,
        format,
      );
    }

    // Parse the export result from headers or response
    final dataLength = response.data is List<int>
        ? (response.data as List<int>).length
        : response.data is List
            ? (response.data as List).length
            : 0;
    final result = ExportResult(
      documentId: documentId,
      format: format,
      fileName: fileName,
      fileSize: dataLength,
      contentType: format.mimeType,
      exportedAt: DateTime.now(),
      localFilePath: localFilePath,
    );

    return result;
  }

  /// Export and get download URL
  Future<String> getExportUrl({
    required String documentId,
    required ExportFormat format,
  }) async {
    if (documentId.isEmpty) {
      throw ArgumentError('Document ID cannot be empty');
    }

    // Return the direct download URL
    return '$baseUrl/api/v1/documents/$documentId/export?format=${format.apiValue}';
  }

  /// Export and open in external app
  Future<String> exportAndOpen({
    required String documentId,
    required ExportFormat format,
  }) async {
    final result = await exportDocument(
      documentId: documentId,
      format: format,
    );

    if (result.localFilePath == null) {
      throw ne.NetworkError('Failed to save file locally', 500);
    }

    return result.localFilePath!;
  }

  /// Get supported export formats for a document
  Future<List<ExportFormat>> getSupportedFormats(String documentId) async {
    // All formats are supported by default
    // In the future, this could check with the server
    return ExportFormat.availableFormats;
  }

  /// Save exported file to device storage
  Future<String> _saveToFile(
    String documentId,
    String fileName,
    List<int> bytes,
    ExportFormat format,
  ) async {
    // Get documents directory
    final directory = await getApplicationDocumentsDirectory();
    final exportDir = Directory('${directory.path}/exports');

    // Create export directory if it doesn't exist
    if (!exportDir.existsSync()) {
      await exportDir.create(recursive: true);
    }

    // Create file path
    final filePath = '${exportDir.path}/$fileName';

    // Write file
    final file = File(filePath);
    await file.writeAsBytes(bytes);

    // Save filename mapping for future lookups
    await _saveFilenameMapping(documentId, fileName, format);

    return filePath;
  }

  /// Save filename mapping to SharedPreferences
  Future<void> _saveFilenameMapping(
      String documentId, String fileName, ExportFormat format) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(
        'export_filename_${documentId}_${format.name}', fileName);
  }

  /// Get saved filename from mapping, fallback to default pattern
  Future<String> _getFilename(String documentId, ExportFormat format) async {
    final prefs = await SharedPreferences.getInstance();
    final savedFilename =
        prefs.getString('export_filename_${documentId}_${format.name}');
    return savedFilename ?? 'document_$documentId${format.extension}';
  }

  /// Get export history for a document
  Future<List<ExportResult>> getExportHistory(String documentId) async {
    // In a real implementation, this would fetch from local storage
    // For now, return empty list
    return [];
  }

  /// Clear export history
  Future<void> clearExportHistory() async {
    // Clear local export files
    final directory = await getApplicationDocumentsDirectory();
    final exportDir = Directory('${directory.path}/exports');

    if (exportDir.existsSync()) {
      exportDir.deleteSync(recursive: true);
    }
  }

  /// Check if export file exists locally
  Future<bool> exportFileExists(String documentId, ExportFormat format) async {
    final directory = await getApplicationDocumentsDirectory();
    final exportDir = Directory('${directory.path}/exports');

    if (!exportDir.existsSync()) {
      return false;
    }

    final fileName = await _getFilename(documentId, format);
    final filePath = '${exportDir.path}/$fileName';
    return File(filePath).exists();
  }

  /// Delete local export file
  Future<void> deleteLocalExport(String documentId, ExportFormat format) async {
    final directory = await getApplicationDocumentsDirectory();
    final exportDir = Directory('${directory.path}/exports');

    if (exportDir.existsSync()) {
      final fileName = await _getFilename(documentId, format);
      final filePath = '${exportDir.path}/$fileName';
      final file = File(filePath);
      if (file.existsSync()) {
        file.deleteSync();
        // Clear the filename mapping
        final prefs = await SharedPreferences.getInstance();
        await prefs.remove('export_filename_${documentId}_${format.name}');
      }
    }
  }

  /// Share exported file
  ///
  /// Returns the local file path to be shared
  Future<String?> shareExport({
    required String documentId,
    required ExportFormat format,
  }) async {
    // First ensure file exists
    final exists = await exportFileExists(documentId, format);
    if (!exists) {
      // Export if not exists
      await exportDocument(
        documentId: documentId,
        format: format,
      );
    }

    final directory = await getApplicationDocumentsDirectory();
    final fileName = await _getFilename(documentId, format);
    final filePath = '${directory.path}/exports/$fileName';
    final file = File(filePath);

    if (file.existsSync()) {
      return filePath;
    }

    return null;
  }
}

/// Export utilities extension
extension ExportUtils on ExportFormat {
  /// Get human-readable description of the format
  String get description {
    switch (this) {
      case ExportFormat.markdown:
        return 'Markdown format with frontmatter metadata. Best for importing to other editors.';
      case ExportFormat.html:
        return 'HTML format with embedded styles. Good for sharing and printing.';
      case ExportFormat.pdf:
        return 'PDF document. Best for sharing read-only documents.';
      case ExportFormat.json:
        return 'JSON format with Yjs document state. For backup and import.';
    }
  }

  /// Get icon for the format
  String get icon {
    switch (this) {
      case ExportFormat.markdown:
        return '📝';
      case ExportFormat.html:
        return '🌐';
      case ExportFormat.pdf:
        return '📄';
      case ExportFormat.json:
        return '{ }';
    }
  }

  /// Check if this format supports offline viewing
  bool get supportsOfflineViewing => this != ExportFormat.json;

  /// Check if this format is editable in external apps
  bool get isEditable =>
      this == ExportFormat.markdown || this == ExportFormat.html;
}
