// main.dart – Einstiegspunkt der Flutter App
//
// Lernkonzepte:
//   - Riverpod ProviderScope
//   - Flutter Web Initialisierung
//   - MaterialApp Setup

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'widgets/simulation_screen.dart';

void main() {
  runApp(const ProviderScope(child: CellularAutomatonApp()));
}

class CellularAutomatonApp extends StatelessWidget {
  const CellularAutomatonApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Cellular Automaton',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark(useMaterial3: true).copyWith(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF00FF46),
          brightness: Brightness.dark,
        ),
        scaffoldBackgroundColor: const Color(0xFF0A0A0A),
      ),
      home: const SimulationScreen(),
    );
  }
}
