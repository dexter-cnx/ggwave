import 'dart:async';
import 'dart:math';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:ggwave_rs_flutter/ggwave_rs_flutter.dart';
import 'package:permission_handler/permission_handler.dart';

import 'tic_tac_toe_protocol.dart';

void main() => runApp(const TicTacToeDemoApp());

class TicTacToeDemoApp extends StatelessWidget {
  const TicTacToeDemoApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'ggwave Tic-Tac-Toe',
      theme: ThemeData(useMaterial3: true, colorSchemeSeed: Colors.indigo),
      home: const TicTacToePage(),
    );
  }
}

class TicTacToePage extends StatefulWidget {
  const TicTacToePage({super.key});

  @override
  State<TicTacToePage> createState() => _TicTacToePageState();
}

class _TicTacToePageState extends State<TicTacToePage> {
  final _transport = GgWaveFlutterTransport();
  final _random = Random.secure();

  StreamSubscription<Uint8List>? _messageSub;
  bool _initialized = false;
  bool _listening = false;
  bool _busy = false;
  bool _connected = false;
  String _board = '---------';
  String? _sessionId;
  late final String _senderId = _hexId(4);
  String? _peerId;
  String? _myMark;
  int _sequence = 0;
  final Map<String, int> _lastSequenceBySender = {};
  String _status = 'Choose Host or Join on two nearby devices.';

  @override
  void initState() {
    super.initState();
    _messageSub = _transport.messages.listen((bytes) {
      unawaited(_handleMessage(bytes));
    });
  }

  String _hexId(int bytes) {
    return List.generate(
      bytes,
      (_) => _random.nextInt(256).toRadixString(16).padLeft(2, '0'),
    ).join().toUpperCase();
  }

  Future<void> _ensureListening() async {
    if (_listening) return;
    final permission = await Permission.microphone.request();
    if (!permission.isGranted) {
      throw StateError('Microphone permission denied');
    }
    if (!_initialized) {
      await _transport.initialize();
      _initialized = true;
    }
    await _transport.startListening(protocol: GgWaveProtocol.audibleFast);
    _listening = true;
  }

