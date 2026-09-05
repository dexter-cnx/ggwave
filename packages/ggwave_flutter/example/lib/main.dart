import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:ggwave_rs_flutter/ggwave_rs_flutter.dart';
import 'package:permission_handler/permission_handler.dart';

void main() => runApp(const GgWaveValidationApp());

class GgWaveValidationApp extends StatelessWidget {
  const GgWaveValidationApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'ggwave Android Validation',
      theme: ThemeData(useMaterial3: true, colorSchemeSeed: Colors.indigo),
      home: const ValidationPage(),
    );
  }
}

enum ValidationRole { tx, rx }

class ValidationProfile {
  const ValidationProfile(this.label, this.protocol, this.frequencyHz);

  final String label;
  final GgWaveProtocol protocol;
  final double? frequencyHz;
}

const _profiles = <ValidationProfile>[
  ValidationProfile('Audible Fast', GgWaveProtocol.audibleFast, null),
  ValidationProfile('Ultrasonic 12 kHz', GgWaveProtocol.ultrasonicFast, 12000),
  ValidationProfile('Ultrasonic 15 kHz', GgWaveProtocol.ultrasonicFast, 15000),
  ValidationProfile('Ultrasonic 18 kHz', GgWaveProtocol.ultrasonicFast, 18000),
];

class ValidationPage extends StatefulWidget {
  const ValidationPage({super.key});

  @override
  State<ValidationPage> createState() => _ValidationPageState();
}

class _ValidationPageState extends State<ValidationPage> {
  final _transport = GgWaveFlutterTransport();
  final _payloadController = TextEditingController(text: 'GGWAVE-TEST-001');

  ValidationRole _role = ValidationRole.tx;
  ValidationProfile _profile = _profiles.first;
  bool _initialized = false;
  bool _listening = false;
  bool _busy = false;
  int _sent = 0;
  int _received = 0;
  String _lastReceived = '—';
  String _status = 'Not initialized';
  StreamSubscription<Uint8List>? _messageSub;

  void _log(String message) {
    debugPrint('[GGWAVE][UI] $message');
  }

  void _logError(String stage, Object error, StackTrace stackTrace) {
    debugPrint('[GGWAVE][ERROR][$stage] $error');
    debugPrintStack(label: '[GGWAVE][STACK][$stage]', stackTrace: stackTrace);
  }

  @override
  void initState() {
    super.initState();
    _log('validation page initialized');
    _messageSub = _transport.messages.listen(
      (bytes) {
        _log('received ${bytes.length} bytes');
        if (!mounted) return;
        setState(() {
          _received += 1;
          _lastReceived = utf8.decode(bytes, allowMalformed: true);
          _status = 'Received ${bytes.length} bytes';
        });
      },
      onError: (Object error, StackTrace stackTrace) {
        _logError('message-stream', error, stackTrace);
      },
    );
  }

  Future<void> _ensureInitialized() async {
    if (_initialized) return;
    _log('transport initialize: start');
    await _transport.initialize();
    _initialized = true;
    _log('transport initialize: success');
  }

  Future<void> _applyProfile() async {
    final hz = _profile.frequencyHz;
    if (hz != null) {
      _log('profile apply: ${_profile.label} ($hz Hz)');
      await _transport.setUltrasonicFrequency(hz);
      _log('profile apply: success');
    }
  }

