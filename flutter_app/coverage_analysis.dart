#!/usr/bin/env dart

/// 覆蓋率分析工具
/// 解析 lcov.info 並生成詳細的覆蓋率報告

import 'dart:io';

class CoverageData {
  final String filePath;
  final int linesFound;
  final int linesHit;
  final int functionsFound;
  final int functionsHit;
  final int branchesFound;
  final int branchesHit;

  CoverageData({
    required this.filePath,
    required this.linesFound,
    required this.linesHit,
    this.functionsFound = 0,
    this.functionsHit = 0,
    this.branchesFound = 0,
    this.branchesHit = 0,
  });

  CoverageData copyWith({
    String? filePath,
    int? linesFound,
    int? linesHit,
    int? functionsFound,
    int? functionsHit,
    int? branchesFound,
    int? branchesHit,
  }) {
    return CoverageData(
      filePath: filePath ?? this.filePath,
      linesFound: linesFound ?? this.linesFound,
      linesHit: linesHit ?? this.linesHit,
      functionsFound: functionsFound ?? this.functionsFound,
      functionsHit: functionsHit ?? this.functionsHit,
      branchesFound: branchesFound ?? this.branchesFound,
      branchesHit: branchesHit ?? this.branchesHit,
    );
  }

  double get lineCoverage => linesFound > 0 ? (linesHit / linesFound) * 100 : 0;
  double get functionCoverage =>
      functionsFound > 0 ? (functionsHit / functionsFound) * 100 : 0;
  double get branchCoverage =>
      branchesFound > 0 ? (branchesHit / branchesFound) * 100 : 0;
}

class CoverageReport {
  final List<CoverageData> files;
  final Map<String, List<CoverageData>> filesByDirectory;

  CoverageReport(this.files) : filesByDirectory = _groupFilesByDirectory(files);

  static Map<String, List<CoverageData>> _groupFilesByDirectory(
      List<CoverageData> files) {
    final Map<String, List<CoverageData>> grouped = {};

    for (var file in files) {
      final dir = _getDirectoryName(file.filePath);
      grouped.putIfAbsent(dir, () => []).add(file);
    }

    return grouped;
  }

  static String _getDirectoryName(String filePath) {
    final parts = filePath.split('/');
    if (parts.length <= 2) return '/';

    // 返回 lib 下的頂層目錄 (例如: lib/services -> services)
    if (parts[0] == 'lib' && parts.length > 1) {
      return parts[1];
    }
    return parts[0];
  }

  List<CoverageData> get lowestCoverage {
    final sorted = List<CoverageData>.from(files);
    sorted.sort((a, b) => a.lineCoverage.compareTo(b.lineCoverage));
    return sorted.take(10).toList();
  }

  List<CoverageData> get largestCoverageGaps {
    final withGaps = files.where((f) =>
        f.linesFound - f.linesHit > 20).toList();
    withGaps.sort((a, b) =>
        (b.linesFound - b.linesHit).compareTo(a.linesFound - a.linesHit));
    return withGaps.take(10).toList();
  }

  double get overallCoverage {
    if (files.isEmpty) return 0;
    final totalLines = files.fold<int>(0, (sum, f) => sum + f.linesFound);
    final totalHit = files.fold<int>(0, (sum, f) => sum + f.linesHit);
    return totalLines > 0 ? (totalHit / totalLines) * 100 : 0;
  }
}

CoverageReport parseLcov(String content) {
  final List<CoverageData> files = [];
  CoverageData? currentFile;

  final lines = content.split('\n');

  for (var line in lines) {
    if (line.startsWith('SF:')) {
      // 保存前一個文件
      if (currentFile != null) {
        files.add(currentFile);
      }

      // 開始新文件
      final filePath = line.substring(3);
      if (filePath.endsWith('.g.dart')) {
        // 跳過生成的文件
        currentFile = null;
        continue;
      }
      currentFile = CoverageData(
        filePath: filePath,
        linesFound: 0,
        linesHit: 0,
      );
    } else if (line.startsWith('LF:') && currentFile != null) {
      final value = int.tryParse(line.substring(3)) ?? 0;
      currentFile = currentFile.copyWith(
        linesFound: value,
      );
    } else if (line.startsWith('LH:') && currentFile != null) {
      final value = int.tryParse(line.substring(3)) ?? 0;
      currentFile = currentFile.copyWith(
        linesHit: value,
      );
    } else if (line.startsWith('FNF:') && currentFile != null) {
      currentFile = currentFile.copyWith(
        functionsFound: int.parse(line.substring(4)),
      );
    } else if (line.startsWith('FNH:') && currentFile != null) {
      currentFile = currentFile.copyWith(
        functionsHit: int.parse(line.substring(4)),
      );
    } else if (line.startsWith('BRF:') && currentFile != null) {
      currentFile = currentFile.copyWith(
        branchesFound: int.parse(line.substring(4)),
      );
    } else if (line.startsWith('BRH:') && currentFile != null) {
      currentFile = currentFile.copyWith(
        branchesHit: int.parse(line.substring(4)),
      );
    } else if (line == 'end_of_record' && currentFile != null) {
      files.add(currentFile);
      currentFile = null;
    }
  }

  // 添加最後一個文件
  if (currentFile != null) {
    files.add(currentFile);
  }

  return CoverageReport(files);
}

