import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/file.dart';

void main() {
  group('FileEntity Tests', () {
    test('FileEntity can be created with required fields', () {
      // Arrange & Act
      const file = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'test.txt',
        fileType: 'text/plain',
        fileSize: 1024,
        downloadUrl: 'https://example.com/test.txt',
      );

      // Assert
      expect(file.id, 'file1');
      expect(file.spaceId, 'space1');
      expect(file.uploadedBy, 'user1');
      expect(file.fileName, 'test.txt');
      expect(file.fileType, 'text/plain');
      expect(file.fileSize, 1024);
      expect(file.downloadUrl, 'https://example.com/test.txt');
      expect(file.documentId, isNull);
      expect(file.createdAt, isNull);
      expect(file.isDeleted, false);
      expect(file.deletedAt, isNull);
    });

    test('FileEntity can be created with all fields', () {
      // Arrange & Act
      final now = DateTime(2024);
      final file = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        documentId: 'doc1',
        uploadedBy: 'user1',
        fileName: 'test.pdf',
        fileType: 'application/pdf',
        fileSize: 2048,
        downloadUrl: 'https://example.com/test.pdf',
        createdAt: now,
      );

      // Assert
      expect(file.id, 'file1');
      expect(file.documentId, 'doc1');
      expect(file.createdAt, now);
    });

    test('FileEntity formattedSize returns correct format', () {
      // Arrange & Act
      const bytesFile = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'small.txt',
        fileType: 'text/plain',
        fileSize: 512,
        downloadUrl: 'url',
      );

      const kbFile = FileEntity(
        id: 'file2',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'medium.txt',
        fileType: 'text/plain',
        fileSize: 5120,
        downloadUrl: 'url',
      );

      const mbFile = FileEntity(
        id: 'file3',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'large.txt',
        fileType: 'text/plain',
        fileSize: 5 * 1024 * 1024,
        downloadUrl: 'url',
      );

      const gbFile = FileEntity(
        id: 'file4',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'huge.bin',
        fileType: 'application/octet-stream',
        fileSize: 2 * 1024 * 1024 * 1024,
        downloadUrl: 'url',
      );

      // Assert
      expect(bytesFile.formattedSize, '512 B');
      expect(kbFile.formattedSize, '5.0 KB');
      expect(mbFile.formattedSize, '5.0 MB');
      expect(gbFile.formattedSize, '2.0 GB');
    });

    test('FileEntity extension returns correct extension', () {
      // Arrange
      const fileWithExt = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'document.pdf',
        fileType: 'application/pdf',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const fileWithoutExt = FileEntity(
        id: 'file2',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'README',
        fileType: 'text/plain',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const fileWithMultipleDots = FileEntity(
        id: 'file3',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'archive.tar.gz',
        fileType: 'application/gzip',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      // Assert
      expect(fileWithExt.extension, 'pdf');
      expect(fileWithoutExt.extension, '');
      expect(fileWithMultipleDots.extension, 'gz');
    });

    test('FileEntity type check getters work correctly', () {
      // Arrange & Act
      const imageFile = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'photo.jpg',
        fileType: 'image/jpeg',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const pdfFile = FileEntity(
        id: 'file2',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'doc.pdf',
        fileType: 'application/pdf',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const videoFile = FileEntity(
        id: 'file3',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'video.mp4',
        fileType: 'video/mp4',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const audioFile = FileEntity(
        id: 'file4',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'audio.mp3',
        fileType: 'audio/mpeg',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      // Assert
      expect(imageFile.isImage, true);
      expect(imageFile.isPdf, false);
      expect(imageFile.isVideo, false);
      expect(imageFile.isAudio, false);

      expect(pdfFile.isImage, false);
      expect(pdfFile.isPdf, true);
      expect(pdfFile.isVideo, false);
      expect(pdfFile.isAudio, false);

      expect(videoFile.isImage, false);
      expect(videoFile.isPdf, false);
      expect(videoFile.isVideo, true);
      expect(videoFile.isAudio, false);

      expect(audioFile.isImage, false);
      expect(audioFile.isPdf, false);
      expect(audioFile.isVideo, false);
      expect(audioFile.isAudio, true);
    });

    test('FileEntity icon returns correct emoji', () {
      // Arrange & Act
      const imageFile = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'photo.jpg',
        fileType: 'image/jpeg',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const pdfFile = FileEntity(
        id: 'file2',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'doc.pdf',
        fileType: 'application/pdf',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const videoFile = FileEntity(
        id: 'file3',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'video.mp4',
        fileType: 'video/mp4',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const audioFile = FileEntity(
        id: 'file4',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'audio.mp3',
        fileType: 'audio/mpeg',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const zipFile = FileEntity(
        id: 'file5',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'archive.zip',
        fileType: 'application/zip',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const wordFile = FileEntity(
        id: 'file6',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'report.docx',
        fileType: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const genericFile = FileEntity(
        id: 'file7',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'unknown.xyz',
        fileType: 'application/unknown',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      // Assert
      expect(imageFile.icon, '🖼️');
      expect(pdfFile.icon, '📄');
      expect(videoFile.icon, '🎬');
      expect(audioFile.icon, '🎵');
      expect(zipFile.icon, '📦');
      expect(wordFile.icon, '📝');
      expect(genericFile.icon, '📎');
    });

    test('FileEntity copyWith creates modified copy', () {
      // Arrange
      const original = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'original.txt',
        fileType: 'text/plain',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      // Act
      final modified = original.copyWith(
        fileName: 'modified.txt',
        fileSize: 2048,
        isDeleted: true,
      );

      // Assert
      expect(modified.id, 'file1'); // Unchanged
      expect(modified.fileName, 'modified.txt');
      expect(modified.fileSize, 2048);
      expect(modified.isDeleted, true);
      expect(modified.uploadedBy, 'user1'); // Unchanged
    });

    test('FileEntity equality uses Equatable (all props)', () {
      // Arrange
      const file1 = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'test.txt',
        fileType: 'text/plain',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const file2 = FileEntity(
        id: 'file1',
        spaceId: 'space1',
        uploadedBy: 'user1',
        fileName: 'test.txt',
        fileType: 'text/plain',
        fileSize: 1024,
        downloadUrl: 'url',
      );

      const file3 = FileEntity(
        id: 'file1',
        spaceId: 'space2', // Different
        uploadedBy: 'user2', // Different
        fileName: 'different.txt', // Different
        fileType: 'text/plain',
        fileSize: 2048, // Different
        downloadUrl: 'url2', // Different
      );

      // Assert - Equatable compares all props, not just ID
      expect(file1, equals(file2)); // All props same
      expect(file1, isNot(equals(file3))); // Some props different
      expect(file1.hashCode, equals(file2.hashCode));
    });
  });

  group('FileUploadProgress Tests', () {
    test('FileUploadProgress can be created', () {
      // Arrange & Act
      const progress = FileUploadProgress(
        fileId: 'file1',
        fileName: 'test.pdf',
        bytesUploaded: 512,
        totalBytes: 1024,
        progress: 0.5,
        status: FileUploadStatus.uploading,
      );

      // Assert
      expect(progress.fileId, 'file1');
      expect(progress.fileName, 'test.pdf');
      expect(progress.bytesUploaded, 512);
      expect(progress.totalBytes, 1024);
      expect(progress.progress, 0.5);
      expect(progress.status, FileUploadStatus.uploading);
      expect(progress.error, isNull);
    });

    test('FileUploadProgress.initial creates initial state', () {
      // Arrange & Act
      final progress = FileUploadProgress.initial('test.pdf', 1024);

      // Assert
      expect(progress.fileId, '');
      expect(progress.fileName, 'test.pdf');
      expect(progress.bytesUploaded, 0);
      expect(progress.totalBytes, 1024);
      expect(progress.progress, 0);
      expect(progress.status, FileUploadStatus.pending);
    });

    test('FileUploadProgress with error', () {
      // Arrange & Act
      const progress = FileUploadProgress(
        fileId: 'file1',
        fileName: 'test.pdf',
        bytesUploaded: 512,
        totalBytes: 1024,
        progress: 0.5,
        status: FileUploadStatus.failed,
        error: 'Network error',
      );

      // Assert
      expect(progress.status, FileUploadStatus.failed);
      expect(progress.error, 'Network error');
    });

    test('FileUploadProgress copyWith creates modified copy', () {
      // Arrange
      const original = FileUploadProgress(
        fileId: 'file1',
        fileName: 'test.pdf',
        bytesUploaded: 0,
        totalBytes: 1024,
        progress: 0,
        status: FileUploadStatus.pending,
      );

      // Act
      final modified = original.copyWith(
        bytesUploaded: 1024,
        progress: 1.0,
        status: FileUploadStatus.completed,
      );

      // Assert
      expect(modified.fileId, 'file1'); // Unchanged
      expect(modified.bytesUploaded, 1024);
      expect(modified.progress, 1.0);
      expect(modified.status, FileUploadStatus.completed);
    });

    test('FileUploadProgress represents complete upload', () {
      // Arrange & Act
      const progress = FileUploadProgress(
        fileId: 'file1',
        fileName: 'test.pdf',
        bytesUploaded: 1024,
        totalBytes: 1024,
        progress: 1.0,
        status: FileUploadStatus.completed,
      );

      // Assert
      expect(progress.progress, 1.0);
      expect(progress.status, FileUploadStatus.completed);
      expect(progress.bytesUploaded, progress.totalBytes);
    });
  });

  group('FileUploadStatus Enum Tests', () {
    test('FileUploadStatus has all values', () {
      // Assert
      expect(FileUploadStatus.pending, isNotNull);
      expect(FileUploadStatus.uploading, isNotNull);
      expect(FileUploadStatus.completed, isNotNull);
      expect(FileUploadStatus.failed, isNotNull);
      expect(FileUploadStatus.cancelled, isNotNull);
    });

    test('FileUploadStatus values are correct', () {
      // Assert
      expect(FileUploadStatus.pending.toString(), contains('pending'));
      expect(FileUploadStatus.uploading.toString(), contains('uploading'));
      expect(FileUploadStatus.completed.toString(), contains('completed'));
      expect(FileUploadStatus.failed.toString(), contains('failed'));
      expect(FileUploadStatus.cancelled.toString(), contains('cancelled'));
    });
  });
}
