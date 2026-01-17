# Android Build Verification for share_plus 12.0.0 Upgrade

## ⚠️ 重要提醒

在合并此 PR 之前，必须完成以下 Android 构建验证步骤，以确保 `share_plus` 从旧版本升级到 12.0.0 不会导致 Android 构建失败。

## 验证步骤

### 1. 清理构建缓存
```bash
cd flutter_app
flutter clean
```

### 2. 获取依赖
```bash
flutter pub get
```

### 3. 验证 Android APK 构建
```bash
flutter build apk
```

**预期结果**: 构建成功，无错误

**可能的问题**:
- `GeneratedPluginRegistrant` 注册错误
- `compileSdk` 版本不兼容
- Android Gradle Plugin (AGP) 版本冲突

### 4. 在设备/模拟器上运行
```bash
flutter run
```

**预期结果**: 应用成功启动，无崩溃

### 5. 测试 share_plus 功能
在应用中触发文件分享功能，确保：
- 分享对话框正常弹出
- 可以选择分享目标应用
- 分享操作成功完成

## 如果遇到构建错误

### 错误 1: Plugin Registration 失败

**症状**: `GeneratedPluginRegistrant` 相关错误

**解决方案**:
1. 检查 `android/app/src/main/kotlin/.../MainActivity.kt` (或 `.java`)
2. 确保使用正确的 Flutter embedding v2 API
3. 可能需要更新 MainActivity 的代码以适配新版本插件

### 错误 2: compileSdk 版本问题

**症状**: 编译 SDK 版本不兼容错误

**解决方案**:
1. 打开 `android/app/build.gradle`
2. 检查 `compileSdk` 版本（建议 34 或更高）
3. 更新 `targetSdk` 到相应版本

### 错误 3: AGP 版本冲突

**症状**: Android Gradle Plugin 版本冲突

**解决方案**:
1. 打开 `android/build.gradle`
2. 检查 AGP 版本（必须 >= 8.12.1，建议 8.12.1 或更高）
3. 确保 Gradle 版本 >= 8.13
4. 确保 Kotlin 版本为 2.2.0

## 回滚方案

如果验证失败且无法快速修复，执行以下操作：

1. 将 `pubspec.yaml` 中的 `share_plus` 版本回滚到之前的稳定版本
2. 运行 `flutter pub get`
3. 重新构建和测试

```yaml
# 回滚示例
dependencies:
  share_plus: ^9.0.0  # 或之前使用的稳定版本
```

## 验证检查清单

- [ ] `flutter clean` 已执行
- [ ] `flutter pub get` 成功
- [ ] `flutter build apk` 成功（无错误）
- [ ] `flutter run` 在设备/模拟器上成功启动
- [ ] 文件分享功能测试通过
- [ ] 无 GeneratedPluginRegistrant 错误
- [ ] 无 compileSdk/AGP 兼容性问题
- [ ] 应用在 Android 设备上稳定运行

## 注意事项

- Flutter SDK 约束已设置为 `>=3.27.0`，确保本地 Flutter 版本符合要求
- share_plus 12.0.0 需要以下最低 Android 配置:
  - Java 17 (required)
  - Android Gradle Plugin (AGP) >= 8.12.1
  - Gradle >= 8.13
  - Kotlin 2.2.0
- 建议在多个 Android 版本上测试（API 21+）

## 完成验证后

在 PR 中添加评论，说明：
1. 验证环境（Flutter 版本、Android SDK 版本）
2. 测试的 Android 版本
3. 构建和运行的结果
4. 任何遇到的问题及解决方案

---
**创建日期**: 2026-01-17
**相关 Issue**: share_plus 12.0.0 升级
**负责人**: 开发团队
