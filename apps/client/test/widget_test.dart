import 'package:flutter_test/flutter_test.dart';
import 'package:client/main.dart';

void main() {
  testWidgets('App renders login page', (WidgetTester tester) async {
    await tester.pumpWidget(const TrustRAGApp());
    expect(find.text('TrustRAG'), findsOneWidget);
  });
}
