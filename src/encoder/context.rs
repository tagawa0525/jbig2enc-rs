use std::collections::HashMap;

use leptonica::recog::JbComponent;
use leptonica::recog::jbclass::{TEMPLATE_BORDER, correlation_init};
use leptonica::region::{ConnectivityType, pix_count_components};
use leptonica::{Pix, PixelDepth};

use crate::comparator::are_equivalent;
use crate::error::Jbig2Error;
use crate::symbol::{SymbolInstance, TextRegionConfig, encode_symbol_table, encode_text_region};
use crate::wire::{
    FileHeader, PageInfo, SEGMENT_END_OF_FILE, SEGMENT_END_OF_PAGE, SEGMENT_IMM_TEXT_REGION,
    SEGMENT_PAGE_INFORMATION, SEGMENT_SYMBOL_TABLE, SegmentHeader, SymbolDict, TextRegion,
    TextRegionSymInsts,
};

/// シンボル数からIAIDビット数（ceil(log2(v))）を算出する。
///
/// C++版 `log2up()`（`jbig2enc.cc:84-93`）に対応。
pub fn log2up(v: usize) -> u32 {
    if v <= 1 {
        return 0;
    }
    usize::BITS - (v - 1).leading_zeros()
}

/// マルチページJBIG2圧縮コンテキスト。
///
/// C++版 `struct jbig2ctx`（`jbig2enc.cc:98-122`）に対応。
///
/// 使用手順:
/// 1. `new()` でコンテキスト生成
/// 2. `add_page()` で各ページを追加（シンボル抽出・分類）
/// 3. `pages_complete()` でグローバルシンボルテーブルを符号化
/// 4. `produce_page()` で各ページを符号化
pub struct Jbig2Context {
    classer: leptonica::recog::JbClasser,
    xres: u32,
    yres: u32,
    full_headers: bool,
    pdf_page_numbering: bool,
    segnum: u32,
    symtab_segment: Option<u32>,
    pagecomps: HashMap<usize, Vec<usize>>,
    single_use_symbols: HashMap<usize, Vec<usize>>,
    num_global_symbols: usize,
    page_xres: Vec<u32>,
    page_yres: Vec<u32>,
    page_width: Vec<u32>,
    page_height: Vec<u32>,
    symmap: HashMap<usize, usize>,
    verbose: bool,
    refinement: bool,
    #[allow(dead_code)]
    refine_level: i32,
    #[allow(dead_code)]
    baseindexes: Vec<usize>,
}

impl Jbig2Context {
    /// マルチページ圧縮コンテキストを生成する。
    ///
    /// C++版 `jbig2_init()`（`jbig2enc.cc:125-143`）に対応。
    ///
    /// # Arguments
    /// - `thresh` - 分類器の閾値（0.4〜1.0、0.85推奨）
    /// - `weight` - 分類器の重み係数（0.0〜1.0、0.5推奨）
    /// - `xres`/`yres` - 解像度（ppi）
    /// - `full_headers` - true: 完全なJBIG2ファイル、false: PDF埋め込み用断片
    /// - `refine_level` - リファインメントレベル（<0 で無効）
    pub fn new(
        thresh: f32,
        weight: f32,
        xres: u32,
        yres: u32,
        full_headers: bool,
        refine_level: i32,
    ) -> Result<Self, Jbig2Error> {
        let classer = correlation_init(JbComponent::ConnComps, 9999, 9999, thresh, weight)
            .map_err(|e| Jbig2Error::InvalidInput(e.to_string()))?;

        Ok(Self {
            classer,
            xres,
            yres,
            full_headers,
            pdf_page_numbering: !full_headers,
            segnum: 0,
            symtab_segment: None,
            pagecomps: HashMap::new(),
            single_use_symbols: HashMap::new(),
            num_global_symbols: 0,
            page_xres: Vec::new(),
            page_yres: Vec::new(),
            page_width: Vec::new(),
            page_height: Vec::new(),
            symmap: HashMap::new(),
            verbose: false,
            refinement: refine_level >= 0,
            refine_level,
            baseindexes: Vec::new(),
        })
    }

