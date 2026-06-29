import 'dart:async';
import 'dart:convert';
import 'dart:js_interop';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

// WASM-Funktionen geben String zurück (kein Promise!)
@JS('init_simulator')
external String jsInitSimulator(
  int width,
  int height,
  bool wrapAround,
  String neighborhood,
  String rule,
  int historySize,
  bool randomStart,
  double density,
  double seed, // u32 auf JS-Seite — als double übergeben um BigInt zu vermeiden
);

@JS('tick')
external String jsTick();

@JS('step')
external String jsStep(int n);

@JS('reset')
external String jsReset();

@JS('toggle_cell')
external String jsToggleCell(int x, int y);

@JS('set_rule')
external String jsSetRule(String rule);

// GridState aus JSON parsen
class GridState {
  final int width;
  final int height;
  final List<int> cells;
  final int generation;
  final int aliveCount;

  const GridState({
    required this.width,
    required this.height,
    required this.cells,
    required this.generation,
    required this.aliveCount,
  });

  factory GridState.fromJson(Map<String, dynamic> json) {
    return GridState(
      width: json['width'] as int,
      height: json['height'] as int,
      cells: (json['cells'] as List).cast<int>(),
      generation: json['generation'] as int,
      aliveCount: json['aliveCount'] as int,
    );
  }
}

GridState _parse(String jsonStr) {
  return GridState.fromJson(jsonDecode(jsonStr) as Map<String, dynamic>);
}

@immutable
class SimulationState {
  final GridState? gridState;
  final bool isRunning;
  final int speed;
  final String currentRule;
  final bool isInitialized;
  final String? error;

  const SimulationState({
    this.gridState,
    this.isRunning = false,
    this.speed = 10,
    this.currentRule = 'game_of_life',
    this.isInitialized = false,
    this.error,
  });

  SimulationState copyWith({
    GridState? gridState,
    bool? isRunning,
    int? speed,
    String? currentRule,
    bool? isInitialized,
    Object? error = _sentinel,
  }) {
    return SimulationState(
      gridState: gridState ?? this.gridState,
      isRunning: isRunning ?? this.isRunning,
      speed: speed ?? this.speed,
      currentRule: currentRule ?? this.currentRule,
      isInitialized: isInitialized ?? this.isInitialized,
      error: error == _sentinel ? this.error : error as String?,
    );
  }
}

const Object _sentinel = Object();

class SimulationNotifier extends StateNotifier<SimulationState> {
  Timer? _timer;

  SimulationNotifier() : super(const SimulationState());

  Future<void> initialize({
    int width = 80,
    int height = 60,
    String rule = 'game_of_life',
    double density = 0.3,
  }) async {
    try {
      final json = jsInitSimulator(
        width,
        height,
        true,
        'moore',
        rule,
        100,
        true,
        density,
        42.0,
      );
      state = state.copyWith(
        gridState: _parse(json),
        isInitialized: true,
        currentRule: rule,
        error: null,
      );
    } catch (e) {
      state = state.copyWith(error: 'Initialisierung fehlgeschlagen: $e');
    }
  }

  void start() {
    if (!state.isInitialized || state.isRunning) return;
    _timer = Timer.periodic(
      Duration(milliseconds: (1000 / state.speed).round()),
      (_) => _doTick(),
    );
    state = state.copyWith(isRunning: true);
  }

  void pause() {
    _timer?.cancel();
    _timer = null;
    state = state.copyWith(isRunning: false);
  }

  void stepOnce() {
    if (!state.isInitialized) return;
    try {
      state = state.copyWith(gridState: _parse(jsTick()));
    } catch (e) {
      state = state.copyWith(error: '$e');
    }
  }

  void resetSim() {
    pause();
    if (!state.isInitialized) return;
    try {
      state = state.copyWith(gridState: _parse(jsReset()));
    } catch (e) {
      state = state.copyWith(error: '$e');
    }
  }

  void setSpeed(int speed) {
    final wasRunning = state.isRunning;
    if (wasRunning) pause();
    state = state.copyWith(speed: speed.clamp(1, 60));
    if (wasRunning) start();
  }

  void changeRule(String rule) {
    if (!state.isInitialized) return;
    try {
      state = state.copyWith(
        gridState: _parse(jsSetRule(rule)),
        currentRule: rule,
      );
    } catch (e) {
      state = state.copyWith(error: '$e');
    }
  }

  void tapCell(int x, int y) {
    if (!state.isInitialized) return;
    try {
      state = state.copyWith(gridState: _parse(jsToggleCell(x, y)));
    } catch (e) {
      state = state.copyWith(error: '$e');
    }
  }

  void _doTick() {
    if (!state.isInitialized) return;
    try {
      state = state.copyWith(gridState: _parse(jsTick()));
    } catch (e) {
      _timer?.cancel();
      state = state.copyWith(isRunning: false, error: '$e');
    }
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }
}

final simulationProvider =
    StateNotifierProvider<SimulationNotifier, SimulationState>(
      (ref) => SimulationNotifier(),
    );
