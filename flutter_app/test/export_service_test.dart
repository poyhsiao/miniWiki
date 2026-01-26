import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/services/export_service.dart';

void main() {
  group('ExportFormat Enum Tests', () {
    test('ExportFormat has all required values', () {
      expect(ExportFormat.markdown.apiValue, 'markdown');
      expect(ExportFormat.markdown.displayName, 'Markdown');
      expect(ExportFormat.markdown.mimeType, 'text/markdown');
      expect(ExportFormat.markdown.extension, '.md');

      expect(ExportFormat.html.apiValue, 'html');
      expect(ExportFormat.pdf.apiValue, 'pdf');
      expect(ExportFormat.json.apiValue, 'json');
    });

    test('ExportFormat.fromString parses valid formats', () {
      expect(ExportFormat.fromString('markdown'), ExportFormat.markdown);
      expect(ExportFormat.fromString('md'), ExportFormat.markdown);
      expect(ExportFormat.fromString('html'), ExportFormat.html);
      expect(ExportFormat.fromString('pdf'), ExportFormat.pdf);
      expect(ExportFormat.fromString('json'), ExportFormat.json);
    });

    test('ExportFormat.fromString is case insensitive', () {
      expect(ExportFormat.fromString('MARKDOWN'), ExportFormat.markdown);
      expect(ExportFormat.fromString('Markdown'), ExportFormat.markdown);
      expect(ExportFormat.fromString('HTML'), ExportFormat.html);
    });

    test('ExportFormat.fromString returns null for invalid format', () {
      expect(ExportFormat.fromString('invalid'), isNull);
      expect(ExportFormat.fromString('docx'), isNull);
      expect(ExportFormat.fromString(''), isNull);
    });

    test('ExportFormat.availableFormats returns all formats', () {
      final formats = ExportFormat.availableFormats;
      expect(formats.length, 4);
      expect(formats, contains(ExportFormat.markdown));
      expect(formats, contains(ExportFormat.html));
      expect(formats, contains(ExportFormat.pdf));
      expect(formats, contains(ExportFormat.json));
    });
  });

  group('ExportResult Tests', () {
    test('ExportResult can be created with all fields', () {
      // Arrange & Act
      final now = DateTime(2024);
      final result = ExportResult(
        documentId: 'doc1',
        format: ExportFormat.markdown,
        fileName: 'document.md',
        fileSize: 1024,
        contentType: 'text/markdown',
        exportedAt: now,
        localFilePath: '/path/to/document.md',
      );

      // Assert
      expect(result.documentId, 'doc1');
      expect(result.format, ExportFormat.markdown);
      expect(result.fileName, 'document.md');
      expect(result.fileSize, 1024);
      expect(result.contentType, 'text/markdown');
      expect(result.exportedAt, now);
      expect(result.localFilePath, '/path/to/document.md');
    });

    test('ExportResult can be created without localFilePath', () {
      // Arrange & Act
      final result = ExportResult(
        documentId: 'doc1',
        format: ExportFormat.pdf,
        fileName: 'document.pdf',
        fileSize: 2048,
        contentType: 'application/pdf',
        exportedAt: DateTime(2024),
      );

      // Assert
      expect(result.localFilePath, isNull);
    });

    test('ExportResult toJson creates correct JSON', () {
      // Arrange
      final result = ExportResult(
        documentId: 'doc1',
        format: ExportFormat.markdown,
        fileName: 'document.md',
        fileSize: 1024,
        contentType: 'text/markdown',
        exportedAt: DateTime(2024),
        localFilePath: '/path/to/file.md',
      );

      // Act
      final json = result.toJson();

      // Assert
      expect(json['document_id'], 'doc1');
      expect(json['format'], 'markdown');
      expect(json['file_name'], 'document.md');
      expect(json['file_size'], 1024);
      expect(json['content_type'], 'text/markdown');
      expect(json['local_file_path'], '/path/to/file.md');
    });

    test('ExportResult fromJson creates instance correctly', () {
      // Arrange
      final json = {
        'document_id': 'doc1',
        'format': 'markdown',
        'file_name': 'document.md',
        'file_size': 1024,
        'content_type': 'text/markdown',
        'exported_at': '2024-01-01T00:00:00.000Z',
        'local_file_path': '/path/to/file.md',
      };

      // Act
      final result = ExportResult.fromJson(json);

      // Assert
      expect(result.documentId, 'doc1');
      expect(result.format, ExportFormat.markdown);
      expect(result.fileName, 'document.md');
      expect(result.fileSize, 1024);
      expect(result.localFilePath, '/path/to/file.md');
    });

    test('ExportResult fromJson handles invalid format with default', () {
      // Arrange
      final json = {
        'document_id': 'doc1',
        'format': 'invalid',
        'file_name': 'document.xyz',
        'file_size': 1024,
        'content_type': 'application/octet-stream',
        'exported_at': '2024-01-01T00:00:00.000Z',
      };

      // Act
      final result = ExportResult.fromJson(json);

      // Assert
      expect(result.format, ExportFormat.json); // Default fallback
    });
  });

  group('ExportState Tests', () {
    test('ExportState can be created with default values', () {
      // Arrange & Act
      const state = ExportState();

      // Assert
      expect(state.isExporting, false);
      expect(state.lastExport, isNull);
      expect(state.error, isNull);
      expect(state.downloadProgress, isNull);
      expect(state.exportHistory, isEmpty);
    });

    test('ExportState can be created with custom values', () {
      // Arrange & Act
      final now = DateTime(2024);
      final lastExport = ExportResult(
        documentId: 'doc1',
        format: ExportFormat.pdf,
        fileName: 'doc.pdf',
        fileSize: 1024,
        contentType: 'application/pdf',
        exportedAt: now,
      );

      final state = ExportState(
        isExporting: true,
        lastExport: lastExport,
        error: 'Export failed',
        downloadProgress: 0.5,
        exportHistory: [lastExport],
      );

      // Assert
      expect(state.isExporting, true);
      expect(state.lastExport, lastExport);
      expect(state.error, 'Export failed');
      expect(state.downloadProgress, 0.5);
      expect(state.exportHistory.length, 1);
    });

    test('ExportState copyWith creates modified copy', () {
      // Arrange
      const original = ExportState(
        error: 'Old error',
      );

      // Act
      final modified = original.copyWith(
        isExporting: true,
        error: 'New error',
      );

      // Assert
      expect(modified.isExporting, true);
      expect(modified.error, 'New error');
    });
  });

  group('ExportUtils Extension Tests', () {
    test('ExportUtils description returns correct description', () {
      expect(ExportFormat.markdown.description,
          contains('Markdown format'));
      expect(ExportFormat.html.description, contains('HTML format'));
      expect(ExportFormat.pdf.description, contains('PDF document'));
      expect(ExportFormat.json.description, contains('JSON format'));
    });

    test('ExportUtils icon returns correct icon', () {
      expect(ExportFormat.markdown.icon, '📝');
      expect(ExportFormat.html.icon, '🌐');
      expect(ExportFormat.pdf.icon, '📄');
      expect(ExportFormat.json.icon, '{ }');
    });

    test('ExportUtils supportsOfflineViewing works correctly', () {
      expect(ExportFormat.markdown.supportsOfflineViewing, true);
      expect(ExportFormat.html.supportsOfflineViewing, true);
      expect(ExportFormat.pdf.supportsOfflineViewing, true);
      expect(ExportFormat.json.supportsOfflineViewing, false);
    });

    test('ExportUtils isEditable works correctly', () {
      expect(ExportFormat.markdown.isEditable, true);
      expect(ExportFormat.html.isEditable, true);
      expect(ExportFormat.pdf.isEditable, false);
      expect(ExportFormat.json.isEditable, false);
    });
  });

  group('ExportFormat Edge Cases', () {
    test('ExportFormat handles alternate extensions', () {
      expect(ExportFormat.fromString('htm'), ExportFormat.html);
    });

    test('ExportFormat displayName is user friendly', () {
      expect(ExportFormat.markdown.displayName, 'Markdown');
      expect(ExportFormat.html.displayName, 'HTML');
      expect(ExportFormat.pdf.displayName, 'PDF');
      expect(ExportFormat.json.displayName, 'JSON');
    });

    test('ExportFormat mimeType follows standards', () {
      expect(ExportFormat.markdown.mimeType, 'text/markdown');
      expect(ExportFormat.html.mimeType, 'text/html');
      expect(ExportFormat.pdf.mimeType, 'application/pdf');
      expect(ExportFormat.json.mimeType, 'application/json');
    });

    test('ExportFormat extension includes dot', () {
      expect(ExportFormat.markdown.extension, '.md');
      expect(ExportFormat.html.extension, '.html');
      expect(ExportFormat.pdf.extension, '.pdf');
      expect(ExportFormat.json.extension, '.json');
    });
  });

  group('ExportResult Edge Cases', () {
    test('ExportResult with zero file size', () {
      // Arrange & Act
      final result = ExportResult(
        documentId: 'doc1',
        format: ExportFormat.json,
        fileName: 'empty.json',
        fileSize: 0,
        contentType: 'application/json',
        exportedAt: DateTime(2024),
      );

      // Assert
      expect(result.fileSize, 0);
    });

    test('ExportResult with large file size', () {
      // Arrange & Act
      final result = ExportResult(
        documentId: 'doc1',
        format: ExportFormat.pdf,
        fileName: 'large.pdf',
        fileSize: 10 * 1024 * 1024, // 10 MB
        contentType: 'application/pdf',
        exportedAt: DateTime(2024),
      );

      // Assert
      expect(result.fileSize, 10 * 1024 * 1024);
    });

    test('ExportResult toJson without localFilePath', () {
      // Arrange
      final result = ExportResult(
        documentId: 'doc1',
        format: ExportFormat.markdown,
        fileName: 'doc.md',
        fileSize: 1024,
        contentType: 'text/markdown',
        exportedAt: DateTime(2024),
      );

      // Act
      final json = result.toJson();

      // Assert
      expect(json['local_file_path'], isNull);
    });

    test('ExportResult fromJson without localFilePath', () {
      // Arrange
      final json = {
        'document_id': 'doc1',
        'format': 'markdown',
        'file_name': 'doc.md',
        'file_size': 1024,
        'content_type': 'text/markdown',
        'exported_at': '2024-01-01T00:00:00.000Z',
      };

      // Act
      final result = ExportResult.fromJson(json);

      // Assert
      expect(result.localFilePath, isNull);
    });
  });

  group('ExportState Edge Cases', () {
    test('ExportState with empty export history', () {
      // Arrange & Act
      const state = ExportState(
        
      );

      // Assert
      expect(state.exportHistory, isEmpty);
    });

    test('ExportState with multiple exports in history', () {
      // Arrange
      final now = DateTime(2024);
      final export1 = ExportResult(
        documentId: 'doc1',
        format: ExportFormat.pdf,
        fileName: 'doc1.pdf',
        fileSize: 1024,
        contentType: 'application/pdf',
        exportedAt: now,
      );

      final export2 = ExportResult(
        documentId: 'doc1',
        format: ExportFormat.markdown,
        fileName: 'doc1.md',
        fileSize: 512,
        contentType: 'text/markdown',
        exportedAt: now,
      );

      // Act
      final state = ExportState(
        exportHistory: [export1, export2],
      );

      // Assert
      expect(state.exportHistory.length, 2);
      expect(state.exportHistory[0].format, ExportFormat.pdf);
      expect(state.exportHistory[1].format, ExportFormat.markdown);
    });

    test('ExportState with complete download progress', () {
      // Arrange & Act
      const state = ExportState(
        downloadProgress: 1.0,
      );

      // Assert
      expect(state.downloadProgress, 1.0);
    });

    test('ExportState with no error', () {
      // Arrange & Act
      const state = ExportState();

      // Assert
      expect(state.error, isNull);
    });

    test('ExportState copyWith preserves unchanged values', () {
      // Arrange
      final now = DateTime(2024);
      final lastExport = ExportResult(
        documentId: 'doc1',
        format: ExportFormat.json,
        fileName: 'doc.json',
        fileSize: 512,
        contentType: 'application/json',
        exportedAt: now,
      );

      final original = ExportState(
        isExporting: true,
        lastExport: lastExport,
        error: 'Error',
        downloadProgress: 0.5,
      );

      // Act
      final modified = original.copyWith(
        isExporting: false,
      );

      // Assert
      expect(modified.isExporting, false);
      expect(modified.lastExport, lastExport); // Preserved
      expect(modified.error, 'Error'); // Preserved
      expect(modified.downloadProgress, 0.5); // Preserved
    });
  });
}
