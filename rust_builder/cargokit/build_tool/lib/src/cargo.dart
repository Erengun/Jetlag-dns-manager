// This is copied from Cargokit (which is the official way to use it currently)
// Details: https://fzyzcjy.github.io/flutter_rust_bridge/manual/integrate/builtin

import 'dart:io';

import 'package:path/path.dart' as path;
import 'package:toml/toml.dart';

class ManifestException implements Exception {
  ManifestException(this.message, {required this.fileName});

  final String? fileName;
  final String message;

  @override
  String toString() {
    if (fileName != null) {
      return 'Failed to parse package manifest at $fileName: $message';
    } else {
      return 'Failed to parse package manifest: $message';
    }
  }
}

class CrateInfo {
  CrateInfo({required this.packageName});

  final String packageName;

  static CrateInfo parseManifest(String manifest, {final String? fileName}) {
    TomlDocument toml;
    try {
      toml = TomlDocument.parse(manifest);
    } on TomlParserException catch (e) {
      throw ManifestException(e.toString(), fileName: fileName);
    }
    final tomlMap = toml.toMap();
    final package = tomlMap['package'];
    if (package == null || package is! Map) {
      throw ManifestException('Missing or invalid package section', fileName: fileName);
    }
    final name = package['name'];
    if (name == null || name is! String) {
      throw ManifestException('Missing or invalid package name', fileName: fileName);
    }
    return CrateInfo(packageName: name);
  }

  static CrateInfo load(String manifestDir) {
    final manifestFile = File(path.join(manifestDir, 'Cargo.toml'));
    final manifest = manifestFile.readAsStringSync();
    return parseManifest(manifest, fileName: manifestFile.path);
  }
}