    /// verbose モードを設定する。
    ///
    /// 有効にすると `pages_complete()` で圧縮統計を stderr に出力する。
    /// C++版の `-v` フラグに対応。
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// 圧縮統計の文字列を返す。
    ///
    /// `pages_complete()` 後に呼び出すことで、ページ数・シンボル数・log2 を
    /// C++版 `jbig2enc.cc:662-665` と同じフォーマットで取得できる。
    pub fn compression_stats(&self) -> String {
        let npages = self.classer.npages;
        let nsymbols = self.classer.pixat.len();
        let log2 = log2up(nsymbols);
        format!("JBIG2 compression complete. pages:{npages} symbols:{nsymbols} log2:{log2}")
    }

    /// ページを追加し、シンボル抽出・分類を実行する。
    ///
    /// C++版 `jbig2_add_page()`（`jbig2enc.cc:498-530`）に対応。
    pub fn add_page(&mut self, pix: &Pix) -> Result<(), Jbig2Error> {
        if pix.depth() != PixelDepth::Bit1 {
            return Err(Jbig2Error::InvalidInput(format!(
                "expected 1bpp image, got {}bpp",
                pix.depth().bits()
            )));
        }

        if self.refinement {
            self.baseindexes.push(self.classer.base_index);
        }

        self.classer
            .add_page(pix)
            .map_err(|e| Jbig2Error::InvalidInput(e.to_string()))?;

        self.page_width.push(pix.width());
        self.page_height.push(pix.height());
        self.page_xres.push(if self.xres != 0 {
            self.xres
        } else {
            pix.xres().max(0) as u32
        });
        self.page_yres.push(if self.yres != 0 {
            self.yres
        } else {
            pix.yres().max(0) as u32
        });

        Ok(())
    }

    /// シンボルテーブルを符号化し、ファイルヘッダ + グローバル辞書セグメントを返す。
    ///
    /// C++版 `jbig2_pages_complete()`（`jbig2enc.cc:537-722`）に対応。
    pub fn pages_complete(&mut self) -> Result<Vec<u8>, Jbig2Error> {
        if self.verbose {
            eprintln!("{}", self.compression_stats());
        }

        let single_page = self.classer.npages == 1;
        let num_symbols = self.classer.pixat.len();

        // 各シンボルの使用回数をカウント
        let mut symbol_used = vec![0u32; num_symbols];
        for &class_id in &self.classer.naclass {
            if class_id >= num_symbols {
                return Err(Jbig2Error::InvalidInput(format!(
                    "naclass contains class_id {class_id} >= num_symbols {num_symbols}"
                )));
            }
            symbol_used[class_id] += 1;
        }

        // グローバル辞書（2回以上使用 or 単一ページ）
        let multiuse_symbols: Vec<usize> = (0..num_symbols)
            .filter(|&i| symbol_used[i] > 1 || single_page)
            .collect();
        self.num_global_symbols = multiuse_symbols.len();

        // ページ → コンポーネントマッピングと単一使用シンボルの分類
        for (comp_idx, &page_num) in self.classer.napage.iter().enumerate() {
            self.pagecomps.entry(page_num).or_default().push(comp_idx);
            let symbol = self.classer.naclass[comp_idx];
            if symbol_used[symbol] == 1 && !single_page {
                self.single_use_symbols
                    .entry(page_num)
                    .or_default()
                    .push(symbol);
            }
        }

        // グローバルシンボルテーブルを符号化
        let result = encode_symbol_table(
            &self.classer.pixat,
            &multiuse_symbols,
            true, // unborder（avg_templates == NULL → unborder）
            TEMPLATE_BORDER as u32,
        )?;
        let sym_data = result.data;
        self.symmap = result.symmap;

        // 出力を組み立てる
        let mut output = Vec::new();

        // FileHeader（full_headers時のみ）
        if self.full_headers {
            let header = FileHeader {
                organisation_type: true,
                unknown_n_pages: false,
                n_pages: self.classer.npages as u32,
            };
            output.extend_from_slice(&header.to_bytes());
        }

        // SymbolDict セグメント
        let symtab = SymbolDict {
            sdhuff: false,
            sdrefagg: false,
            sdhuffdh: 0,
            sdhuffdw: 0,
            sdhuffbmsize: false,
            sdhuffagginst: false,
            bmcontext: false,
            bmcontextretained: false,
            sdtemplate: 0,
            sdrtemplate: false,
            a1x: 3,
            a1y: -1,
            a2x: -3,
            a2y: -1,
            a3x: 2,
            a3y: -2,
            a4x: -2,
            a4y: -2,
            exsyms: multiuse_symbols.len() as u32,
            newsyms: multiuse_symbols.len() as u32,
        };
        let symtab_bytes = symtab.to_bytes();

        self.symtab_segment = Some(self.segnum);
        let seg = SegmentHeader {
            number: self.segnum,
            seg_type: SEGMENT_SYMBOL_TABLE,
            deferred_non_retain: false,
            retain_bits: 1,
            referred_to: vec![],
            page: 0,
            data_length: (symtab_bytes.len() + sym_data.len()) as u32,
        };
        self.segnum += 1;

        output.extend_from_slice(&seg.to_bytes());
        output.extend_from_slice(&symtab_bytes);
        output.extend_from_slice(&sym_data);

        Ok(output)
    }