void printReport(CoverageReport report) {
  print('\n' + '=' * 80);
  print(' 測試覆蓋率分析報告'.padLeft(50));
  print('=' * 80);

  print('\n總體覆蓋率: ${report.overallCoverage.toStringAsFixed(1)}%');
  print('已測試文件數: ${report.files.length}');

  // 按目錄分組的覆蓋率
  print('\n' + '-' * 80);
  print('各模塊覆蓋率:');
  print('-' * 80);

  final sortedDirs = report.filesByDirectory.entries.toList()
    ..sort((a, b) {
      final avgA = a.value.isEmpty ? 0 :
        a.value.map((f) => f.lineCoverage).fold<double>(0, (sum, cov) => sum + cov) / a.value.length;
      final avgB = b.value.isEmpty ? 0 :
        b.value.map((f) => f.lineCoverage).fold<double>(0, (sum, cov) => sum + cov) / b.value.length;
      return avgA.compareTo(avgB);
    });

  for (var entry in sortedDirs) {
    final files = entry.value;
    if (files.isEmpty) continue;

    final avgCoverage = files
        .map((f) => f.lineCoverage)
        .reduce((a, b) => a + b) / files.length;
    final totalLines = files.fold<int>(0, (sum, f) => sum + f.linesFound);
    final totalHit = files.fold<int>(0, (sum, f) => sum + f.linesHit);

    print('\n${entry.key}:');
    print('  覆蓋率: ${avgCoverage.toStringAsFixed(1)}%');
    print('  文件數: ${files.length}');
    print('  代碼行: $totalHit/$totalLines');
  }

  // 覆蓋率最低的10個文件
  print('\n' + '-' * 80);
  print('覆蓋率最低的 10 個文件 (優先測試目標):');
  print('-' * 80);

  for (var i = 0; i < report.lowestCoverage.length; i++) {
    final file = report.lowestCoverage[i];
    print('\n${i + 1}. ${file.filePath}');
    print('   覆蓋率: ${file.lineCoverage.toStringAsFixed(1)}% (${file.linesHit}/${file.linesFound} 行)');
    print('   未覆蓋: ${file.linesFound - file.linesHit} 行');
  }

  // 最大的覆蓋率缺口
  print('\n' + '-' * 80);
  print('需要最多測試工作的文件 (未覆蓋行數 > 20):');
  print('-' * 80);

  final gaps = report.largestCoverageGaps;
  if (gaps.isEmpty) {
    print('沒有文件需要超過20行的測試補充');
  } else {
    for (var i = 0; i < gaps.length && i < 10; i++) {
      final file = gaps[i];
      print('\n${i + 1}. ${file.filePath}');
      print('   覆蓋率: ${file.lineCoverage.toStringAsFixed(1)}%');
      print('   未覆蓋: ${file.linesFound - file.linesHit} 行 (總計: ${file.linesFound} 行)');
    }
  }

  // 改進建議
  print('\n' + '-' * 80);
  print('改進建議:');
  print('-' * 80);

  final critical = report.lowestCoverage.where((f) => f.lineCoverage < 30).toList();
  final needsWork = report.lowestCoverage
      .where((f) => f.lineCoverage >= 30 && f.lineCoverage < 70).toList();
  final good = report.lowestCoverage.where((f) => f.lineCoverage >= 70).toList();

  print('\n優先級 1 - 緊急 (< 30% 覆蓋率): ${critical.length} 個文件');
  for (var file in critical.take(5)) {
    print('  - ${file.filePath} (${file.lineCoverage.toStringAsFixed(1)}%)');
  }

  print('\n優先級 2 - 重要 (30-70% 覆蓋率): ${needsWork.length} 個文件');
  for (var file in needsWork.take(5)) {
    print('  - ${file.filePath} (${file.lineCoverage.toStringAsFixed(1)}%)');
  }

  print('\n優先級 3 - 改進 (> 70% 覆蓋率): ${good.length} 個文件');
  for (var file in good.take(5)) {
    print('  - ${file.filePath} (${file.lineCoverage.toStringAsFixed(1)}%)');
  }

  print('\n' + '=' * 80);
}

