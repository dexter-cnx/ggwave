import 'dart:io';

import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';
import 'package:native_toolchain_rust/native_toolchain_rust.dart';

void main(List<String> args) async {
  await build(args, (input, output) async {
    await const RustBuilder(
      assetName: 'lib/src/rust/frb_generated.dart',
    ).run(input: input, output: output);

    if (input.config.buildCodeAssets &&
        input.config.code.targetOS == OS.android) {
      _bundleAndroidCxxRuntime(input, output);
    }
  });
}

void _bundleAndroidCxxRuntime(BuildInput input, BuildOutputBuilder output) {
  final codeConfig = input.config.code;
  final compiler = codeConfig.cCompiler?.compiler;
  if (compiler == null) {
    throw StateError(
      'Android C compiler metadata is unavailable; cannot locate libc++_shared.so.',
    );
  }

  final toolchainRoot = File.fromUri(compiler).parent.parent;
  final targetTriple = switch (codeConfig.targetArchitecture.name) {
    'arm' => 'arm-linux-androideabi',
    'arm64' => 'aarch64-linux-android',
    'x64' => 'x86_64-linux-android',
    final architecture => throw UnsupportedError(
      'Unsupported Android architecture for libc++_shared.so: $architecture',
    ),
  };

  final runtime = File(
    '${toolchainRoot.path}/sysroot/usr/lib/$targetTriple/libc++_shared.so',
  );
  if (!runtime.existsSync()) {
    throw StateError('Android C++ runtime not found at ${runtime.path}.');
  }

  output.dependencies.add(runtime.uri);
  output.assets.code.add(
    CodeAsset(
      package: input.packageName,
      name: 'libc++_shared.so',
      file: runtime.uri,
      linkMode: DynamicLoadingBundled(),
    ),
  );
}
