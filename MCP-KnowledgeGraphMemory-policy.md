# MCP-Knowledge-Graph-Memory — Best Practices

> **Tujuan dokumen**: Menyediakan template *best practices* untuk konfigurasi, operasional, dan audit **MCP Knowledge Graph Memory**. **Jangan** isi nama proyek/ruang hingga agen menjalankan `/init`. Semua nilai diapit `{{…}}` adalah **placeholder**.

---

## 0) Metadata & Scope

* **Namespace utama**: `{{NAMESPACE}}`
* **Owner (teknis)**: `{{OWNER_NAME}}` — `{{OWNER_CONTACT}}`
* **Lokasi penyimpanan lokal**: `{{MEMORY_PATH}}` (contoh: `~/.mcp-memory/{{NAMESPACE}}/graph.json`)
* **Kebijakan privasi**: `{{PRIVACY_POLICY_REF}}`
* **Tanggal mulai**: `{{START_DATE}}`

> **Catatan**: Jika belum ada nilai, biarkan kosong. Agen penginisiasi mengisi setelah `/init`.

---

## 1) Prinsip Desain (tanpa kompromi)

1. **Kanonisasi entitas** — setiap entitas wajib punya:

   * `type` (contoh: `Person|Project|Law|Service|Module|Case`)
   * `id` **stabil** (slug, kebal spasi/kapital) → `{{ID_SCHEME}}`
   * `labels` untuk alias/sinonim (opsional tapi disarankan)
2. **Relasi terarah & berperan** — gunakan predikat baku:

   * `MEMBER_OF`, `OWNS`, `DEPENDS_ON`, `GOVERNED_BY`, `CITES`, `FOCUSES_ON`, `PART_OF`, `DERIVES_FROM`
3. **Observasi berwaktu** — jangan overwrite. Tambahkan `observed_at`, `valid_from`, `valid_to`, `source`, `who`, `confidence`.
4. **Kontekstual & dapat diaudit** — simpan `source_url`, `jurisdiction`, `version`, `hash` untuk fakta normatif/kode.

> **Placeholder aturan tambahan**: `{{ADDITIONAL_PRINCIPLES}}`

---

## 2) Isolasi & Namespacing

* **Ruang memori** dipisah per tujuan: `{{NAMESPACE}}`, `{{NAMESPACE_SECONDARY}}`, dst.
* Setiap ruang memiliki berkas/DB terpisah: `{{MEMORY_PATH}}`, `{{SECONDARY_MEMORY_PATH}}`.
* **PII/sensitif**: gunakan tag `sensitive:true` + opsi enkripsi/penyimpanan terpisah `{{SECURE_STORE_REF}}`.

**Contoh `.mcp.json`** (gunakan placeholder; agen akan mengisi):

```json
{
  "mcpServers": {
    "memory-{{NAMESPACE}}": {
      "command": "npx",
      "args": [
        "@modelcontextprotocol/server-memory",
        "--memory-path",
        "{{MEMORY_PATH}}"
      ]
    }
  }
}
```

---

## 3) Kriteria “Layak Ingat” (Whitelist) vs “Abaikan” (Blacklist)

**Whitelist (WAJIB masuk memori):**

* Keputusan final (`RFD`, `ADR`), definisi istilah kanonik, kontrak antarmodul
* Sumber primer (tautan referensi hukum/dokumen resmi), versi rilis, batasan sistem

**Blacklist (JANGAN disimpan):**

* Log sementara/debug sekali pakai, opini spekulatif tanpa sumber, kredensial rahasia (simpan hanya pointer)

> **Regulasi khusus/goal proyek**: `{{MEMORY_POLICY_LINK}}`

---

## 4) Skema Minimal Node/Edge/Observation

**Entity (node)**

```json
{
  "type": "{{ENTITY_TYPE}}",
  "id": "{{ENTITY_ID}}",
  "labels": ["{{ALIAS1}}", "{{ALIAS2}}"],
  "props": { "jurisdiction": "{{JURIS}}", "version": "{{VERSION}}", "sensitive": {{IS_SENSITIVE}} }
}
```

**Relation (edge)**

```json
{
  "from": "{{SUBJECT_ID}}",
  "predicate": "{{PREDICATE}}",
  "to": "{{OBJECT_ID}}",
  "props": { "confidence": {{CONFIDENCE}}, "observed_at": "{{TS}}", "source": "{{URL_OR_REF}}" }
}
```

**Observation (temporal)**

```json
{
  "subject": "{{ENTITY_OR_EDGE_ID}}",
  "key": "{{FIELD}}",
  "value": "{{VALUE}}",
  "observed_at": "{{TS}}",
  "valid_from": "{{FROM}}",
  "valid_to": "{{TO}}",
  "who": "{{AGENT_NAME}}",
  "source": "{{URL_OR_REF}}"
}
```

---

## 5) Kontrak Perilaku Agen (Prompt Kebijakan)

```
WHEN writing to memory:
- Upsert entity with {type,id,labels}; NEVER create without type/id.
- Create/merge relation {predicate,observed_at,source,confidence} idempotently.
- Append observations with timestamps; DO NOT overwrite historical values.
- Respect namespace={{NAMESPACE}} and policy={{MEMORY_POLICY_LINK}}.
- For secrets/PII: store pointer only {kind, location_ref, owner}, tag sensitive:true.
```

---

## 6) Operasi CRUD (Idempoten & Deterministik)