  Future<void> _host() async {
    setState(() => _busy = true);
    try {
      await _ensureListening();
      _sessionId = _hexId(3);
      _peerId = null;
      _myMark = 'X';
      _connected = false;
      _board = '---------';
      _sequence = 0;
      _lastSequenceBySender.clear();
      await _send(TicTacToeMessageType.hello);
      if (!mounted) return;
      setState(() => _status = 'Game $_sessionId announced. Waiting for O…');
    } catch (error) {
      if (mounted) setState(() => _status = 'Host error: $error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _join() async {
    setState(() => _busy = true);
    try {
      _sessionId = null;
      _peerId = null;
      _myMark = null;
      _connected = false;
      _board = '---------';
      _sequence = 0;
      _lastSequenceBySender.clear();
      await _ensureListening();
      if (!mounted) return;
      setState(() => _status = 'Listening for a nearby Tic-Tac-Toe host…');
    } catch (error) {
      if (mounted) setState(() => _status = 'Join error: $error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _send(TicTacToeMessageType type) async {
    final session = _sessionId;
    if (session == null) return;
    final message = TicTacToeMessage(
      type: type,
      sessionId: session,
      senderId: _senderId,
      sequence: ++_sequence,
      board: _board,
    );
    final waveform = await _transport.encode(
      message.encode(),
      protocol: GgWaveProtocol.audibleFast,
      volume: 65,
    );
    await _transport.play(waveform);
  }

  Future<void> _handleMessage(Uint8List bytes) async {
    TicTacToeMessage? message;
    try {
      message = TicTacToeMessage.tryDecode(bytes);
    } on FormatException {
      return;
    }
    if (message == null || message.senderId == _senderId || !mounted) return;

    final previousSequence = _lastSequenceBySender[message.senderId] ?? -1;
    if (message.sequence <= previousSequence) return;

    if (message.type == TicTacToeMessageType.hello && _myMark == null) {
      _lastSequenceBySender[message.senderId] = message.sequence;
      setState(() {
        _sessionId = message!.sessionId;
        _peerId = message.senderId;
        _myMark = 'O';
        _board = message.board;
        _connected = true;
        _status = 'Joined ${message.sessionId} as O. X moves first.';
      });
      try {
        await _send(TicTacToeMessageType.ready);
      } catch (error) {
        if (mounted) setState(() => _status = 'Ready send error: $error');
      }
      return;
    }

    if (message.sessionId != _sessionId) return;
    if (_peerId != null && message.senderId != _peerId) return;
    _lastSequenceBySender[message.senderId] = message.sequence;

    if (message.type == TicTacToeMessageType.ready && _myMark == 'X') {
      setState(() {
        _peerId = message!.senderId;
        _connected = true;
        _status = 'O joined. Your turn (X).';
      });
      return;
    }

    if (message.type == TicTacToeMessageType.state) {
      final remoteMark = _myMark == 'X' ? 'O' : 'X';
      if (!_isValidRemoteTransition(_board, message.board, remoteMark)) return;
      setState(() {
        _board = message!.board;
        _status = _statusForBoard();
      });
      return;
    }

    if (message.type == TicTacToeMessageType.reset) {
      setState(() {
        _board = '---------';
        _connected = true;
        _status = _myMark == 'X' ? 'New round. Your turn (X).' : 'New round. Waiting for X.';
      });
    }
  }

  bool _isValidRemoteTransition(String before, String after, String remoteMark) {
    if (before.length != 9 || after.length != 9) return false;
    var changes = 0;
    for (var i = 0; i < 9; i++) {
      if (before[i] == after[i]) continue;
      if (before[i] != '-' || after[i] != remoteMark) return false;
      changes++;
    }
    return changes == 1;
  }

  String get _turnMark {
    final x = 'X'.allMatches(_board).length;
    final o = 'O'.allMatches(_board).length;
    return x == o ? 'X' : 'O';
  }

  String _statusForBoard() {
    final winner = winnerForBoard(_board);
    if (winner == 'draw') return 'Draw.';
    if (winner.isNotEmpty) return '$winner wins.';
    return _turnMark == _myMark ? 'Your turn ($_myMark).' : 'Waiting for $_turnMark…';
  }

  Future<void> _tapCell(int index) async {
    if (_busy || !_connected || _myMark == null) return;
    if (winnerForBoard(_board).isNotEmpty || _turnMark != _myMark || _board[index] != '-') return;

    setState(() {
      _busy = true;
      _board = _board.substring(0, index) + _myMark! + _board.substring(index + 1);
      _status = _statusForBoard();
    });
    try {
      await _send(TicTacToeMessageType.state);
    } catch (error) {
      if (mounted) setState(() => _status = 'Move send error: $error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _resetRound() async {
    if (!_connected) return;
    setState(() {
      _busy = true;
      _board = '---------';
    });
    try {
      await _send(TicTacToeMessageType.reset);
      if (mounted) setState(() => _status = _myMark == 'X' ? 'New round. Your turn (X).' : 'New round. Waiting for X.');
    } catch (error) {
      if (mounted) setState(() => _status = 'Reset send error: $error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  void dispose() {
    unawaited(_messageSub?.cancel());
    if (_listening) unawaited(_transport.stopListening());
    unawaited(_transport.dispose());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final winner = winnerForBoard(_board);
    final canReset = _connected && !_busy;

    return Scaffold(
      appBar: AppBar(title: const Text('ggwave Tic-Tac-Toe')),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(20),
          children: [
            const Text(
              '1 ↔ 1 acoustic reference demo',
              style: TextStyle(fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 8),
            Text(_status),
            const SizedBox(height: 16),
            Wrap(
              spacing: 12,
              runSpacing: 12,
              children: [
                FilledButton.icon(
                  onPressed: _busy ? null : _host,
                  icon: const Icon(Icons.campaign),
                  label: const Text('Host as X'),
                ),
                OutlinedButton.icon(
                  onPressed: _busy ? null : _join,
                  icon: const Icon(Icons.hearing),
                  label: const Text('Join as O'),
                ),
              ],
            ),
            const SizedBox(height: 20),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  children: [
                    Text('Session: ${_sessionId ?? '—'}   You: ${_myMark ?? '—'}'),
                    const SizedBox(height: 16),
                    AspectRatio(
                      aspectRatio: 1,
                      child: GridView.builder(
                        physics: const NeverScrollableScrollPhysics(),
                        gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                          crossAxisCount: 3,
                          mainAxisSpacing: 8,
                          crossAxisSpacing: 8,
                        ),
                        itemCount: 9,
                        itemBuilder: (context, index) {
                          final value = _board[index];
                          final enabled = _connected &&
                              !_busy &&
                              winner.isEmpty &&
                              _turnMark == _myMark &&
                              value == '-';
                          return FilledButton.tonal(
                            onPressed: enabled ? () => _tapCell(index) : null,
                            child: Text(
                              value == '-' ? '' : value,
                              style: Theme.of(context).textTheme.displaySmall,
                            ),
                          );
                        },
                      ),
                    ),
                    const SizedBox(height: 16),
                    OutlinedButton.icon(
                      onPressed: canReset ? _resetRound : null,
                      icon: const Icon(Icons.refresh),
                      label: const Text('New round'),
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            const Text(
              'Run this entrypoint on two physical devices. The demo uses Audible Fast and sends tiny versioned state packets; all game-specific protocol code stays inside the example.',
            ),
          ],
        ),
      ),
    );
  }
}