void main() async {
  final lcovPath = 'coverage/lcov.info';

  final file = File(lcovPath);
  if (!await file.exists()) {
    print('錯誤: 找不到 $lcovPath');
    print('請先運行測試並生成覆蓋率報告:');
    print('  flutter test --coverage');
    exit(1);
  }

  final content = await file.readAsString();
  final report = parseLcov(content);

  printReport(report);

  // 生成 markdown 報告
  final markdownReport = generateMarkdownReport(report);
  final markdownFile = File('coverage/REPORT.md');
  await markdownFile.writeAsString(markdownReport);

  print('\n詳細報告已生成: coverage/REPORT.md');
}

String generateMarkdownReport(CoverageReport report) {
  final buffer = StringBuffer();

  buffer.writeln('# 測試覆蓋率分析報告\n');
  buffer.writeln('**生成時間**: ${DateTime.now()}\n');
  buffer.writeln('## 總體概況\n');
  buffer.writeln('- **總體覆蓋率**: ${report.overallCoverage.toStringAsFixed(1)}%');
  buffer.writeln('- **已測試文件數**: ${report.files.length}');
  buffer.writeln('- **目標覆蓋率**: 80%\n');

  buffer.writeln('## 各模塊覆蓋率\n');
  buffer.writeln('| 模塊 | 覆蓋率 | 文件數 | 代碼行數 |');
  buffer.writeln('|------|--------|--------|----------|');

  final sortedDirs = report.filesByDirectory.entries.toList()
    ..sort((a, b) {
      final avgA = a.value.isEmpty ? 0 :
        a.value.map((f) => f.lineCoverage).fold<double>(0, (sum, cov) => sum + cov) / a.value.length;
      final avgB = b.value.isEmpty ? 0 :
        b.value.map((f) => f.lineCoverage).fold<double>(0, (sum, cov) => sum + cov) / b.value.length;
      return avgA.compareTo(avgB);
    });

  for (var entry in sortedDirs) {
    final files = entry.value;
    if (files.isEmpty) continue;

    final avgCoverage = files
        .map((f) => f.lineCoverage)
        .reduce((a, b) => a + b) / files.length;
    final totalLines = files.fold<int>(0, (sum, f) => sum + f.linesFound);
    final totalHit = files.fold<int>(0, (sum, f) => sum + f.linesHit);

    buffer.writeln('| ${entry.key} | ${avgCoverage.toStringAsFixed(1)}% | ${files.length} | $totalHit/$totalLines |');
  }

  buffer.writeln('\n## 覆蓋率最低的 10 個文件\n');
  buffer.writeln('| 排名 | 文件 | 覆蓋率 | 已覆蓋/總行數 | 未覆蓋行數 |');
  buffer.writeln('|------|------|--------|-------------|-----------|');

  for (var i = 0; i < report.lowestCoverage.length; i++) {
    final file = report.lowestCoverage[i];
    final name = file.filePath.replaceFirst('lib/', '');
    buffer.writeln('| ${i + 1} | $name | ${file.lineCoverage.toStringAsFixed(1)}% | ${file.linesHit}/${file.linesFound} | ${file.linesFound - file.linesHit} |');
  }

  buffer.writeln('\n## 改進計劃\n');

  final critical = report.lowestCoverage.where((f) => f.lineCoverage < 30).toList();
  final needsWork = report.lowestCoverage
      .where((f) => f.lineCoverage >= 30 && f.lineCoverage < 70).toList();

  buffer.writeln('### 優先級 1 - 緊急 (< 30% 覆蓋率)\n');
  for (var file in critical) {
    final name = file.filePath.replaceFirst('lib/', '');
    buffer.writeln('#### $name');
    buffer.writeln('- 當前覆蓋率: ${file.lineCoverage.toStringAsFixed(1)}%');
    buffer.writeln('- 未覆蓋: ${file.linesFound - file.linesHit} 行');
    buffer.writeln('- 建議: 為所有公共方法編寫單元測試\n');
  }

  buffer.writeln('### 優先級 2 - 重要 (30-70% 覆蓋率)\n');
  for (var file in needsWork.take(10)) {
    final name = file.filePath.replaceFirst('lib/', '');
    buffer.writeln('#### $name');
    buffer.writeln('- 當前覆蓋率: ${file.lineCoverage.toStringAsFixed(1)}%');
    buffer.writeln('- 未覆蓋: ${file.linesFound - file.linesHit} 行');
    buffer.writeln('- 建議: 補充邊界情況和錯誤處理的測試\n');
  }

  return buffer.toString();
}
