# Phase 1: 算術符号化 (`arith/`)

**Status**: IN_PROGRESS

## 概要

C++版 `jbig2arith.h` + `jbig2arith.cc` (約970行) をRustに移植する。
JBIG2算術符号化の基盤であり、他の全フェーズが依存する。

## C++対応

| C++関数/構造体 | Rustモジュール | 説明 |
|---|---|---|
| `ctbl[]` (92エントリ) | `arith::state_table` | QMコーダ状態テーブル |
| `jbig2enc_ctx` | `arith::encoder::ArithEncoder` | コーダ状態管理 |
| `encode_bit` / `byteout` / `emit` / `encode_final` | `arith::encoder` | ビット符号化ステートマシン |
| `jbig2enc_int` / `intencrange[]` | `arith::int_encode` | 整数符号化（13種） |
| `jbig2enc_oob` | `arith::int_encode` | OOBセンチネル |
| `jbig2enc_iaid` | `arith::iaid` | シンボルID符号化 |
| `jbig2enc_bitimage` | `arith::image` | パックド1bppビットマップ符号化 |
| `jbig2enc_refine` | `arith::image` | リファインメント符号化 |

## 外部依存

なし（leptonica不要、純粋なビット演算のみ）

## モジュール構成

```
src/arith/
  mod.rs           -- pub use 再エクスポート
  state_table.rs   -- QMコーダ 92状態テーブル (StateEntry, STATE_TABLE)
  encoder.rs       -- ArithEncoder (c/a/ct/b/bp、出力バッファ、コンテキスト)
  int_encode.rs    -- 整数符号化 (IntProc enum, encode_int, encode_oob)
  iaid.rs          -- シンボルID符号化 (encode_iaid)
  image.rs         -- ビットマップ符号化 (encode_bitimage, encode_refine)
```

## PR構成

### PR 1: 状態テーブル + ArithEncoder core
- `StateEntry` 構造体 + `STATE_TABLE` (92エントリ)
- `ArithEncoder::new()` / `reset()` / `flush()` / `to_vec()`
- `encode_bit()` / `byteout()` / `emit()` / `encode_final()`
- テスト: 状態テーブル検証、H.2テストベクタ

### PR 2: 整数符号化
- `IntProc` enum (13種)
- `IntEncRange` + レンジテーブル (13エントリ)
- `encode_int()` / `encode_oob()`
- `encode_iaid()`
- テスト: 既知整数値の符号化結果比較

### PR 3: ビットマップ符号化
- `encode_bitimage()` — TPGD対応パックド1bpp符号化
- `encode_refine()` — リファインメント符号化
- テスト: 既知ビットマップパターンの符号化結果比較

## テスト戦略

1. 状態テーブルの全92エントリをC++の`ctbl[]`と照合
2. JBIG2仕様 Appendix H.2のテストベクタでバイト比較
3. 既知整数値の符号化結果をC++出力と比較
4. 小さな既知ビットマップ（8x8パターン）のTPGD on/off両方で比較

## リスク

- `encode_bit`/`byteout`/`emit`のステートマシンはビット単位の精度が必要
- 対処: H.2テストベクタを最初に通す。中間状態のステップ比較で検証