  Future<void> _startListening() async {
    setState(() => _busy = true);
    _log('listen: start (${_profile.label})');
    try {
      final permission = await Permission.microphone.request();
      _log('microphone permission: $permission');
      if (!permission.isGranted) {
        if (!mounted) return;
        setState(() => _status = 'Microphone permission denied');
        return;
      }
      await _ensureInitialized();
      await _applyProfile();
      _log('native startListening: start');
      await _transport.startListening(protocol: _profile.protocol);
      _log('native startListening: success');
      if (!mounted) return;
      setState(() {
        _listening = true;
        _status = 'Listening: ${_profile.label}';
      });
    } catch (error, stackTrace) {
      _logError('listen', error, stackTrace);
      if (mounted) setState(() => _status = 'Listen error: $error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _stopListening() async {
    setState(() => _busy = true);
    _log('listen: stop');
    try {
      await _transport.stopListening();
      _log('listen: stopped');
      if (!mounted) return;
      setState(() {
        _listening = false;
        _status = 'Listening stopped';
      });
    } catch (error, stackTrace) {
      _logError('stop-listening', error, stackTrace);
      if (mounted) setState(() => _status = 'Stop error: $error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _send() async {
    final text = _payloadController.text.trim();
    if (text.isEmpty) {
      setState(() => _status = 'Enter a payload first');
      return;
    }

    setState(() => _busy = true);
    final bytes = Uint8List.fromList(utf8.encode(text));
    _log(
      'send: start bytes=${bytes.length} profile=${_profile.label} protocol=${_profile.protocol.id}',
    );
    try {
      await _ensureInitialized();
      await _applyProfile();

      final volume = _profile.protocol.isUltrasonic ? 85 : 60;
      _log('encode: start volume=$volume');
      final waveform = await _transport.encode(
        bytes,
        protocol: _profile.protocol,
        volume: volume,
      );
      _log('encode: success samples=${waveform.length}');

      _log('play: start');
      await _transport.play(waveform);
      _log('play: accepted by native layer');

      if (!mounted) return;
      setState(() {
        _sent += 1;
        _status = 'Sent ${bytes.length} bytes via ${_profile.label}';
      });
    } catch (error, stackTrace) {
      _logError('send', error, stackTrace);
      if (mounted) setState(() => _status = 'Send error: $error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _changeRole(ValidationRole role) async {
    if (_listening) {
      await _stopListening();
    }
    if (!mounted) return;
    setState(() {
      _role = role;
      _status = role == ValidationRole.tx ? 'TX mode' : 'RX mode';
    });
    _log('role changed: ${role.name}');
  }

  Future<void> _changeProfile(ValidationProfile? profile) async {
    if (profile == null) return;
    if (_listening) {
      await _stopListening();
    }
    if (!mounted) return;
    setState(() => _profile = profile);
    _log('profile changed: ${profile.label}');
  }

  @override
  void dispose() {
    _log('validation page disposed');
    _payloadController.dispose();
    unawaited(_messageSub?.cancel());
    if (_listening) {
      unawaited(_transport.stopListening());
    }
    unawaited(_transport.dispose());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final isTx = _role == ValidationRole.tx;

    return Scaffold(
      appBar: AppBar(title: const Text('ggwave Android Validation')),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            SegmentedButton<ValidationRole>(
              segments: const [
                ButtonSegment(
                  value: ValidationRole.tx,
                  label: Text('TX / Send'),
                  icon: Icon(Icons.volume_up),
                ),
                ButtonSegment(
                  value: ValidationRole.rx,
                  label: Text('RX / Listen'),
                  icon: Icon(Icons.mic),
                ),
              ],
              selected: {_role},
              onSelectionChanged: _busy
                  ? null
                  : (value) => _changeRole(value.first),
            ),
            const SizedBox(height: 16),
            DropdownButtonFormField<ValidationProfile>(
              initialValue: _profile,
              decoration: const InputDecoration(
                labelText: 'Protocol / Frequency',
                border: OutlineInputBorder(),
              ),
              items: _profiles
                  .map(
                    (profile) => DropdownMenuItem(
                      value: profile,
                      child: Text(profile.label),
                    ),
                  )
                  .toList(),
              onChanged: _busy ? null : _changeProfile,
            ),
            const SizedBox(height: 16),
            if (isTx) ...[
              TextField(
                controller: _payloadController,
                maxLength: 140,
                decoration: const InputDecoration(
                  labelText: 'Payload',
                  border: OutlineInputBorder(),
                ),
              ),
              FilledButton.icon(
                onPressed: _busy ? null : _send,
                icon: const Icon(Icons.send),
                label: Text(_busy ? 'Working…' : 'Send'),
              ),
            ] else ...[
              FilledButton.icon(
                onPressed: _busy
                    ? null
                    : (_listening ? _stopListening : _startListening),
                icon: Icon(_listening ? Icons.stop : Icons.mic),
                label: Text(_listening ? 'Stop Listening' : 'Start Listening'),
              ),
            ],
            const SizedBox(height: 24),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Status',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(_status),
                    const Divider(height: 24),
                    Text('Sent: $_sent'),
                    Text('Received: $_received'),
                    const SizedBox(height: 8),
                    const Text('Last received payload:'),
                    SelectableText(_lastReceived),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            const Text(
              'Use two physical Android devices. Set one to TX and the other to RX, select the same profile on both, then send/listen. Reverse roles for the second direction.',
            ),
          ],
        ),
      ),
    );
  }
}
