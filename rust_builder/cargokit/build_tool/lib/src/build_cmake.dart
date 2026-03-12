// This is copied from Cargokit (which is the official way to use it currently)
// Details: https://fzyzcjy.github.io/flutter_rust_bridge/manual/integrate/builtin

import 'dart:io';

import 'package:path/path.dart' as path;

import 'artifacts_provider.dart';
import 'builder.dart';
import 'environment.dart';
import 'options.dart';
import 'target.dart';

class BuildCMake {
  final CargokitUserOptions userOptions;

  BuildCMake({required this.userOptions});

  Future<void> build() async {
    final target = Target.forFlutterName(Environment.targetPlatform);
    if (target == null) {
      throw Exception("Unknown target platform: ${Environment.targetPlatform}");
    }

    final environment = BuildEnvironment.fromEnvironment(isAndroid: false);
    final provider =
        ArtifactProvider(environment: environment, userOptions: userOptions);
    final artifacts = await provider.getArtifacts([target]);

    final libs = artifacts[target];
    if (libs == null) {
      throw StateError('No artifacts found for target $target');
    }

    for (final lib in libs) {
      if (lib.type == ArtifactType.dylib) {
        File(lib.path)
            .copySync(path.join(Environment.outputDir, lib.finalFileName));
      }
    }
  }
}