    /// 指定ページのテキストリージョンを符号化する。
    ///
    /// C++版 `jbig2_produce_page()`（`jbig2enc.cc:725-893`）に対応。
    ///
    /// # Arguments
    /// - `page_no` - ページ番号（0始まり、add_page の呼び出し順）
    /// - `xres`/`yres` - 解像度上書き。None の場合はページ自身の値を使用
    pub fn produce_page(
        &mut self,
        page_no: usize,
        xres: Option<u32>,
        yres: Option<u32>,
    ) -> Result<Vec<u8>, Jbig2Error> {
        // pages_complete が呼ばれていること
        let symtab_seg = self.symtab_segment.ok_or_else(|| {
            Jbig2Error::InvalidInput("pages_complete() must be called before produce_page()".into())
        })?;

        // ページ番号の検証
        if page_no >= self.classer.npages {
            return Err(Jbig2Error::InvalidInput(format!(
                "page_no {page_no} >= npages {}",
                self.classer.npages
            )));
        }

        let is_last_page = page_no + 1 == self.classer.npages;
        let include_trailer = is_last_page && self.full_headers;
        let page_assoc = if self.pdf_page_numbering {
            1
        } else {
            (1 + page_no) as u32
        };

        let mut output = Vec::new();

        // ---- ページ情報セグメント ----
        let pageinfo = PageInfo {
            width: self.page_width[page_no],
            height: self.page_height[page_no],
            xres: xres.unwrap_or(self.page_xres[page_no]),
            yres: yres.unwrap_or(self.page_yres[page_no]),
            is_lossless: self.refinement,
            contains_refinements: false,
            default_pixel: false,
            default_operator: 0,
            aux_buffers: false,
            operator_override: false,
            segment_flags: 0,
        };
        let pageinfo_bytes = pageinfo.to_bytes();

        let seg_pageinfo = SegmentHeader {
            number: self.segnum,
            seg_type: SEGMENT_PAGE_INFORMATION,
            deferred_non_retain: false,
            retain_bits: 0,
            referred_to: vec![],
            page: page_assoc,
            data_length: pageinfo_bytes.len() as u32,
        };
        self.segnum += 1;
        output.extend_from_slice(&seg_pageinfo.to_bytes());
        output.extend_from_slice(&pageinfo_bytes);

        // ---- ページ固有シンボルテーブル（該当時） ----
        let page_single_use = self.single_use_symbols.get(&page_no);
        let has_extra_symtab = page_single_use.is_some_and(|v| !v.is_empty());

        let mut second_symmap = HashMap::new();
        let mut extra_symtab_seg_num = 0u32;

        if has_extra_symtab {
            let single_use = page_single_use.unwrap();
            let result = encode_symbol_table(
                &self.classer.pixat,
                single_use,
                true,
                TEMPLATE_BORDER as u32,
            )?;
            second_symmap = result.symmap;
            let extra_sym_data = result.data;

            let extra_symtab = SymbolDict {
                sdhuff: false,
                sdrefagg: false,
                sdhuffdh: 0,
                sdhuffdw: 0,
                sdhuffbmsize: false,
                sdhuffagginst: false,
                bmcontext: false,
                bmcontextretained: false,
                sdtemplate: 0,
                sdrtemplate: false,
                a1x: 3,
                a1y: -1,
                a2x: -3,
                a2y: -1,
                a3x: 2,
                a3y: -2,
                a4x: -2,
                a4y: -2,
                exsyms: single_use.len() as u32,
                newsyms: single_use.len() as u32,
            };
            let extra_symtab_bytes = extra_symtab.to_bytes();

            extra_symtab_seg_num = self.segnum;
            let seg = SegmentHeader {
                number: self.segnum,
                seg_type: SEGMENT_SYMBOL_TABLE,
                deferred_non_retain: false,
                retain_bits: 1,
                referred_to: vec![],
                page: page_assoc,
                data_length: (extra_symtab_bytes.len() + extra_sym_data.len()) as u32,
            };
            self.segnum += 1;

            output.extend_from_slice(&seg.to_bytes());
            output.extend_from_slice(&extra_symtab_bytes);
            output.extend_from_slice(&extra_sym_data);
        }

        // ---- テキストリージョン ----
        let comps = self
            .pagecomps
            .get(&page_no)
            .ok_or_else(|| Jbig2Error::InvalidInput(format!("no components for page {page_no}")))?;

        let numsyms = self.num_global_symbols + page_single_use.map_or(0, |v| v.len());
        let symbits = log2up(numsyms);

        // コンポーネントを SymbolInstance に変換
        let instances: Vec<SymbolInstance> = comps
            .iter()
            .map(|&comp_idx| {
                let (x, y) = self.classer.ptall[comp_idx];
                let class_id = self.classer.naclass[comp_idx];
                SymbolInstance { x, y, class_id }
            })
            .collect();

        let symmap2 = if has_extra_symtab {
            Some(&second_symmap)
        } else {
            None
        };

        let cfg = TextRegionConfig {
            symmap: &self.symmap,
            symmap2,
            global_sym_count: self.num_global_symbols,
            symbits,
            strip_width: 1,
            unborder: true,
            border_size: TEMPLATE_BORDER as u32,
        };

        let text_result = encode_text_region(&instances, &self.classer.pixat, &cfg)?;
        let text_data = text_result.data;

        // テキストリージョンヘッダ
        let textreg = TextRegion {
            width: self.page_width[page_no],
            height: self.page_height[page_no],
            x: 0,
            y: 0,
            comb_operator: 0,
            sbhuff: false,
            sbrefine: self.refinement,
            logsbstrips: 0,
            refcorner: 0,
            transposed: false,
            sbcombop: 0,
            sbdefpixel: false,
            sbdsoffset: 0,
            sbrtemplate: false,
        };
        let textreg_bytes = textreg.to_bytes();

        let textreg_syminsts = TextRegionSymInsts {
            sbnuminstances: comps.len() as u32,
        };
        let syminsts_bytes = textreg_syminsts.to_bytes();

        let textreg_data_length =
            textreg_bytes.len() as u32 + syminsts_bytes.len() as u32 + text_data.len() as u32;

        let mut referred_to = vec![symtab_seg];
        if has_extra_symtab {
            referred_to.push(extra_symtab_seg_num);
        }

        let seg_textreg = SegmentHeader {
            number: self.segnum,
            seg_type: SEGMENT_IMM_TEXT_REGION,
            deferred_non_retain: false,
            retain_bits: 2,
            referred_to,
            page: page_assoc,
            data_length: textreg_data_length,
        };
        self.segnum += 1;

        output.extend_from_slice(&seg_textreg.to_bytes());
        output.extend_from_slice(&textreg_bytes);
        output.extend_from_slice(&syminsts_bytes);
        output.extend_from_slice(&text_data);

        // ---- End of Page（full_headers時のみ）----
        if self.full_headers {
            let seg_eop = SegmentHeader {
                number: self.segnum,
                seg_type: SEGMENT_END_OF_PAGE,
                deferred_non_retain: false,
                retain_bits: 0,
                referred_to: vec![],
                page: page_assoc,
                data_length: 0,
            };
            self.segnum += 1;
            output.extend_from_slice(&seg_eop.to_bytes());
        }

        // ---- End of File（最終ページ + full_headers時）----
        if include_trailer {
            let seg_eof = SegmentHeader {
                number: self.segnum,
                seg_type: SEGMENT_END_OF_FILE,
                deferred_non_retain: false,
                retain_bits: 0,
                referred_to: vec![],
                page: 0,
                data_length: 0,
            };
            self.segnum += 1;
            output.extend_from_slice(&seg_eof.to_bytes());
        }

        Ok(output)
    }

