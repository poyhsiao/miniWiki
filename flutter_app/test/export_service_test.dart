import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:dio/dio.dart';
import 'package:miniwiki/services/export_service.dart';
import 'package:miniwiki/core/network/api_client.dart';

class MockApiClient extends Mock implements ApiClient {}

class MockDio extends Mock implements Dio {}

void main() {
  late ExportService exportService;
  late MockApiClient mockApiClient;

  setUp(() {
    mockApiClient = MockApiClient();
    exportService = ExportService(
      apiClient: mockApiClient,
      baseUrl: 'http://localhost:8080',
    );
  });

  group('ExportService', () {
    group('exportDocument', () {
      test('should throw ArgumentError when documentId is empty', () async {
        expect(
          () => exportService.exportDocument(
            documentId: '',
            format: ExportFormat.markdown,
          ),
          throwsA(isA<ArgumentError>()),
        );
      });

      test('should throw NetworkError on failed export', () async {
        when(() => mockApiClient.dio.get(
              any(),
              queryParameters: any(named: 'queryParameters'),
              options: any(named: 'options'),
              onReceiveProgress: any(named: 'onReceiveProgress'),
            )).thenThrow(
          DioException(
            requestOptions: RequestOptions(path: '/test'),
            error: 'Connection failed',
          ),
        );

        expect(
          () => exportService.exportDocument(
            documentId: 'test-doc-id',
            format: ExportFormat.markdown,
          ),
          throwsA(isA<DioException>()),
        );
      });
    });

    group('getExportUrl', () {
      test('should throw ArgumentError when documentId is empty', () async {
        expect(
          () => exportService.getExportUrl(
            documentId: '',
            format: ExportFormat.markdown,
          ),
          throwsA(isA<ArgumentError>()),
        );
      });

      test('should return correct export URL', () async {
        final url = await exportService.getExportUrl(
          documentId: 'doc-123',
          format: ExportFormat.markdown,
        );

        expect(
          url,
          equals(
              'http://localhost:8080/api/v1/documents/doc-123/export?format=markdown'),
        );
      });
    });

    group('exportAndOpen', () {
      test('should throw NetworkError when file save fails', () async {
        when(() => mockApiClient.dio.get(
              any(),
              queryParameters: any(named: 'queryParameters'),
              options: any(named: 'options'),
              onReceiveProgress: any(named: 'onReceiveProgress'),
            )).thenThrow(
          DioException(
            requestOptions: RequestOptions(path: '/test'),
            error: 'Connection failed',
          ),
        );

        expect(
          () => exportService.exportAndOpen(
            documentId: 'test-doc-id',
            format: ExportFormat.html,
          ),
          throwsA(isA<DioException>()),
        );
      });
    });

    group('getSupportedFormats', () {
      test('should return all available formats', () async {
        final formats = await exportService.getSupportedFormats('doc-123');

        expect(formats, equals(ExportFormat.availableFormats));
        expect(formats.length, equals(4));
      });
    });

    group('ExportFormat', () {
      test('fromString should return correct format for valid inputs', () {
        expect(
            ExportFormat.fromString('markdown'), equals(ExportFormat.markdown));
        expect(ExportFormat.fromString('md'), equals(ExportFormat.markdown));
        expect(ExportFormat.fromString('html'), equals(ExportFormat.html));
        expect(ExportFormat.fromString('htm'), equals(ExportFormat.html));
        expect(ExportFormat.fromString('pdf'), equals(ExportFormat.pdf));
        expect(ExportFormat.fromString('json'), equals(ExportFormat.json));
      });

      test('fromString should return null for invalid input', () {
        expect(ExportFormat.fromString('invalid'), isNull);
        expect(ExportFormat.fromString(''), isNull);
      });

      test('availableFormats should contain all formats', () {
        final formats = ExportFormat.availableFormats;

        expect(formats.contains(ExportFormat.markdown), isTrue);
        expect(formats.contains(ExportFormat.html), isTrue);
        expect(formats.contains(ExportFormat.pdf), isTrue);
        expect(formats.contains(ExportFormat.json), isTrue);
      });
    });

    group('ExportUtils', () {
      test('markdown format should have correct description', () {
        expect(
          ExportFormat.markdown.description,
          contains('Markdown format with frontmatter metadata'),
        );
      });

      test('html format should have correct description', () {
        expect(
          ExportFormat.html.description,
          contains('HTML format with embedded styles'),
        );
      });

      test('pdf format should have correct description', () {
        expect(
          ExportFormat.pdf.description,
          contains('PDF document'),
        );
      });

      test('markdown format should be editable', () {
        expect(ExportFormat.markdown.isEditable, isTrue);
      });

      test('html format should be editable', () {
        expect(ExportFormat.html.isEditable, isTrue);
      });

      test('pdf format should not be editable', () {
        expect(ExportFormat.pdf.isEditable, isFalse);
      });

      test('json format should not support offline viewing', () {
        expect(ExportFormat.json.supportsOfflineViewing, isFalse);
      });

      test('markdown should support offline viewing', () {
        expect(ExportFormat.markdown.supportsOfflineViewing, isTrue);
      });
    });

    group('ExportResult', () {
      test('should correctly serialize to JSON', () {
        final result = ExportResult(
          documentId: 'doc-123',
          format: ExportFormat.markdown,
          fileName: 'test.md',
          fileSize: 1024,
          contentType: 'text/markdown',
          exportedAt: DateTime(2024, 1, 15, 12, 0, 0),
          localFilePath: '/path/to/file.md',
        );

        final json = result.toJson();

        expect(json['document_id'], equals('doc-123'));
        expect(json['format'], equals('markdown'));
        expect(json['file_name'], equals('test.md'));
        expect(json['file_size'], equals(1024));
        expect(json['content_type'], equals('text/markdown'));
        expect(json['local_file_path'], equals('/path/to/file.md'));
      });

      test('should correctly deserialize from JSON', () {
        final json = {
          'document_id': 'doc-456',
          'format': 'html',
          'file_name': 'test.html',
          'file_size': 2048,
          'content_type': 'text/html',
          'exported_at': '2024-01-15T12:00:00.000',
          'local_file_path': '/path/to/html',
        };

        final result = ExportResult.fromJson(json);

        expect(result.documentId, equals('doc-456'));
        expect(result.format, equals(ExportFormat.html));
        expect(result.fileName, equals('test.html'));
        expect(result.fileSize, equals(2048));
        expect(result.contentType, equals('text/html'));
        expect(result.localFilePath, equals('/path/to/html'));
      });
    });

    group('ExportState', () {
      test('should have correct default values', () {
        const state = ExportState();

        expect(state.isExporting, isFalse);
        expect(state.lastExport, isNull);
        expect(state.error, isNull);
        expect(state.downloadProgress, isNull);
        expect(state.exportHistory, isEmpty);
      });

      test('copyWith should update only specified fields', () {
        const initialState = ExportState();
        final updatedState = initialState.copyWith(
          isExporting: true,
          error: 'Test error',
        );

        expect(updatedState.isExporting, isTrue);
        expect(updatedState.error, equals('Test error'));
        expect(updatedState.downloadProgress, isNull);
      });
    });
  });
}
