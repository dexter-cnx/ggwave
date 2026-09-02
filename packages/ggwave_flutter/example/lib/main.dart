import 'package:flutter/material.dart';
import 'package:ggwave_rs_flutter/ggwave_rs_flutter.dart';

void main() {
  // Importing the public package here intentionally keeps this example as a
  // consumer compile check. Native codec setup is exercised by CI after FRB
  // generation; hardware microphone/speaker validation is tracked separately.
  runApp(
    const MaterialApp(
      home: Scaffold(
        body: Center(
          child: Text(
            'ggwave_rs_flutter example — see README for native setup and microphone permissions',
          ),
        ),
      ),
    ),
  );
}
