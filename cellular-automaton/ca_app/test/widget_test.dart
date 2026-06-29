import 'package:ca_app/main.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('App startet ohne Fehler', (WidgetTester tester) async {
    await tester.pumpWidget(const CellularAutomatonApp());
    expect(find.byType(CellularAutomatonApp), findsOneWidget);
  });
}
