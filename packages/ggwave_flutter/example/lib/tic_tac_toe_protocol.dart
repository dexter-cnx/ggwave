import 'dart:convert';
import 'dart:typed_data';

const ticTacToeWireVersion = 'TT1';

enum TicTacToeMessageType { hello, ready, state, reset }

class TicTacToeMessage {
  const TicTacToeMessage({
    required this.type,
    required this.sessionId,
    required this.senderId,
    required this.sequence,
    required this.board,
  });

  final TicTacToeMessageType type;
  final String sessionId;
  final String senderId;
  final int sequence;
  final String board;

  Uint8List encode() {
    final typeCode = switch (type) {
      TicTacToeMessageType.hello => 'H',
      TicTacToeMessageType.ready => 'R',
      TicTacToeMessageType.state => 'S',
      TicTacToeMessageType.reset => 'X',
    };
    return Uint8List.fromList(
      utf8.encode(
        '$ticTacToeWireVersion|$typeCode|$sessionId|$senderId|$sequence|$board',
      ),
    );
  }

  static TicTacToeMessage? tryDecode(Uint8List bytes) {
    final parts = utf8.decode(bytes, allowMalformed: false).split('|');
    if (parts.length != 6 || parts[0] != ticTacToeWireVersion) return null;

    final type = switch (parts[1]) {
      'H' => TicTacToeMessageType.hello,
      'R' => TicTacToeMessageType.ready,
      'S' => TicTacToeMessageType.state,
      'X' => TicTacToeMessageType.reset,
      _ => null,
    };
    final sequence = int.tryParse(parts[4]);
    if (type == null || sequence == null || sequence < 0) return null;
    if (!_validId(parts[2]) || !_validId(parts[3])) return null;
    if (!_validBoard(parts[5])) return null;

    return TicTacToeMessage(
      type: type,
      sessionId: parts[2],
      senderId: parts[3],
      sequence: sequence,
      board: parts[5],
    );
  }

  static bool _validId(String value) =>
      value.isNotEmpty && value.length <= 12 && !value.contains('|');

  static bool _validBoard(String value) =>
      value.length == 9 && value.split('').every((c) => c == '-' || c == 'X' || c == 'O');
}

String winnerForBoard(String board) {
  const wins = <List<int>>[
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
  ];
  for (final line in wins) {
    final a = board[line[0]];
    if (a != '-' && a == board[line[1]] && a == board[line[2]]) return a;
  }
  return board.contains('-') ? '' : 'draw';
}