* **Upsert entitas**: cek eksistensi by `id`; gabungkan `props` tanpa menghapus field penting.
* **Upsert relasi**: `(from, predicate, to)` unik; update hanya `props`.
* **Observasi**: selalu `append` + stempel waktu.
* **Query umum**:

  * *Neighborhood 2-hop*: ringkas konteks inti `{{ENTITY_FOCUS}}`.
  * *Temporal slice*: status per `{{DATE_ISO}}`.
  * *Path audit*: jejak `CLAIM → SOURCE`.

---

## 7) Kurasi, Deduplikasi, & TTL

**Jadwal rutin**

* **Harian**: deduplikasi triple `(subject,predicate,object)` → simpan hanya salinan terbaru.
* **Mingguan**: laporan 10 node/edge terbaru + semua bertag `sensitive:true` untuk review.
* **Bulanan**: pruning observasi bertag `temporary:true` lebih tua dari `{{TTL_DAYS}}` hari.

**Script contoh (bash, aman & dapat dipantau)**

```bash
#!/usr/bin/env bash
set -euo pipefail
MEM_PATH="{{MEMORY_PATH}}"
BACKUP_DIR="{{BACKUP_DIR}}"
STAMP="$(date -Iseconds)"
mkdir -p "$BACKUP_DIR"
trap 'echo "[ERR] Prune failed at $STAMP" >&2' ERR

# Backup + hash
cp "$MEM_PATH" "$BACKUP_DIR/graph.$STAMP.json"
sha256sum "$BACKUP_DIR/graph.$STAMP.json" > "$BACKUP_DIR/graph.$STAMP.json.sha256"

# Placeholder: panggil util dedup/TTL (implement oleh agen):
# node tools/dedup.js --input "$MEM_PATH" --output "$MEM_PATH"
# node tools/ttl-prune.js --input "$MEM_PATH" --ttl-days {{TTL_DAYS}}

echo "[OK] Prune/backup done: $STAMP"
```

---

## 8) Backup, Ekspor, & Portabilitas

* **Backup harian** ke `{{BACKUP_DIR}}`; simpan `*.sha256` untuk integritas.
* **Ekspor** ke `GraphML/NDJSON` untuk visualisasi (Gephi) → `{{EXPORT_PATH}}`.
* **Retensi**: `{{BACKUP_RETENTION_DAYS}}` hari.

---

## 9) Keamanan & Kepatuhan

* Izin file: `0600` untuk berkas memori & backup.
* PII/rahasia: simpan pointer; rahasia asli dikelola di `{{SECRET_MANAGER}}`.
* Audit berkala: `{{AUDIT_FREQ}}` (cek akses OS, perubahan skema, entitas sensitif).

---

## 10) Skalabilitas & Backend Alternatif (opsional)

* **Trigger migrasi**: jika *neighborhood 2-hop* > `{{P95_TARGET_MS}}` ms atau ukuran > `{{GRAPH_SIZE_THRESHOLD}}` MB.
* **Opsi**: `{{GRAPH_BACKEND}}` (contoh: Postgres+edge table, Graphiti, Zep, Neptune).
* **Strategi migrasi**: ekspor → ETL → validasi hash → verifikasi kueri (golden queries) → cutover.

---

## 11) Monitoring & SLO

* **SLO**: P95 query ≤ `{{P95_TARGET_MS}}` ms; kesalahan tulis memori ≤ `{{ERROR_BUDGET}}`/minggu.
* **Log**: tulis ke `{{LOG_PATH}}`; failure diarahkan ke `{{PENDING_QUEUE}}` untuk replay.
* **Golden queries**: `{{GOLDEN_QUERY_LIST}}` dieksekusi harian.

---

## 12) Failure Modes & Recovery

* **Graf korup/lenyap**: restore dari backup terbaru + verifikasi `sha256`.
* **Timeout MCP**: tulis ke `{{PENDING_QUEUE}}` lalu replay job (`{{REPLAY_CMD}}`).
* **Kontradiksi fakta**: tandai konflik → buat tiket `{{TICKETING_SYSTEM}}` untuk resolusi manual.

---

## 13) Unit Test & Validasi Skema

**Contoh (pseudo, agen akan mengisi implementasi):**

```ts
// tests/memory.spec.ts (placeholder)
// - upsert entity idempotent
// - unique edge (from,predicate,to)
// - append-only observations
// - schema guard: entity must have {type,id}
```

---

## 14) Pro & Kontra (ringkas)

**Pro**: struktur eksplisit, query/audit kuat, portabel lokal.
**Kontra**: privasi/backup tanggung jawab lokal; variasi fitur antar implementasi; risiko bloat.

---

## 15) Asumsi & Validasi

* **Asumsi**: implementasi mendukung `--memory-path`/namespace. **Validasi**: `{{MEMORY_SERVER_HELP_CMD}}`.
* **Asumsi**: agen klien patuh kontrak prompt. **Validasi**: uji e2e penulisan memori.

---

## 16) Tugas /init yang Harus Diisi Agen

* Set `{{NAMESPACE}}`, `{{MEMORY_PATH}}`, `{{BACKUP_DIR}}`, `{{EXPORT_PATH}}`.
* Lengkapi `{{ID_SCHEME}}`, daftar `{{PREDICATES_ALLOWED}}`.
* Isi `{{TTL_DAYS}}`, `{{P95_TARGET_MS}}`, `{{GRAPH_SIZE_THRESHOLD}}`, `{{ERROR_BUDGET}}`.
* Daftarkan `{{GOLDEN_QUERY_LIST}}` + skenario audit.
* Perbarui `.mcp.json` & jalankan smoke test memori.

---

## 17) Changelog

* `{{DATE}}` — Template dibuat.
* `{{DATE}}` — Disesuaikan oleh agen (isi hasil `/init`).