    /// シンボルクラスの統合を行う（ブルートフォース版）。
    ///
    /// C++版 `jbig2enc_auto_threshold()`（`jbig2enc.cc:357-376`）に対応。
    /// `pages_complete()` 前に呼ぶことで、視覚的に等価なシンボルクラスを統合し
    /// シンボル辞書のサイズを削減する。
    ///
    /// 全テンプレートペアを O(n²) で比較する。テンプレート数が少ない場合に適している。
    pub fn auto_threshold(&mut self) {
        let mut i = 0;
        while i < self.classer.pixat.len() {
            let mut j = i + 1;
            while j < self.classer.pixat.len() {
                let equivalent =
                    are_equivalent(&self.classer.pixat[i], &self.classer.pixat[j]).unwrap_or(false);

                if equivalent {
                    unite_and_remove(
                        &mut self.classer.pixat,
                        &mut self.classer.naclass,
                        &mut self.classer.nclass,
                        i,
                        j,
                    );
                    // j 位置に新しい要素が来ているのでインクリメントしない
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }

    /// シンボルクラスの統合を行う（ハッシュ加速版）。
    ///
    /// C++版 `jbig2enc_auto_threshold_using_hash()`（`jbig2enc.cc:428-487`）に対応。
    /// ハッシュ値でバケットに分けてからバケット内で比較することで、
    /// `auto_threshold()` より高速に動作する。
    ///
    /// ハッシュ: `(holes + 10*h + 10000*w) % 10_000_000`
    /// （holes = 4連結の連結成分数, h = 高さ, w = 幅）
    pub fn auto_threshold_using_hash(&mut self) {
        if self.classer.pixat.is_empty() {
            return;
        }

        // Phase 1: ハッシュバケット構築
        let mut buckets: HashMap<u32, Vec<usize>> = HashMap::new();
        for (i, pix) in self.classer.pixat.iter().enumerate() {
            let holes = match pix_count_components(pix, ConnectivityType::FourWay) {
                Ok(count) => count,
                Err(_) => {
                    // 連結成分数を取得できないテンプレートはバケットに入れない。
                    // Phase 2 の比較対象から外れるため、統合されずに残る。
                    continue;
                }
            };
            let w = pix.width();
            let h = pix.height();
            let hash = (holes + 10 * h + 10000 * w) % 10_000_000;
            buckets.entry(hash).or_default().push(i);
        }

        // Phase 2: バケット内でペアワイズ比較し、統合対象を収集
        let mut representant_map: Vec<(usize, Vec<usize>)> = Vec::new();
        for bucket in buckets.values_mut() {
            let mut i = 0;
            while i < bucket.len() {
                let mut to_unite = Vec::new();
                let mut j = i + 1;
                while j < bucket.len() {
                    let first_idx = bucket[i];
                    let second_idx = bucket[j];
                    let equivalent = are_equivalent(
                        &self.classer.pixat[first_idx],
                        &self.classer.pixat[second_idx],
                    )
                    .unwrap_or(false);

                    if equivalent {
                        to_unite.push(second_idx);
                        bucket.swap_remove(j);
                        // j にはバケット末尾の要素が来るのでインクリメントしない
                    } else {
                        j += 1;
                    }
                }
                if !to_unite.is_empty() {
                    representant_map.push((bucket[i], to_unite));
                }
                i += 1;
            }
        }

        // Phase 3: リダイレクトテーブルで naclass を 1 パスで更新
        let n = self.classer.pixat.len();
        let mut redirect: Vec<usize> = (0..n).collect();
        for (representant, to_unite) in &representant_map {
            for &idx in to_unite {
                redirect[idx] = *representant;
            }
        }
        for class_id in &mut self.classer.naclass {
            if *class_id < n {
                *class_id = redirect[*class_id];
            }
        }

        // Phase 4: バッチ削除（高インデックスから順に swap_remove）
        let mut to_remove: Vec<usize> = representant_map
            .into_iter()
            .flat_map(|(_, list)| list)
            .collect();
        to_remove.sort_unstable();
        to_remove.dedup();

        // to_remove は削除前のインデックスで固定されているため、
        // swap + pop を使う場合は必ず高いインデックスから順に処理する。
        // こうすることで、まだ処理していないインデックスより前側の要素だけが
        // 移動し、to_remove 内の残りのインデックスが無効化されない。
        for &idx in to_remove.iter().rev() {
            swap_remove_class_at(
                &mut self.classer.pixat,
                &mut self.classer.naclass,
                &mut self.classer.nclass,
                idx,
            );
        }
    }
}

/// 2つのテンプレートを統合する。
///
/// `second` のテンプレートへの参照を `first` にリダイレクトし、
/// `pixat` から `second` を削除する。
///
/// C++版 `unite_templates_with_indexes()`（`jbig2enc.cc:295-353`）に対応。
fn unite_and_remove(
    pixat: &mut Vec<Pix>,
    naclass: &mut [usize],
    nclass: &mut usize,
    first: usize,
    second: usize,
) {
    // naclass の参照を second → first に変更
    for class_id in naclass.iter_mut() {
        if *class_id == second {
            *class_id = first;
        }
    }

    swap_remove_class_at(pixat, naclass, nclass, second);
}

/// `pixat[idx]` を swap_remove し、末尾から移動した要素への参照を更新する。
///
/// - `pixat[idx]` を `pixat[last]` と swap し、pop で末尾を削除する。
/// - `naclass` 内の `last` への参照を `idx` に書き換える。
/// - `nclass` を 1 減らす。
///
/// 複数インデックスを一括削除する場合は、呼び出し側で必ず降順（高インデックス
/// から順）に呼び出すこと。昇順で呼ぶと swap によってまだ削除していない
/// インデックスの要素が移動し、不正なアクセスを引き起こす。
fn swap_remove_class_at(
    pixat: &mut Vec<Pix>,
    naclass: &mut [usize],
    nclass: &mut usize,
    idx: usize,
) {
    let last = pixat.len() - 1;
    if idx != last {
        pixat.swap(idx, last);
        for class_id in naclass.iter_mut() {
            if *class_id == last {
                *class_id = idx;
            }
        }
    }
    pixat.pop();
    *nclass -= 1;
}
