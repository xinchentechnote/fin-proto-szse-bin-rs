//! Encode/decode benchmarks for the SZSE binary library, mirroring
//! bench/bench.zig in fin-proto-szse-bin-zig: same methodology (10k warmup
//! iterations, then 10 batches of 100k operations, fastest batch reported),
//! so the numbers are directly comparable.
//!
//! Output is one TSV line per case on stdout:
//!   lib \t lang \t op \t ns_per_op \t frame_bytes
//!
//! Run with: cargo run --release --example bench

use std::hint::black_box;
use std::time::Instant;

use binary_codec::BinaryCodec;
use bytes::{Bytes, BytesMut};
use szse_binary::extend_100101::Extend100101;
use szse_binary::new_order::{NewOrder, NewOrderApplExtendEnum};

const LIB: &str = "szse";
const LANG: &str = "rust";
const ITERS: usize = 100_000;
const BATCHES: usize = 10;
const WARMUP: usize = 10_000;

fn msg() -> NewOrder {
    NewOrder {
        appl_id: "010".to_string(),
        submitting_pbuid: "PBU001".to_string(),
        security_id: "600000".to_string(),
        security_id_source: "101".to_string(),
        owner_type: 1,
        clearing_firm: "01".to_string(),
        transact_time: 2026_0109_0930_00123,
        user_info: "user01".to_string(),
        cl_ord_id: "ORD000001".to_string(),
        account_id: "ACC000000001".to_string(),
        branch_id: "0001".to_string(),
        order_restrictions: "R".to_string(),
        side: "1".to_string(),
        ord_type: "2".to_string(),
        order_qty: 100,
        price: 10500,
        appl_extend: NewOrderApplExtendEnum::Extend100101(Extend100101 {
            stop_px: -123456789,
            min_qty: -123456789,
            max_price_levels: 1234,
            time_in_force: "a".to_string(),
            cash_margin: "a".to_string(),
        }),
    }
}

fn main() {
    // --- encode -------------------------------------------------------------
    let msg = msg();
    let mut buf = BytesMut::with_capacity(1024);
    for _ in 0..WARMUP {
        buf.clear();
        msg.encode(&mut buf);
    }
    let frame_bytes = buf.len();
    let mut best: u128 = u128::MAX;
    for _ in 0..BATCHES {
        let start = Instant::now();
        for _ in 0..ITERS {
            buf.clear();
            msg.encode(&mut buf);
        }
        best = best.min(start.elapsed().as_nanos());
    }
    black_box(&buf);
    println!(
        "{}\t{}\tencode\t{:.1}\t{}",
        LIB,
        LANG,
        best as f64 / ITERS as f64,
        frame_bytes
    );

    // --- decode -------------------------------------------------------------
    let raw: &'static [u8] = Box::leak(buf.to_vec().into_boxed_slice());
    for _ in 0..WARMUP {
        let mut b = Bytes::from_static(raw);
        let m = NewOrder::decode(&mut b).unwrap();
        black_box(m);
    }
    let mut best: u128 = u128::MAX;
    for _ in 0..BATCHES {
        let start = Instant::now();
        for _ in 0..ITERS {
            let mut b = Bytes::from_static(raw);
            let m = NewOrder::decode(&mut b).unwrap();
            black_box(m);
        }
        best = best.min(start.elapsed().as_nanos());
    }
    println!(
        "{}\t{}\tdecode\t{:.1}\t{}",
        LIB,
        LANG,
        best as f64 / ITERS as f64,
        raw.len()
    );
}
