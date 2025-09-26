Berikut **jawaban untuk *open_questions.md*** berdasarkan **PRD v2.0** dan patch revisinya.
---

## 1) Performance Target Validation

**Jawaban:** *Tercapai dengan disiplin engineering dan fallback per-platform.*

* **Target utama**: P99 input processing ≤ **25 ms**, P95 startup ≤ **200 ms**, P95 render ≤ **33 ms** (≈30 FPS). Ini sejalan dengan PRD v2.0 dan sudah menggantikan target lama yang terlalu agresif. Implementasi: `crossterm` raw mode, event loop non-blocking, double-buffer render, pre-compose line diff, alokasi nol di hot-path. Windows `cmd.exe` berisiko; **Windows Terminal** & **PowerShell** aman sebagai baseline.  
* **Go/No-Go (Akhir Minggu-2)**:

  * **Go** jika: Linux/macOS memenuhi target; Windows Terminal P99 ≤ 30–35 ms (toleransi 10%); `cmd.exe` boleh fallback ke 45 ms.
  * **No-Go** → aktifkan “reduced effects mode” (kurangi repaint, matikan gradient/smooth cursor).
* **Bukti**: harness benchmark internal: simulasi 12 keystroke/detik & 30 keystroke/detik; time-to-paint dari event ke frame commit, plus leak check 30 menit sesi Endurance. 

**Keputusan:** Lanjut ke prototyping Minggu-1 s.d. 2; terapkan fallback & matriks terminal.

---

## 2) Team Resource Availability

**Jawaban:** *Cukup untuk rilis 16 minggu, dengan rencana cadangan jika tim lebih ramping.*

* **Konfigurasi ideal (PRD)**: 1.0 FTE Senior Rust, 1.0 FTE CLI, 0.5 FTE Writer, 0.5 FTE QA, 0.25 FTE PM → **on track** untuk 16 minggu. 
* **Plan B (jika kekurangan 0.5–1.0 FTE)**: pangkas fase konten → rilis **T1–T7 (70 level)** di v1.0; T8–T10 via v1.1–v1.2. Writer ditambah kontrak 2 minggu pada Week-7/8 untuk kalibrasi akhir.

**Keputusan:** Minta komitmen tertulis resource di Kickoff; jika <80% terpenuhi, aktifkan Plan B dan revisi scope per fase. 

---

## 3) Market Size & User Validation

**Jawaban:** *Cukup untuk proyek OSS + potensi Pro kecil; butuh validasi cepat.*

* Estimasi awal: dari populasi dev aktif global, segmen nyaman CLI dan tertarik latihan mengetik konservatif 0.3–0.6% → **~150k–300k** TAM; 1-tahun pertama **5–10k** unduhan realistis jika onboarding lancar.
* Validasi 4 minggu: landing page + waitlist, 2 kuesioner komunitas dev, uji instal lintas platform (npm, cargo, release binary). Sukses minimal: **≥500** daftar tunggu, **≥40%** menyatakan siap pakai CLI, **≥25%** minat konten kode (Rust/TS).

**Keputusan:** Jalan riset pasar Week-1→4, gunakan hasil untuk prioritas fitur Drill & distribusi. 

---

## 4) Security Audit Scope & Budget

**Jawaban:** *USD 15k cukup untuk audit fokus terminal/input; sediakan buffer +20%.*

* **Cakupan wajib**: filter escape sequence/CSI, validasi buffer & index, sandbox I/O (hanya dir profil/konfigurasi), fuzzing parser input, review path handling & atomic write, kebijakan telemetry opt-in.
* **Deliverables**: threat model, daftar temuan CVSS, PR rekomendasi, skrip fuzz + CI job.
* **Anggaran**: 15k cukup untuk 1.5–2 minggu tim kecil; tambah **contingency 3k** bila perlu cross-platform deep-dive. 

**Keputusan:** Kirim RFP di Week-2; jadwalkan audit Week-8/9. 

---

## 5) Level 100 Difficulty Calibration

**Jawaban:** *Definisi “hampir mustahil” sudah terukur; validasi pakai panel expert + bot.*

