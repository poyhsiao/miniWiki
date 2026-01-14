import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/core/network/network_error.dart' as ne;
import 'package:miniwiki/services/export_service.dart';

/// Export state for the provider
class ExportUiState {
  final bool isExporting;
  final ExportResult? lastExport;
  final String? error;
  final double? downloadProgress;
  final List<ExportResult> exportHistory;
  final ExportFormat? selectedFormat;
  final bool showExportDialog;

  const ExportUiState({
    this.isExporting = false,
    this.lastExport,
    this.error,
    this.downloadProgress,
    this.exportHistory = const [],
    this.selectedFormat,
    this.showExportDialog = false,
  });

  ExportUiState copyWith({
    bool? isExporting,
    ExportResult? lastExport,
    String? error,
    double? downloadProgress,
    List<ExportResult>? exportHistory,
    ExportFormat? selectedFormat,
    bool? showExportDialog,
  }) {
    return ExportUiState(
      isExporting: isExporting ?? this.isExporting,
      lastExport: lastExport ?? this.lastExport,
      error: error ?? this.error,
      downloadProgress: downloadProgress ?? this.downloadProgress,
      exportHistory: exportHistory ?? this.exportHistory,
      selectedFormat: selectedFormat ?? this.selectedFormat,
      showExportDialog: showExportDialog ?? this.showExportDialog,
    );
  }
}

/// Provider for export operations
class ExportNotifier extends StateNotifier<ExportUiState> {
  final ExportService _exportService;

  ExportNotifier(this._exportService) : super(const ExportUiState());

  /// Export a document in the specified format
  Future<ExportResult> exportDocument({
    required String documentId,
    required ExportFormat format,
    bool downloadToDevice = true,
  }) async {
    state = state.copyWith(
      isExporting: true,
      error: null,
      downloadProgress: 0.0,
      selectedFormat: format,
    );

    try {
      final result = await _exportService.exportDocument(
        documentId: documentId,
        format: format,
        downloadToDevice: downloadToDevice,
        onDownloadProgress: (progress) {
          state = state.copyWith(downloadProgress: progress);
        },
      );

      state = state.copyWith(
        isExporting: false,
        lastExport: result,
        downloadProgress: 1.0,
        exportHistory: [result, ...state.exportHistory].take(10).toList(),
      );

      return result;
    } catch (e) {
      state = state.copyWith(
        isExporting: false,
        error: e is ne.NetworkError ? e.message : e.toString(),
        downloadProgress: null,
      );
      rethrow;
    }
  }

  /// Get supported export formats
  Future<List<ExportFormat>> getSupportedFormats(String documentId) async {
    return _exportService.getSupportedFormats(documentId);
  }

  /// Check if export file exists locally
  Future<bool> exportFileExists(String documentId, ExportFormat format) async {
    return _exportService.exportFileExists(documentId, format);
  }

  /// Share exported file
  Future<String?> shareExport({
    required String documentId,
    required ExportFormat format,
  }) async {
    return _exportService.shareExport(documentId: documentId, format: format);
  }

  /// Get export URL for sharing
  Future<String> getExportUrl({
    required String documentId,
    required ExportFormat format,
  }) async {
    return _exportService.getExportUrl(documentId: documentId, format: format);
  }

  /// Show export dialog
  void showDialog({ExportFormat? initialFormat}) {
    state = state.copyWith(
      showExportDialog: true,
      selectedFormat: initialFormat,
      error: null,
    );
  }

  /// Hide export dialog
  void hideDialog() {
    state = state.copyWith(showExportDialog: false, selectedFormat: null);
  }

  /// Select export format
  void selectFormat(ExportFormat format) {
    state = state.copyWith(selectedFormat: format);
  }

  /// Clear error
  void clearError() {
    state = state.copyWith(error: null);
  }

  /// Clear export history
  Future<void> clearExportHistory() async {
    await _exportService.clearExportHistory();
    state = state.copyWith(exportHistory: const [], lastExport: null);
  }
}

/// Export service provider
final exportServiceProvider = Provider<ExportService>((ref) {
  return ExportService(
    apiClient: ref.watch(apiClientProvider),
    baseUrl: '',
  );
});

/// Export notifier provider
final exportNotifierProvider =
    StateNotifierProvider<ExportNotifier, ExportUiState>((ref) {
  return ExportNotifier(ref.watch(exportServiceProvider));
});
