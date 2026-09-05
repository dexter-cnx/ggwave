import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import '../lib/tic_tac_toe_protocol.dart';

void main() {
  test('round-trips a compact state message', () {
    const message = TicTacToeMessage(
      type: TicTacToeMessageType.state,
      sessionId: 'A1B2C3',
      senderId: 'D4E5F6',
      sequence: 7,
      board: 'X-O-X----',
    );

    final encoded = message.encode();
    final decoded = TicTacToeMessage.tryDecode(encoded);

    expect(encoded.length, lessThan(140));
    expect(decoded, isNotNull);
    expect(decoded!.type, TicTacToeMessageType.state);
    expect(decoded.sessionId, 'A1B2C3');
    expect(decoded.senderId, 'D4E5F6');
    expect(decoded.sequence, 7);
    expect(decoded.board, 'X-O-X----');
  });

  test('rejects malformed payloads', () {
    expect(
      TicTacToeMessage.tryDecode(Uint8List.fromList('TT0|S|A|B|1|---------'.codeUnits)),
      isNull,
    );
    expect(
      TicTacToeMessage.tryDecode(Uint8List.fromList('TT1|S|A|B|1|bad'.codeUnits)),
      isNull,
    );
  });

  test('detects wins and draws', () {
    expect(winnerForBoard('XXXOO----'), 'X');
    expect(winnerForBoard('XO-XO-X--'), 'X');
    expect(winnerForBoard('XOXOOXXXO'), 'draw');
    expect(winnerForBoard('XO-------'), '');
  });
}
