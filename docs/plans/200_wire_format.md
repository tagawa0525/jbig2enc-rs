# Phase 2: ワイヤフォーマット (`wire/`)

**Status**: IMPLEMENTED

## 概要

JBIG2バイナリ構造体とセグメントヘッダのシリアライズを実装する。

## C++対応

| C++ファイル       | 行数  | 内容               |
| ----------------- | ----- | ------------------ |
| `jbig2structs.h`  | 191行 | バイナリ構造体定義 |
| `jbig2segments.h` | 173行 | セグメントヘッダ   |

合計: 約360行

## 設計方針

### ビッグエンディアン・手動バイトパッキング

C++版は `#[repr(C, packed)]` + `htonl()` でビッグエンディアン変換しているが、
Rust版では packed struct は使わず、`to_bytes() -> Vec<u8>` メソッドで手動シリアライズする。

理由:

- packed struct はRustで `unsafe` が必要（アライメント違反の参照が未定義動作）
- ビットフィールドのエンディアン依存レイアウト（C++の `__BIG_ENDIAN__` 分岐）を避けられる
- 明示的なバイト操作の方がポータブルで検証しやすい

### 構造体一覧

| Rust構造体           | C++構造体                    | サイズ | 用途                                     |
| -------------------- | ---------------------------- | ------ | ---------------------------------------- |
| `FileHeader`         | `jbig2_file_header`          | 13B    | ファイルヘッダ                           |
| `PageInfo`           | `jbig2_page_info`            | 19B    | ページ情報セグメント                     |
| `GenericRegion`      | `jbig2_generic_region`       | 26B    | ジェネリックリージョン                   |
| `SymbolDict`         | `jbig2_symbol_dict`          | 18B    | シンボル辞書                             |
| `TextRegion`         | `jbig2_text_region`          | 19B    | テキストリージョン                       |
| `TextRegionAtFlags`  | `jbig2_text_region_atflags`  | 4B     | テキストリージョンATフラグ               |
| `TextRegionSymInsts` | `jbig2_text_region_syminsts` | 4B     | テキストリージョンシンボルインスタンス数 |
| `SegmentHeader`      | `Segment`                    | 可変   | セグメントヘッダ                         |

### `SegmentHeader` の可変長フィールド

セグメントヘッダのサイズは以下に依存:

- `reference_size`: segment number ≤ 256 → 1B, ≤ 65536 → 2B, else 4B
- `page_size`: page ≤ 255 → 1B, else 4B
- `referred_to`: reference_size × referred_to.len()

合計 = 6 (固定部) + reference_size × n_referred + page_size + 4 (data length)

### 固定部分のバイトレイアウト (`jbig2_segment` = 6バイト)

```text
[0..4]  segment number (u32 BE)
[4]     flags byte:
          bit 0-5: type (6 bits)
          bit 6:   page_assoc_size (1 bit) — 0=1byte, 1=4bytes
          bit 7:   deferred_non_retain (1 bit)
[5]     referred-to count byte:
          bit 0-4: retain_bits (5 bits)
          bit 5-7: segment_count (3 bits)
```

注意: C++版では segment_count が 0-4 の範囲でのみ使用（5以上は長形式が必要だが実装されていない）。

## ファイル構成

```text
src/wire/
  mod.rs          -- pub use re-exports
  constants.rs    -- マジックバイト、セグメントタイプ定数（済み）
  structs.rs      -- バイナリ構造体 to_bytes()
  segment.rs      -- SegmentHeader to_bytes()
tests/
  wire_structs.rs -- 構造体シリアライズテスト
  wire_segment.rs -- セグメントヘッダテスト
```

## テスト方針

各構造体の `to_bytes()` 出力をC++版の `memcpy` 結果と同一のバイト列で検証する。
C++の使用パターン（`jbig2enc.cc`）から実際のフィールド値を取り、期待バイト列を手計算する。

## PR構成

### PR1: 構造体 + セグメント RED/GREEN

1. `docs:` 計画書コミット
2. `test:` structs + segment テスト（RED: `#[ignore]`）
3. `feat:` structs 実装（GREEN）
4. `feat:` segment 実装（GREEN）

## 参照

- C++ 構造体定義: [jbig2enc/src/jbig2structs.h](https://github.com/agl/jbig2enc/blob/master/src/jbig2structs.h)
- C++ セグメントヘッダ: [jbig2enc/src/jbig2segments.h](https://github.com/agl/jbig2enc/blob/master/src/jbig2segments.h)
- C++ 使用パターン: [jbig2enc/src/jbig2enc.cc](https://github.com/agl/jbig2enc/blob/master/src/jbig2enc.cc)
- JBIG2仕様: [jbig2enc/doc/fcd14492.pdf](https://github.com/agl/jbig2enc/blob/master/doc/fcd14492.pdf) (Section 7.2, 7.4)
