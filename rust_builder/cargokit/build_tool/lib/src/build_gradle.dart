// This is copied from Cargokit (which is the official way to use it currently)
// Details: https://fzyzcjy.github.io/flutter_rust_bridge/manual/integrate/builtin

import 'dart:io';

import 'package:path/path.dart' as path;

import 'artifacts_provider.dart';
import 'builder.dart';
import 'environment.dart';
import 'options.dart';
import 'target.dart';

class BuildGradle {
  BuildGradle({required this.userOptions});

  final CargokitUserOptions userOptions;

  Future<void> build() async {
    final targets = Environment.targetPlatforms.map((arch) {
      final target = Target.forFlutterName(arch);
      if (target == null) {
        throw Exception(
            "Unknown Android target or platform: $arch");
      }
      return target;
    }).toList();

    final environment = BuildEnvironment.fromEnvironment(isAndroid: true);
    final provider =
        ArtifactProvider(environment: environment, userOptions: userOptions);
    final artifacts = await provider.getArtifacts(targets);

    for (final target in targets) {
      final libs = artifacts[target];
      if (libs == null) {
        throw StateError(
            'No artifacts found for target $target');
      }
      final androidAbi = target.android;
      if (androidAbi == null) {
        throw StateError(
            'Target $target has no Android ABI mapping');
      }
      final outputDir = path.join(Environment.outputDir, androidAbi);
      Directory(outputDir).createSync(recursive: true);

      for (final lib in libs) {
        if (lib.type == ArtifactType.dylib) {
          File(lib.path).copySync(path.join(outputDir, lib.finalFileName));
        }
      }
    }
  }
}
