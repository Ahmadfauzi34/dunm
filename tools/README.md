# RRM Rust Utility Tools

Kumpulan alat diagnostik untuk melakukan inspeksi mendalam (deep introspection) terhadap *MCTS (Monte Carlo Tree Search)* dan *Fractal Arena* RRM.

## 1. Axiom Decay & Pruning Tracker (`axiom_decay_tracker.py`)

Skrip ini menganalisis file *log* eksekusi Rust untuk melacak riwayat setiap *Axiom Path*. Alat ini berfungsi untuk menjawab pertanyaan: **"Kapan sebuah aksioma mulai hancur (pruned) dan di kedalaman mana ia mulai memudar (decay) probabilitasnya?"**

### Cara Penggunaan:

Anda dapat mem-pipe (mengalirkan) *output* langsung dari eksekusi `cargo run`, atau menyimpannya ke dalam file log terlebih dahulu.

**Opsi 1 (Disarankan): Menyimpan ke Log dan Menganalisis**
```bash
# 1. Jalankan simulasi dan simpan log-nya
cargo run --release --bin rrm_rust -- --test_arc --arc_id 22233c11.json > sim.log

# 2. Jalankan tracker
./tools/axiom_decay_tracker.py sim.log
```

**Opsi 2: Pipe Langsung (Real-time Analysis di akhir eksekusi)**
```bash
cargo run --release --bin rrm_rust -- --test_arc --arc_id 22233c11.json | ./tools/axiom_decay_tracker.py
```

### Membaca Hasil Output:
- **📉 Fading (-X.XX):** Probabilitas (Amplitudo) aksioma tersebut menurun karena *Pragmatic Error* atau *Epistemic Value*-nya memburuk.
- **⚠️ SEVERE DECAY:** Probabilitas turun sangat drastis (lebih dari 0.2) dalam satu iterasi *Quantum Tunneling/Ghost Amplitude*.
- **💀 PRUNED/DEAD:** Probabilitas jatuh di bawah ambang batas (biasanya `< 0.01`). Node ini dihapus (di-prune) dari *Fractal Arena*.
- **🏆 GROUND STATE FOUND:** Node mencapai *Pragmatic Error == 0.0* pada *Microscopic Phase* (Kedalaman >= 1). Ini adalah tebakan sempurna.
