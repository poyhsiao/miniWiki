import 'dart:async';

import 'package:mocktail/mocktail.dart';
import 'package:flutter_test/flutter_test.dart';

/// 測試輔助工具類
class TestHelpers {
  /// 註冊所有 Mock 類的 fallback 值
  static void registerFallbacks() {
    registerFallbackValue(const MockApiRequest());
    registerFallbackValue(const MockApiResponse());
    registerFallbackValue(const MockDocumentData());
  }

  /// 設置基本的測試配置
  static void setupTestEnvironment() {
    TestWidgetsFlutterBinding.ensureInitialized();
    registerFallbacks();
  }

  /// 創建測試用的 UUID
  static String createTestUuid([String suffix = '']) {
    return 'test-uuid-${DateTime.now().millisecondsSinceEpoch}-$suffix';
  }

  /// 創建測試用的時間戳
  static DateTime createTestTimestamp([int daysAgo = 0]) {
    return DateTime.now().subtract(Duration(days: daysAgo));
  }
}

/// Mock 數據類
class MockApiRequest {
  const MockApiRequest();
}

class MockApiResponse {
  const MockApiResponse();
}

class MockDocumentData {
  const MockDocumentData();
}

/// 異步測試輔助函數
/// 用於測試異步操作是否完成
Future<void> delay([int milliseconds = 100]) async {
  await Future.delayed(Duration(milliseconds: milliseconds));
}

/// 驗證是否拋出特定類型的異常
Future<void> expectThrows<T>(Future<void> Function() callback) async {
  try {
    await callback();
    fail('Expected exception of type $T but none was thrown');
  } on TestFailure {
    // 重新拋出測試框架失敗，不將其視為預期異常
    rethrow;
  } catch (e) {
    expect(e, isA<T>());
  }
}

/// 驗證 Future 是否完成超時
Future<void> expectTimeout(
  Future<void> Function() callback, {
  Duration timeout = const Duration(seconds: 5),
}) async {
  bool timedOut = false;
  try {
    await callback().timeout(timeout);
  } on TimeoutException catch (_) {
    timedOut = true;
  }
  expect(timedOut, isTrue);
}
