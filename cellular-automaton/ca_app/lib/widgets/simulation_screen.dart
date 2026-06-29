// simulation_screen.dart – Haupt-Screen der App
//
// Verbindet alle Widgets zu einem Screen.
// Lernkonzepte:
//   - ConsumerWidget (Riverpod)
//   - CustomPainter für das Gitter
//   - GestureDetector für Zell-Interaktion
//   - LayoutBuilder für responsive Größen

import 'dart:js_interop';
import 'dart:js_interop_unsafe';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../state/simulation_state.dart';

// =============================================================================
// SimulationScreen – Haupt-Screen
// =============================================================================

class SimulationScreen extends ConsumerStatefulWidget {
  const SimulationScreen({super.key});

  @override
  ConsumerState<SimulationScreen> createState() => _SimulationScreenState();
}

class _SimulationScreenState extends ConsumerState<SimulationScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _waitForWasmAndInit();
    });
  }

  void _waitForWasmAndInit() {
    if (!mounted) return;
    final ready = globalContext.getProperty('wasmReady'.toJS);
    if (ready?.dartify() == true) {
      ref
          .read(simulationProvider.notifier)
          .initialize(width: 80, height: 60, density: 0.3);
    } else {
      Future.delayed(const Duration(milliseconds: 100), _waitForWasmAndInit);
    }
  }

  @override
  Widget build(BuildContext context) {
    final simState = ref.watch(simulationProvider);

    return Scaffold(
      backgroundColor: const Color(0xFF0A0A0A),
      body: Column(
        children: [
          // Titel-Leiste
          _buildHeader(simState),

          // Gitter – nimmt den meisten Platz ein
          Expanded(
            child: simState.isInitialized && simState.gridState != null
                ? _buildCanvas(simState)
                : _buildLoadingScreen(simState),
          ),

          // Steuerung unten
          _buildControls(simState),
        ],
      ),
    );
  }

  // =========================================================================
  // Header
  // =========================================================================

  Widget _buildHeader(SimulationState simState) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: const Color(0xFF1A1A1A),
      child: Row(
        children: [
          const Text(
            '🧬 Cellular Automaton',
            style: TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.bold,
              color: Color(0xFF00FF46),
            ),
          ),
          const Spacer(),
          if (simState.gridState != null) ...[
            Text(
              'Gen: ${simState.gridState!.generation}',
              style: const TextStyle(color: Colors.white70, fontSize: 12),
            ),
            const SizedBox(width: 16),
            Text(
              'Alive: ${simState.gridState!.aliveCount}',
              style: const TextStyle(color: Colors.white70, fontSize: 12),
            ),
          ],
        ],
      ),
    );
  }

  // =========================================================================
  // Canvas
  // =========================================================================

  Widget _buildCanvas(SimulationState simState) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final gridState = simState.gridState!;

        // Zellgröße berechnen damit das Gitter den Platz ausfüllt
        final cellW = constraints.maxWidth / gridState.width;
        final cellH = constraints.maxHeight / gridState.height;
        final cellSize = cellW < cellH ? cellW : cellH;

        return GestureDetector(
          // Zelle beim Antippen togglen
          onTapDown: (details) {
            final x = (details.localPosition.dx / cellSize).floor();
            final y = (details.localPosition.dy / cellSize).floor();
            ref.read(simulationProvider.notifier).tapCell(x, y);
          },
          child: CustomPaint(
            size: Size(cellSize * gridState.width, cellSize * gridState.height),
            painter: GridPainter(gridState: gridState, cellSize: cellSize),
          ),
        );
      },
    );
  }

  // =========================================================================
  // Loading / Error
  // =========================================================================

  Widget _buildLoadingScreen(SimulationState simState) {
    if (simState.error != null) {
      return Center(
        child: Text(
          'Fehler: ${simState.error}',
          style: const TextStyle(color: Colors.red),
        ),
      );
    }
    return const Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          CircularProgressIndicator(color: Color(0xFF00FF46)),
          SizedBox(height: 16),
          Text('Simulator wird geladen...'),
        ],
      ),
    );
  }

  // =========================================================================
  // Controls
  // =========================================================================

  Widget _buildControls(SimulationState simState) {
    return Container(
      padding: const EdgeInsets.all(12),
      color: const Color(0xFF1A1A1A),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Buttons
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              // Play / Pause
              IconButton.filled(
                icon: Icon(simState.isRunning ? Icons.pause : Icons.play_arrow),
                onPressed: simState.isInitialized
                    ? () {
                        if (simState.isRunning) {
                          ref.read(simulationProvider.notifier).pause();
                        } else {
                          ref.read(simulationProvider.notifier).start();
                        }
                      }
                    : null,
                color: const Color(0xFF00FF46),
              ),
              const SizedBox(width: 8),

              // Step
              IconButton.outlined(
                icon: const Icon(Icons.skip_next),
                onPressed: simState.isInitialized
                    ? () => ref.read(simulationProvider.notifier).stepOnce()
                    : null,
                tooltip: 'Ein Schritt',
              ),
              const SizedBox(width: 8),

              // Reset
              IconButton.outlined(
                icon: const Icon(Icons.restart_alt),
                onPressed: simState.isInitialized
                    ? () => ref.read(simulationProvider.notifier).resetSim()
                    : null,
                tooltip: 'Zurücksetzen',
              ),
              const SizedBox(width: 16),

              // Regel-Auswahl
              _buildRuleSelector(simState),
            ],
          ),

          const SizedBox(height: 8),

          // Geschwindigkeit
          Row(
            children: [
              const Text('Speed:', style: TextStyle(fontSize: 12)),
              Expanded(
                child: Slider(
                  value: simState.speed.toDouble(),
                  min: 1,
                  max: 60,
                  divisions: 59,
                  label: '${simState.speed} fps',
                  activeColor: const Color(0xFF00FF46),
                  onChanged: (value) {
                    ref
                        .read(simulationProvider.notifier)
                        .setSpeed(value.round());
                  },
                ),
              ),
              Text(
                '${simState.speed} fps',
                style: const TextStyle(fontSize: 12, color: Colors.white70),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildRuleSelector(SimulationState simState) {
    final rules = [
      ('game_of_life', 'Game of Life'),
      ('high_life', 'HighLife'),
      ('maze', 'Maze'),
      ('seeds', 'Seeds'),
    ];

    return DropdownButton<String>(
      value: simState.currentRule,
      dropdownColor: const Color(0xFF2A2A2A),
      style: const TextStyle(color: Colors.white, fontSize: 13),
      underline: Container(height: 1, color: const Color(0xFF00FF46)),
      items: rules
          .map((r) => DropdownMenuItem(value: r.$1, child: Text(r.$2)))
          .toList(),
      onChanged: simState.isInitialized
          ? (value) {
              if (value != null) {
                ref.read(simulationProvider.notifier).changeRule(value);
              }
            }
          : null,
    );
  }
}

// =============================================================================
// GridPainter – Zeichnet das Gitter mit CustomPainter
// =============================================================================

/// CustomPainter rendert das Gitter direkt auf den Canvas.
///
/// Das ist sehr performant – Flutter ruft paint() nur auf wenn
/// sich der Zustand geändert hat (shouldRepaint gibt true zurück).
class GridPainter extends CustomPainter {
  final GridState gridState;
  final double cellSize;

  // Paint-Objekte einmal erstellen, nicht bei jedem Frame neu
  final Paint _alivePaint = Paint()..color = const Color(0xFF00FF46);
  final Paint _deadPaint = Paint()..color = const Color(0xFF0A0A0A);

  GridPainter({required this.gridState, required this.cellSize});

  @override
  void paint(Canvas canvas, Size size) {
    // Hintergrund füllen
    canvas.drawRect(Rect.fromLTWH(0, 0, size.width, size.height), _deadPaint);

    // Nur lebendige Zellen zeichnen (effizienter als alle Zellen)
    for (int i = 0; i < gridState.cells.length; i++) {
      if (gridState.cells[i] == 1) {
        final x = i % gridState.width;
        final y = i ~/ gridState.width;

        canvas.drawRect(
          Rect.fromLTWH(
            x * cellSize + 0.5, // 0.5 = kleiner Abstand zwischen Zellen
            y * cellSize + 0.5,
            cellSize - 0.5,
            cellSize - 0.5,
          ),
          _alivePaint,
        );
      }
    }
  }

  @override
  bool shouldRepaint(GridPainter oldDelegate) {
    // Neu zeichnen wenn sich der Grid-Zustand geändert hat
    return oldDelegate.gridState != gridState;
  }
}