* PRD menetapkan: **Eff-WPM ≥130, Akurasi ≥99.5%, error severity ≤3, 3k–3.5k char/120 s, tanpa backspace di 20% akhir**. Ini *achievable* untuk <1% pengguna, setelah latihan intensif. 
* Validasi: (a) **Expert panel 10 orang**—target ≥2 lulus kriteria dalam 4–6 minggu latihan; (b) **bot konsisten** (input replay) memastikan skor deterministik.

**Keputusan:** Bekukan parameter Level-100; buka hanya penyesuaian minor (±5% TierWeight) pasca uji panel. 

---

## 6) Content Creation Resource Requirements

**Jawaban:** *0.5 FTE Writer cukup bila generator dinamis matang + kurasi ketat; kalau tidak, tambah 0.25 FTE atau geser 1–2 minggu.*

* Perkiraan beban: 100 level, 10 tier, 2 bahasa + QA → **120–160 jam** kurasi. Generator menanggung 60–70% beban; sisanya review & balancing.
* Mitigasi: library kata teknis terstruktur, histogram kelas karakter per tier, checklist profanitas & keterbacaan.

**Keputusan:** Pertahankan 0.5 FTE **dengan syarat** generator selesai Week-6; jika slip, tambah 0.25 FTE kontrak di Week-7/8.  

---

## 7) Distribution & Installation Strategy

**Jawaban:** *Tiga jalur resmi, dengan fallback jelas.*

* **npm/pnpm wrapper** (postinstall unduh binary → fallback `cargo build`), **cargo install** (crates.io), dan **GitHub Releases**. Alias `type` hanya jika tidak bentrok; Windows pakai shim `.cmd`. Non-ANSI/CI → auto `--no-splash`. E2E test untuk PATH & izin. 
* KPI onboarding: success-rate instal ≥ **95%** (30 mesin uji), waktu first-run P95 ≤ **5 s** termasuk unduh.

**Keputusan:** Lock strategi ini; implementasi packaging dimulai Week-9. 

---

## 8) Accessibility Compliance Scope

**Jawaban:** *Fokus pada yang feasible di CLI.*

* **Wajib**: kontras WCAG AA, info tidak bergantung warna, semua navigasi keyboard, opsi **mono** & **high-contrast**, ukuran teks dapat diatur.
* **Feasible**: kompatibilitas dasar screen reader (hindari spam repaint; gunakan teks murni saat `--no-splash`).
* **Tidak Feasible (MVP)**: dukungan IME kompleks & pembacaan konten live di semua pembaca layar.

**Keputusan:** Dokumentasikan batasan; QA aksesibilitas di Week-10 dengan checklist.  

---

## 9) Error Detection Algorithm Complexity

**Jawaban:** *Layak secara kinerja dengan jendela terbatas & diff inkremental.*

* **Desain**: deteksi Damerau-Levenshtein **sliding window N=5** + pointer indeks target → biaya per keystroke O(1) amortized.
* **Target**: P99 < 25 ms tercapai pada CPU 2-core modern; jika melampaui, matikan transposition di T1–T3 atau turunkan N ke 3.
* **Verifikasi**: microbench criterion + replay 10/30 keystroke/detik.  

**Keputusan:** Implement prototipe Week-3; pilih N berdasarkan hasil bench.

---

## 10) Telemetry Privacy & Legal Review

**Jawaban:** *Cakupan ringan, compliant, dan sepenuhnya opt-in.*

* **Kebijakan**: tanpa konten ketikan; hanya metrik agregat (latensi, platform, level, durasi). Data lokal transparan & dapat diekspor/hapus.
* **Langkah hukum**: review singkat GDPR (hak akses & penghapusan), lisensi OSS, dan teks persetujuan. Estimasi konsultasi **≤ USD 3k** sesuai PRD. 

**Keputusan:** Jadwalkan legal review Week-6 sebelum implementasi telemetry.

---

## Ringkasan Tenggat & Owner

* **Minggu-1/2 (IMMEDIATE):** #1 Performance prototyping (Tech Lead), #2 Resource commit (PM). 
* **Minggu-3/4:** #3 Market validation (PM), #4 Security RFP (PM+Sec), #7 Distribusi PoC (DevOps), #8 A11y scope (UX), #9 Algoritma bench (Sr Rust). 
* **Minggu-5/6:** #5 Kalibrasi L100 (UX+Writer), #6 Resource konten (PM), #10 Legal telemetry (Legal). 

---
