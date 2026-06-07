#!/usr/bin/env python3
import sys
import re
from collections import defaultdict

def analyze_decay(log_content):
    """
    Reads the RRM simulation log and traces the probability of each Axiom Path
    to detect where it starts decaying and where it gets pruned (dies).
    """
    # Lebih luwes (Robust Regex) - mendeteksi berbagai variasi log MCTS
    # Matches: [Depth 0] Axioms: ["ROOT_START"] | Pragmatic: -1500.00 | Epistemic: 0.00 | Prob: 1.0000 | Dim: 11x11
    # ATAU format MCTS alternatif lainnya jika kelak visualizer berubah.
    pattern = re.compile(r"\[Depth\s+(\d+)\]\s+Axioms:\s+\[(.*?)\]\s+\|\s+Pragmatic:\s+([-\d.]+)(?:\s+\|\s+Epistemic:\s+([-\d.]+))?\s+\|\s+Prob:\s+([\d.]+)")

    # Data structure: path -> list of (depth, prob, pragmatic_error, epistemic_val)
    history = defaultdict(list)
    max_depth_seen = 0
    ground_states_found = 0

    for line in log_content.splitlines():
        match = pattern.search(line)
        if match:
            depth = int(match.group(1))
            raw_path = match.group(2)
            # Cleanup raw path, e.g., '"ROOT_START", "SCALE_AND_FILL_2_3x3"' -> 'ROOT_START -> SCALE_AND_FILL_2_3x3'
            clean_path = raw_path.replace('"', '').replace(', ', ' -> ')
            prag = float(match.group(3))

            # Toleransi jika format log tidak mencetak Epistemic
            epistemic = float(match.group(4)) if match.group(4) else 0.0
            prob = float(match.group(5))

            history[clean_path].append((depth, prob, prag, epistemic))
            if depth > max_depth_seen:
                max_depth_seen = depth

    print("=========================================================")
    print(" 🕵️‍♂️  Axiom Decay & Pruning Tracker (Meta-Upgraded)")
    print("=========================================================")
    print(f"Total Unique Paths Evaluated : {len(history)}")
    print(f"Max Depth Reached            : {max_depth_seen}")
    print("---------------------------------------------------------\n")

    # Analyze each path
    # Mengurutkan berdasarkan kedalaman tercapai, lalu jumlah elemen dalam path
    for path, records in sorted(history.items(), key=lambda x: (max(r[0] for r in x[1]), len(x[1]))):
        # records is a list of (depth, prob, prag, epistemic)
        records = sorted(records, key=lambda x: x[0]) # Sort by depth

        print(f"🌿 Path: {path}")

        previous_prob = None
        for i, (depth, prob, prag, epistemic) in enumerate(records):
            status = ""

            # Detect Decay (Drop in probability)
            if previous_prob is not None and prob < previous_prob:
                drop = previous_prob - prob
                if drop >= 0.2:
                    status = f"⚠️ SEVERE DECAY (-{drop:.2f})"
                else:
                    status = f"📉 Fading (-{drop:.2f})"

            # Detect Death/Pruning (Prob <= 0.01)
            if prob <= 0.01:
                status += " 💀 PRUNED/DEAD (Prob too low)"
            # Robust Ground State check: Error is very close to 0.0 (prevent float drift errors)
            elif abs(prag) <= 1e-5 and depth > 0:
                status += " 🏆 GROUND STATE FOUND!"
                ground_states_found += 1

            print(f"   ├─ Depth {depth} | Prob: {prob:.4f} | Pragmatic: {prag:>8.2f} | Epis: {epistemic:>5.2f} {status}")
            previous_prob = prob

        # If the path didn't reach the max_depth and isn't a ground state, it was structurally abandoned
        last_depth = records[-1][0]
        last_prob = records[-1][1]
        last_prag = records[-1][2]

        if last_depth < max_depth_seen and last_prob > 0.01 and abs(last_prag) > 1e-5:
             print(f"   └─ 🪦 ABANDONED at Depth {last_depth} (Did not spawn children or halted)")
        elif abs(last_prag) <= 1e-5 and last_depth > 0:
             print("   └─ ✨ SOLUTION BRANCH")
        else:
             print("   └─ End of trace")
        print()

    print("=========================================================")
    print(f" 🏁 SUMMARY: Found {ground_states_found} distinct Ground State matches in the MCTS tree.")
    print("=========================================================")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        if sys.argv[1] in ["-h", "--help"]:
            print("Usage: ./axiom_decay_tracker.py [log_file.txt]\nIf no file is provided, it reads from stdin.")
            sys.exit(0)
        with open(sys.argv[1], 'r') as f:
            analyze_decay(f.read())
    else:
        # Read from stdin
        analyze_decay(sys.stdin.read())
