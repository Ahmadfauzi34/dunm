# Macro: EXTRACT_ANOMALY
Ini adalah skill grammar topologi kuantum untuk menggeser origin dari suatu objek yang menjadi minority (anomali simetri) ke (0,0) lalu melakukan cropping area (merubah global_width/height).

```yaml
id: MACRO:EXTRACT_ANOMALY
tier: 8
description: Cari anomali warna/simetri dan pindahkan ke pusat dimensi (0,0), hilangkan sisa dunia.
sequence:
  - axiom_type: ISOLATE_SYMMETRY_ANOMALY
    delta_x: 0.0
    delta_y: 0.0
    physics_tier: 5
  - axiom_type: CROP_TO_ORIGIN
    delta_x: 0.0
    delta_y: 0.0
    physics_tier: 7
```
