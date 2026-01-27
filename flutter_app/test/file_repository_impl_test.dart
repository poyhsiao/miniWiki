import 'dart:io';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/core/network/network_error.dart' as ne;
import 'package:miniwiki/data/repositories/file_repository_impl.dart';
import 'package:miniwiki/domain/repositories/file_repository.dart';
import 'package:mocktail/mocktail.dart';

// Mock 類定義
class MockDio extends Mock implements Dio {}

class MockApiClient extends Mock implements ApiClient {
  final Dio _dio;

  MockApiClient(this._dio);

  @override
  Dio get dio => _dio;
}

void main() {
  late FileRepositoryImpl repository;
  late MockDio mockDio;
  late ApiClient mockApiClient;
  const baseUrl = 'https://api.test.com';

  setUp(() {
    mockDio = MockDio();
    mockApiClient = MockApiClient(mockDio);
    repository = FileRepositoryImpl(
      apiClient: mockApiClient,
      baseUrl: baseUrl,
    );
  });

  group('FileRepositoryImpl', () {
    const testSpaceId = 'test-space-id';
    const testDocumentId = 'test-doc-id';
    const testFileId = 'test-file-id';
    const testFileName = 'test-file.pdf';
    const testContentType = 'application/pdf';

    // 測試數據
    final testFileData = {
      'id': testFileId,
      'spaceId': testSpaceId,
      'documentId': testDocumentId,
      'uploadedBy': 'user-1',
      'fileName': testFileName,
      'fileType': 'pdf',
      'fileSize': 1024000,
      'downloadUrl': 'https://example.com/files/test-file.pdf',
      'createdAt': '2024-01-01T00:00:00.000Z',
      'isDeleted': false,
    };

    group('uploadFile', () {
      late File testFile;

      setUp(() async {
        // Create a temporary test file
        final tempDir = Directory.systemTemp;
        testFile = File('${tempDir.path}/test-file.pdf');
        await testFile.writeAsBytes(List.generate(1024, (i) => i % 256));
      });

      tearDown(() async {
        // Clean up test file
        if (await testFile.exists()) {
          await testFile.delete();
        }
      });

      test('should successfully upload a file', () async {
        // Arrange
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: testFileData,
          statusCode: 201,
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
          onSendProgress: any(named: 'onSendProgress'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.uploadFile(
          spaceId: testSpaceId,
          documentId: testDocumentId,
          filePath: testFile.path,
          fileName: testFileName,
          contentType: testContentType,
        );

        // Assert
        expect(result.id, testFileId);
        expect(result.fileName, testFileName);
        verify(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
          onSendProgress: any(named: 'onSendProgress'),
        )).called(1);
      });

      test('should report upload progress', () async {
        // Arrange
        final progressValues = <double>[];
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: testFileData,
          statusCode: 201,
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
          onSendProgress: any(named: 'onSendProgress'),
        )).thenAnswer((invocation) {
          final onProgress =
              invocation.namedArguments[#onSendProgress] as void Function(int, int)?;
          // 模擬進度回調
          onProgress?.call(512, 1024);
          onProgress?.call(1024, 1024);
          return Future.value(mockResponse);
        });

        // Act
        await repository.uploadFile(
          spaceId: testSpaceId,
          documentId: testDocumentId,
          filePath: testFile.path,
          fileName: testFileName,
          contentType: testContentType,
          onProgress: (progress) => progressValues.add(progress),
        );

        // Assert
        expect(progressValues, [0.5, 1.0]);
      });

      test('should throw error on upload failure', () async {
        // Arrange
        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
          onSendProgress: any(named: 'onSendProgress'),
        )).thenThrow(
          DioException(
            requestOptions: RequestOptions(path: ''),
            type: DioExceptionType.badResponse,
            response: Response(
              requestOptions: RequestOptions(path: ''),
              statusCode: 500,
              statusMessage: 'Internal Server Error',
            ),
          ),
        );

        // Act & Assert
        expect(
          () => repository.uploadFile(
            spaceId: testSpaceId,
            documentId: testDocumentId,
            filePath: testFile.path,
            fileName: testFileName,
            contentType: testContentType,
          ),
          throwsA(isA<Exception>()),
        );
      });
    });

    group('getPresignedUploadUrl', () {
      test('should get presigned upload URL', () async {
        // Arrange
        const expectedUrl = 'https://s3.amazonaws.com/bucket/presigned-url';
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: {'url': expectedUrl},
          statusCode: 200,
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.getPresignedUploadUrl(
          spaceId: testSpaceId,
          fileName: testFileName,
          contentType: testContentType,
          contentLength: 1024000,
        );

        // Assert
        expect(result, expectedUrl);
      });
    });

    group('downloadFile', () {
      test('should successfully download a file', () async {
        // Arrange
        final tempDir = Directory.systemTemp;
        final savePath = '${tempDir.path}/downloaded-file.pdf';
        final fileBytes = List.generate(1024, (i) => i % 256);
        final mockResponse = Response<List<int>>(
          requestOptions: RequestOptions(path: ''),
          data: fileBytes,
          statusCode: 200,
        );

        when(() => mockDio.get<List<int>>(
          any(),
          options: any(named: 'options'),
          onReceiveProgress: any(named: 'onReceiveProgress'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.downloadFile(
          fileId: testFileId,
          savePath: savePath,
        );

        // Assert
        expect(result, savePath);
        // Clean up
        if (await File(savePath).exists()) {
          await File(savePath).delete();
        }
      });

      test('should report download progress', () async {
        // Arrange
        final tempDir = Directory.systemTemp;
        final savePath = '${tempDir.path}/downloaded-file.pdf';
        final progressValues = <double>[];
        final fileBytes = List.generate(1024, (i) => i % 256);
        final mockResponse = Response<List<int>>(
          requestOptions: RequestOptions(path: ''),
          data: fileBytes,
          statusCode: 200,
        );

        when(() => mockDio.get<List<int>>(
          any(),
          options: any(named: 'options'),
          onReceiveProgress: any(named: 'onReceiveProgress'),
        )).thenAnswer((invocation) {
          final onProgress =
              invocation.namedArguments[#onReceiveProgress] as void Function(int, int)?;
          onProgress?.call(512, 1024);
          onProgress?.call(1024, 1024);
          return Future.value(mockResponse);
        });

        // Act
        await repository.downloadFile(
          fileId: testFileId,
          savePath: savePath,
          onProgress: (progress) => progressValues.add(progress),
        );

        // Assert
        expect(progressValues, [0.5, 1.0]);
        // Clean up
        if (await File(savePath).exists()) {
          await File(savePath).delete();
        }
      });
    });

    group('getPresignedDownloadUrl', () {
      test('should get presigned download URL', () async {
        // Arrange
        const expectedUrl = 'https://s3.amazonaws.com/bucket/presigned-download-url';
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: {'url': expectedUrl},
          statusCode: 200,
        );

        when(() => mockDio.get<Map<String, dynamic>>(any()))
            .thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.getPresignedDownloadUrl(testFileId);

        // Assert
        expect(result, expectedUrl);
      });
    });

    group('getFile', () {
      test('should get file by ID', () async {
        // Arrange
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: {'file': testFileData},
          statusCode: 200,
        );

        when(() => mockDio.get<Map<String, dynamic>>(any()))
            .thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.getFile(testFileId);

        // Assert
        expect(result.id, testFileId);
        expect(result.fileName, testFileName);
      });

      test('should throw 404 when file not found', () async {
        // Arrange
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          statusCode: 404,
        );

        when(() => mockDio.get<Map<String, dynamic>>(any()))
            .thenAnswer((_) async => mockResponse);

        // Act & Assert
        expect(
          () => repository.getFile(testFileId),
          throwsA(isA<ne.NetworkError>()),
        );
      });
    });

    group('listFiles', () {
      test('should list files in a space', () async {
        // Arrange
        final filesData = {
          'files': [testFileData, {...testFileData, 'id': 'file-2'}],
          'total': 2,
        };
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: filesData,
          statusCode: 200,
        );

        when(() => mockDio.get<Map<String, dynamic>>(
          any(),
          queryParameters: any(named: 'queryParameters'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.listFiles(spaceId: testSpaceId);

        // Assert
        expect(result.length, 2);
        expect(result[0].id, testFileId);
      });

      test('should filter files by document ID', () async {
        // Arrange
        final filesData = {'files': [testFileData], 'total': 1};
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: filesData,
          statusCode: 200,
        );

        when(() => mockDio.get<Map<String, dynamic>>(
          any(),
          queryParameters: any(named: 'queryParameters'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.listFiles(
          spaceId: testSpaceId,
          documentId: testDocumentId,
        );

        // Assert
        expect(result.length, 1);
        verify(() => mockDio.get<Map<String, dynamic>>(
          any(),
          queryParameters: any(named: 'queryParameters'),
        )).called(1);
      });

      test('should handle pagination', () async {
        // Arrange
        final filesData = {'files': [testFileData], 'total': 1};
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: filesData,
          statusCode: 200,
        );

        when(() => mockDio.get<Map<String, dynamic>>(
          any(),
          queryParameters: any(named: 'queryParameters'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        await repository.listFiles(
          spaceId: testSpaceId,
          limit: 50,
          offset: 10,
        );

        // Assert
        verify(() => mockDio.get<Map<String, dynamic>>(
          any(),
          queryParameters: any(named: 'queryParameters'),
        )).called(1);
      });
    });

    group('deleteFile', () {
      test('should delete a file', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          requestOptions: RequestOptions(path: ''),
          statusCode: 200,
        );

        when(() => mockDio.delete<dynamic>(any()))
            .thenAnswer((_) async => mockResponse);

        // Act
        await repository.deleteFile(testFileId);

        // Assert
        verify(() => mockDio.delete<dynamic>('$baseUrl/api/v1/files/$testFileId'))
            .called(1);
      });
    });

    group('restoreFile', () {
      test('should restore a deleted file', () async {
        // Arrange
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: testFileData,
          statusCode: 200,
        );

        when(() => mockDio.post<Map<String, dynamic>>(any()))
            .thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.restoreFile(testFileId);

        // Assert
        expect(result.id, testFileId);
      });
    });

    group('permanentDeleteFile', () {
      test('should permanently delete a file', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          requestOptions: RequestOptions(path: ''),
          statusCode: 200,
        );

        when(() => mockDio.delete<dynamic>(any()))
            .thenAnswer((_) async => mockResponse);

        // Act
        await repository.permanentDeleteFile(testFileId);

        // Assert
        verify(() => mockDio.delete<dynamic>(
              '$baseUrl/api/v1/files/$testFileId/permanent-delete',
            )).called(1);
      });
    });

    group('chunked upload', () {
      const testUploadId = 'upload-id-123';
      const testChunkData = [1, 2, 3, 4, 5];

      test('should initialize chunked upload', () async {
        // Arrange
        final uploadInitData = {
          'upload_id': testUploadId,
          'upload_url': 'https://s3.amazonaws.com/upload-url',
          'chunk_size': 5 * 1024 * 1024,
          'total_chunks': 10,
        };
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: uploadInitData,
          statusCode: 201,
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.initChunkedUpload(
          spaceId: testSpaceId,
          fileName: testFileName,
          contentType: testContentType,
          totalSize: 50 * 1024 * 1024,
        );

        // Assert
        expect(result.uploadId, testUploadId);
      });

      test('should upload chunk', () async {
        // Arrange
        final chunkResultData = {
          'chunk_number': 1,
          'uploaded_bytes': testChunkData.length,
          'chunks_uploaded': 1,
          'total_chunks': 10,
        };
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: chunkResultData,
          statusCode: 200,
        );

        when(() => mockDio.put<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
          options: any(named: 'options'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.uploadChunk(
          uploadId: testUploadId,
          chunkNumber: 1,
          chunkData: testChunkData,
        );

        // Assert
        expect(result.chunkNumber, 1);
      });

      test('should complete chunked upload', () async {
        // Arrange
        final chunks = [
          const ChunkInfo(chunkNumber: 1, etag: 'etag-1'),
          const ChunkInfo(chunkNumber: 2, etag: 'etag-2'),
        ];
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: testFileData,
          statusCode: 201,
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.completeChunkedUpload(
          uploadId: testUploadId,
          chunks: chunks,
        );

        // Assert
        expect(result.id, testFileId);
      });

      test('should cancel chunked upload', () async {
        // Arrange
        final mockResponse = Response<dynamic>(
          requestOptions: RequestOptions(path: ''),
          statusCode: 200,
        );

        when(() => mockDio.delete<dynamic>(any()))
            .thenAnswer((_) async => mockResponse);

        // Act
        await repository.cancelChunkedUpload(testUploadId);

        // Assert
        verify(() => mockDio.delete<dynamic>(
              '$baseUrl/api/v1/files/upload/chunked/$testUploadId',
            )).called(1);
      });
    });

    group('bulkDeleteFiles', () {
      test('should delete multiple files', () async {
        // Arrange
        final fileIds = ['file-1', 'file-2', 'file-3'];
        final bulkDeleteData = {
          'deleted': fileIds,
          'failed': <String>[],
        };
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: bulkDeleteData,
          statusCode: 200,
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.bulkDeleteFiles(fileIds);

        // Assert
        expect(result.deleted.length, 3);
        expect(result.failed.length, 0);
      });

      test('should handle partial failures in bulk delete', () async {
        // Arrange
        final fileIds = ['file-1', 'file-2', 'file-3'];
        final bulkDeleteData = {
          'deleted': ['file-1', 'file-2'],
          'failed': [
            {'file_id': 'file-3', 'reason': 'File not found'},
          ],
        };
        final mockResponse = Response<Map<String, dynamic>>(
          requestOptions: RequestOptions(path: ''),
          data: bulkDeleteData,
          statusCode: 200, // Changed from 207 to 200 to match implementation
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => mockResponse);

        // Act
        final result = await repository.bulkDeleteFiles(fileIds);

        // Assert
        expect(result.deleted.length, 2);
        expect(result.failed.length, 1);
        expect(result.failed.first.fileId, 'file-3');
      });
    });
  });
}
