// This is copied from Cargokit (which is the official way to use it currently)
// Details: https://fzyzcjy.github.io/flutter_rust_bridge/manual/integrate/builtin

import 'package:build_tool/build_tool.dart' as build_tool;

Future<void> main(List<String> arguments) async {
  await build_tool.runMain(arguments);
}
