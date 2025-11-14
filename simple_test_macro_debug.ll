; ModuleID = 'simple_test_macro.cbe1867ae30e759b-cgu.0'
source_filename = "simple_test_macro.cbe1867ae30e759b-cgu.0"
target datalayout = "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20"
target triple = "wasm32-unknown-wasip1"

%"proc_macro::bridge::client::HandleCounters" = type { %"core::sync::atomic::AtomicU32", %"core::sync::atomic::AtomicU32", %"core::sync::atomic::AtomicU32", %"core::sync::atomic::AtomicU32" }
%"core::sync::atomic::AtomicU32" = type { i32 }

@0 = private unnamed_addr constant <{ [4 x i8], [4 x i8] }> <{ [4 x i8] zeroinitializer, [4 x i8] undef }>, align 4
@"_ZN10proc_macro6bridge6client5state12BRIDGE_STATE28_$u7b$$u7b$closure$u7d$$u7d$3VAL17h35515071b4f0251bE" = external dso_local global ptr
@alloc_b528b4e8f2b0a1f7e7a8d6c80f929ed7 = private unnamed_addr constant <{ ptr }> <{ ptr @_ZN4core3ops8function6FnOnce9call_once17h724cd20ab485ddabE }>, align 4
@vtable.0 = private unnamed_addr constant <{ [12 x i8], ptr }> <{ [12 x i8] c"\00\00\00\00\00\00\00\00\01\00\00\00", ptr @"_ZN68_$LT$std..thread..local..AccessError$u20$as$u20$core..fmt..Debug$GT$3fmt17h0f90263692302964E" }>, align 4
@alloc_2ee7ba9733a263ad3a32ba87b5ec3eae = private unnamed_addr constant <{ [70 x i8] }> <{ [70 x i8] c"cannot access a Thread Local Storage value during or after destruction" }>, align 1
@alloc_fed7c39be2257104251b708fe6a72699 = private unnamed_addr constant <{ [58 x i8] }> <{ [58 x i8] c"/home/ubuntu/macovedj/rust/library/std/src/thread/local.rs" }>, align 1
@alloc_e6c9f9962a69f7b89e88e08c7a757000 = private unnamed_addr constant <{ ptr, [12 x i8] }> <{ ptr @alloc_fed7c39be2257104251b708fe6a72699, [12 x i8] c":\00\00\00\04\01\00\00\1A\00\00\00" }>, align 4
@vtable.1 = private unnamed_addr constant <{ [12 x i8], ptr }> <{ [12 x i8] c"\00\00\00\00\00\00\00\00\01\00\00\00", ptr @"_ZN57_$LT$proc_macro..LexError$u20$as$u20$core..fmt..Debug$GT$3fmt17h528a9c6ca545d38cE" }>, align 4
@alloc_00ae4b301f7fab8ac9617c03fcbd7274 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Result::unwrap()` on an `Err` value" }>, align 1
@alloc_ce18dc9b9ca144fde65711b7622c392e = private unnamed_addr constant <{ [8 x i8] }> <{ [8 x i8] c"LexError" }>, align 1
@alloc_1e51fcb9bd01d98adcecd30e41bb5e77 = private unnamed_addr constant <{ [37 x i8] }> <{ [37 x i8] c"impl Test { fn test() -> u32 { 42 } }" }>, align 1
@alloc_2fde4ec5f096efdf419de6d65bf7eb80 = private unnamed_addr constant <{ [20 x i8] }> <{ [20 x i8] c"simple_test_macro.rs" }>, align 1
@alloc_719f6a58c1831a79b91c1db0675fe4c4 = private unnamed_addr constant <{ ptr, [12 x i8] }> <{ ptr @alloc_2fde4ec5f096efdf419de6d65bf7eb80, [12 x i8] c"\14\00\00\00\08\00\00\005\00\00\00" }>, align 4
@alloc_6d61909b84c2d9e08f37facf1736c7b0 = private unnamed_addr constant <{ [5 x i8] }> <{ [5 x i8] c"42u32" }>, align 1
@alloc_82198754260bc90b7e65095d98d42cbc = private unnamed_addr constant <{ ptr, [12 x i8] }> <{ ptr @alloc_2fde4ec5f096efdf419de6d65bf7eb80, [12 x i8] c"\14\00\00\00\12\00\00\00\15\00\00\00" }>, align 4
@alloc_b0c53e55057e69dee802c11500c76a1a = private unnamed_addr constant <{ [10 x i8] }> <{ [10 x i8] c"SimpleTest" }>, align 1
@_ZN10proc_macro6bridge6client8COUNTERS17h123be4e8787dfa25E = external dso_local global %"proc_macro::bridge::client::HandleCounters"
@alloc_838643ae3af987662ea334a8391c7e77 = private unnamed_addr constant <{ [11 x i8] }> <{ [11 x i8] c"simple_attr" }>, align 1
@alloc_e165750b2c9b7609c3ed9fd42a449661 = private unnamed_addr constant <{ [11 x i8] }> <{ [11 x i8] c"simple_bang" }>, align 1
@alloc_d4dd14be659da0581772b418b7dbf435 = private unnamed_addr constant <{ [4 x i8], ptr, [4 x i8], ptr, [4 x i8], ptr, ptr, [4 x i8], ptr, [4 x i8], ptr, ptr, [8 x i8], [4 x i8], ptr, [4 x i8], ptr, ptr, [8 x i8] }> <{ [4 x i8] zeroinitializer, ptr @alloc_b0c53e55057e69dee802c11500c76a1a, [4 x i8] c"\0A\00\00\00", ptr inttoptr (i32 4 to ptr), [4 x i8] zeroinitializer, ptr @_ZN10proc_macro6bridge6client8COUNTERS17h123be4e8787dfa25E, ptr @_ZN10proc_macro6bridge14selfless_reify31reify_to_extern_c_fn_hrt_bridge7wrapper17hc2c884cf27212d9bE, [4 x i8] c"\01\00\00\00", ptr @alloc_838643ae3af987662ea334a8391c7e77, [4 x i8] c"\0B\00\00\00", ptr @_ZN10proc_macro6bridge6client8COUNTERS17h123be4e8787dfa25E, ptr @_ZN10proc_macro6bridge14selfless_reify31reify_to_extern_c_fn_hrt_bridge7wrapper17h75894541db39f96aE, [8 x i8] undef, [4 x i8] c"\02\00\00\00", ptr @alloc_e165750b2c9b7609c3ed9fd42a449661, [4 x i8] c"\0B\00\00\00", ptr @_ZN10proc_macro6bridge6client8COUNTERS17h123be4e8787dfa25E, ptr @_ZN10proc_macro6bridge14selfless_reify31reify_to_extern_c_fn_hrt_bridge7wrapper17h72baa72a51305101E, [8 x i8] undef }>, align 4
@__rustc_proc_macro_decls_cbe1867ae30e759b__ = dso_local constant <{ ptr, [4 x i8] }> <{ ptr @alloc_d4dd14be659da0581772b418b7dbf435, [4 x i8] c"\03\00\00\00" }>, align 4
@llvm.used = appending global [1 x ptr] [ptr @__rustc_proc_macro_decls_cbe1867ae30e759b__], section "llvm.metadata"

; <proc_macro::bridge::ExpnGlobals<Span> as proc_macro::bridge::rpc::DecodeMut<S>>::decode
; Function Attrs: nounwind
define internal void @"_ZN107_$LT$proc_macro..bridge..ExpnGlobals$LT$Span$GT$$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17hddbe6a95510ab15eE"(ptr sret([12 x i8]) align 4 %_0, ptr align 4 %r, ptr align 1 %s) unnamed_addr #0 {
start:
; call <proc_macro::bridge::client::Span as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  %_3 = call i32 @"_ZN96_$LT$proc_macro..bridge..client..Span$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17ha0c787454e2dc58cE"(ptr align 4 %r, ptr align 1 %s) #5
; call <proc_macro::bridge::client::Span as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  %_4 = call i32 @"_ZN96_$LT$proc_macro..bridge..client..Span$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17ha0c787454e2dc58cE"(ptr align 4 %r, ptr align 1 %s) #5
; call <proc_macro::bridge::client::Span as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  %_5 = call i32 @"_ZN96_$LT$proc_macro..bridge..client..Span$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17ha0c787454e2dc58cE"(ptr align 4 %r, ptr align 1 %s) #5
  store i32 %_3, ptr %_0, align 4
  %0 = getelementptr inbounds i8, ptr %_0, i32 4
  store i32 %_4, ptr %0, align 4
  %1 = getelementptr inbounds i8, ptr %_0, i32 8
  store i32 %_5, ptr %1, align 4
  ret void
}

; proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::result::Result<T,E>>::encode
; Function Attrs: nounwind
define internal void @"_ZN10proc_macro6bridge104_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..result..Result$LT$T$C$E$GT$$GT$6encode17ha09c881eb3267a40E"(i32 %0, i32 %1, ptr align 4 %w, ptr align 1 %s) unnamed_addr #0 {
start:
  %self = alloca [8 x i8], align 4
  store i32 %0, ptr %self, align 4
  %2 = getelementptr inbounds i8, ptr %self, i32 4
  store i32 %1, ptr %2, align 4
  %_4 = load i32, ptr %self, align 4
  %3 = icmp eq i32 %_4, 0
  br i1 %3, label %bb3, label %bb2

bb3:                                              ; preds = %start
  %4 = getelementptr inbounds i8, ptr %self, i32 4
  %t = load i32, ptr %4, align 4
; call proc_macro::bridge::buffer::Buffer::push
  call void @_ZN10proc_macro6bridge6buffer6Buffer4push17hd875a707b2685225E(ptr align 4 %w, i8 0) #5
; call proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::option::Option<T>>::encode
  call void @"_ZN10proc_macro6bridge100_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..option..Option$LT$T$GT$$GT$6encode17hf7a1b2022cc87ae9E"(i32 %t, ptr align 4 %w, ptr align 1 %s) #5
  br label %bb4

bb2:                                              ; preds = %start
; call proc_macro::bridge::buffer::Buffer::push
  call void @_ZN10proc_macro6bridge6buffer6Buffer4push17hd875a707b2685225E(ptr align 4 %w, i8 1) #5
; call <() as proc_macro::bridge::rpc::Encode<S>>::encode
  call void @"_ZN69_$LT$$LP$$RP$$u20$as$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$GT$6encode17heb8703baf7784cd9E"(ptr align 4 %w, ptr align 1 %s) #5
  br label %bb4

bb4:                                              ; preds = %bb2, %bb3
  ret void

bb1:                                              ; No predecessors!
  unreachable
}

; proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::result::Result<T,E>>::encode
; Function Attrs: nounwind
define internal void @"_ZN10proc_macro6bridge104_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..result..Result$LT$T$C$E$GT$$GT$6encode17he747a828f2cb7c4dE"(ptr align 4 %self, ptr align 4 %w, ptr align 1 %s) unnamed_addr #0 {
start:
  %e = alloca [12 x i8], align 4
  %0 = load i32, ptr %self, align 4
  %1 = icmp eq i32 %0, -2147483645
  %_4 = select i1 %1, i32 0, i32 1
  %2 = icmp eq i32 %_4, 0
  br i1 %2, label %bb3, label %bb2

bb3:                                              ; preds = %start
; call proc_macro::bridge::buffer::Buffer::push
  call void @_ZN10proc_macro6bridge6buffer6Buffer4push17hd875a707b2685225E(ptr align 4 %w, i8 0) #5
; call <() as proc_macro::bridge::rpc::Encode<S>>::encode
  call void @"_ZN69_$LT$$LP$$RP$$u20$as$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$GT$6encode17heb8703baf7784cd9E"(ptr align 4 %w, ptr align 1 %s) #5
  br label %bb4

bb2:                                              ; preds = %start
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %e, ptr align 4 %self, i32 12, i1 false)
; call proc_macro::bridge::buffer::Buffer::push
  call void @_ZN10proc_macro6bridge6buffer6Buffer4push17hd875a707b2685225E(ptr align 4 %w, i8 1) #5
; call <proc_macro::bridge::rpc::PanicMessage as proc_macro::bridge::rpc::Encode<S>>::encode
  call void @"_ZN98_$LT$proc_macro..bridge..rpc..PanicMessage$u20$as$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$GT$6encode17hf798b9715d063dc7E"(ptr align 4 %e, ptr align 4 %w, ptr align 1 %s) #5
  br label %bb4

bb4:                                              ; preds = %bb2, %bb3
  ret void

bb1:                                              ; No predecessors!
  unreachable
}

; proc_macro::bridge::selfless_reify::reify_to_extern_c_fn_hrt_bridge::wrapper
; Function Attrs: nounwind
define internal void @_ZN10proc_macro6bridge14selfless_reify31reify_to_extern_c_fn_hrt_bridge7wrapper17h72baa72a51305101E(ptr sret([20 x i8]) align 4 %_0, ptr align 4 %bridge) unnamed_addr #0 {
start:
  %_5 = alloca [32 x i8], align 4
  %self = alloca [0 x i8], align 1
  %f = alloca [0 x i8], align 1
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_5, ptr align 4 %bridge, i32 32, i1 false)
; call proc_macro::bridge::client::Client<proc_macro::TokenStream,proc_macro::TokenStream>::expand1::{{closure}}
  call void @"_ZN10proc_macro6bridge6client63Client$LT$proc_macro..TokenStream$C$proc_macro..TokenStream$GT$7expand128_$u7b$$u7b$closure$u7d$$u7d$17h3957b62716e0ff30E"(ptr sret([20 x i8]) align 4 %_0, ptr align 1 %f, ptr align 4 %_5) #5
  ret void
}

; proc_macro::bridge::selfless_reify::reify_to_extern_c_fn_hrt_bridge::wrapper
; Function Attrs: nounwind
define internal void @_ZN10proc_macro6bridge14selfless_reify31reify_to_extern_c_fn_hrt_bridge7wrapper17h75894541db39f96aE(ptr sret([20 x i8]) align 4 %_0, ptr align 4 %bridge) unnamed_addr #0 {
start:
  %_5 = alloca [32 x i8], align 4
  %self = alloca [0 x i8], align 1
  %f = alloca [0 x i8], align 1
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_5, ptr align 4 %bridge, i32 32, i1 false)
; call proc_macro::bridge::client::Client<(proc_macro::TokenStream,proc_macro::TokenStream),proc_macro::TokenStream>::expand2::{{closure}}
  call void @"_ZN10proc_macro6bridge6client97Client$LT$$LP$proc_macro..TokenStream$C$proc_macro..TokenStream$RP$$C$proc_macro..TokenStream$GT$7expand228_$u7b$$u7b$closure$u7d$$u7d$17h670d352edbf3ccdbE"(ptr sret([20 x i8]) align 4 %_0, ptr align 1 %f, ptr align 4 %_5) #5
  ret void
}

; proc_macro::bridge::selfless_reify::reify_to_extern_c_fn_hrt_bridge::wrapper
; Function Attrs: nounwind
define internal void @_ZN10proc_macro6bridge14selfless_reify31reify_to_extern_c_fn_hrt_bridge7wrapper17hc2c884cf27212d9bE(ptr sret([20 x i8]) align 4 %_0, ptr align 4 %bridge) unnamed_addr #0 {
start:
  %_5 = alloca [32 x i8], align 4
  %self = alloca [0 x i8], align 1
  %f = alloca [0 x i8], align 1
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_5, ptr align 4 %bridge, i32 32, i1 false)
; call proc_macro::bridge::client::Client<proc_macro::TokenStream,proc_macro::TokenStream>::expand1::{{closure}}
  call void @"_ZN10proc_macro6bridge6client63Client$LT$proc_macro..TokenStream$C$proc_macro..TokenStream$GT$7expand128_$u7b$$u7b$closure$u7d$$u7d$17h61fe9898d7f64129E"(ptr sret([20 x i8]) align 4 %_0, ptr align 1 %f, ptr align 4 %_5) #5
  ret void
}

; proc_macro::bridge::buffer::Buffer::push
; Function Attrs: inlinehint nounwind
define internal void @_ZN10proc_macro6bridge6buffer6Buffer4push17hd875a707b2685225E(ptr align 4 %self, i8 %v) unnamed_addr #1 {
start:
  %v2 = alloca [12 x i8], align 4
  %v1 = alloca [12 x i8], align 4
  %src = alloca [20 x i8], align 4
  %_7 = alloca [20 x i8], align 4
  %b = alloca [20 x i8], align 4
  %0 = getelementptr inbounds i8, ptr %self, i32 4
  %_4 = load i32, ptr %0, align 4
  %1 = getelementptr inbounds i8, ptr %self, i32 8
  %_5 = load i32, ptr %1, align 4
  %_3 = icmp eq i32 %_4, %_5
  br i1 %_3, label %bb1, label %bb4

bb4:                                              ; preds = %start
  br label %bb5

bb1:                                              ; preds = %start
  store i32 0, ptr %v1, align 4
  %2 = getelementptr inbounds i8, ptr %v1, i32 4
  store ptr inttoptr (i32 1 to ptr), ptr %2, align 4
  %3 = getelementptr inbounds i8, ptr %v1, i32 8
  store i32 0, ptr %3, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %v2, ptr align 4 %v1, i32 12, i1 false)
  %4 = getelementptr inbounds i8, ptr %v2, i32 4
  %self3 = load ptr, ptr %4, align 4
  %5 = getelementptr inbounds i8, ptr %v2, i32 8
  %len = load i32, ptr %5, align 4
  %capacity = load i32, ptr %v2, align 4
  store ptr %self3, ptr %src, align 4
  %6 = getelementptr inbounds i8, ptr %src, i32 4
  store i32 %len, ptr %6, align 4
  %7 = getelementptr inbounds i8, ptr %src, i32 8
  store i32 %capacity, ptr %7, align 4
  %8 = getelementptr inbounds i8, ptr %src, i32 12
  store ptr @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from7reserve17h7f50c2c3b7d04034E", ptr %8, align 4
  %9 = getelementptr inbounds i8, ptr %src, i32 16
  store ptr @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from4drop17h97d84a4094b82bf1E", ptr %9, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %b, ptr align 4 %self, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self, ptr align 4 %src, i32 20, i1 false)
  %10 = getelementptr inbounds i8, ptr %b, i32 12
  %_8 = load ptr, ptr %10, align 4
  call void %_8(ptr sret([20 x i8]) align 4 %_7, ptr align 4 %b, i32 1) #5
; call core::ptr::drop_in_place<proc_macro::bridge::buffer::Buffer>
  call void @"_ZN4core3ptr55drop_in_place$LT$proc_macro..bridge..buffer..Buffer$GT$17h24f1b8cbac89e26dE"(ptr align 4 %self) #5
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self, ptr align 4 %_7, i32 20, i1 false)
  br label %bb5

bb5:                                              ; preds = %bb1, %bb4
  %self4 = load ptr, ptr %self, align 4
  %11 = getelementptr inbounds i8, ptr %self, i32 4
  %count = load i32, ptr %11, align 4
  %_9 = getelementptr inbounds i8, ptr %self4, i32 %count
  store i8 %v, ptr %_9, align 1
  %12 = getelementptr inbounds i8, ptr %self, i32 4
  %13 = getelementptr inbounds i8, ptr %self, i32 4
  %14 = load i32, ptr %13, align 4
  %15 = add i32 %14, 1
  store i32 %15, ptr %12, align 4
  ret void
}

; proc_macro::bridge::client::run_client
; Function Attrs: nounwind
define internal void @_ZN10proc_macro6bridge6client10run_client17h05f2e6744ce9eab5E(ptr sret([20 x i8]) align 4 %_0, ptr align 4 %config, ptr align 1 %f) unnamed_addr #0 {
start:
  %0 = alloca [4 x i8], align 4
  %op = alloca [4 x i8], align 4
  %_30 = alloca [0 x i8], align 1
  %_28 = alloca [12 x i8], align 4
  %e = alloca [12 x i8], align 4
  %_24 = alloca [12 x i8], align 4
  %_15 = alloca [20 x i8], align 4
  %data = alloca [20 x i8], align 4
  %_12 = alloca [4 x i8], align 4
  %_9 = alloca [20 x i8], align 4
  %f2 = alloca [20 x i8], align 4
  %self1 = alloca [8 x i8], align 4
  %self = alloca [12 x i8], align 4
  %force_show_panics = alloca [1 x i8], align 1
  %buf = alloca [20 x i8], align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %buf, ptr align 4 %config, i32 20, i1 false)
  %1 = getelementptr inbounds i8, ptr %config, i32 20
  %dispatch.0 = load ptr, ptr %1, align 4
  %2 = getelementptr inbounds i8, ptr %1, i32 4
  %dispatch.1 = load ptr, ptr %2, align 4
  %3 = getelementptr inbounds i8, ptr %config, i32 28
  %4 = load i8, ptr %3, align 4
  %5 = trunc i8 %4 to i1
  %6 = zext i1 %5 to i8
  store i8 %6, ptr %force_show_panics, align 1
  store ptr %force_show_panics, ptr %_9, align 4
  %7 = getelementptr inbounds i8, ptr %_9, i32 4
  store ptr %buf, ptr %7, align 4
  %8 = getelementptr inbounds i8, ptr %_9, i32 8
  store ptr %dispatch.0, ptr %8, align 4
  %9 = getelementptr inbounds i8, ptr %8, i32 4
  store ptr %dispatch.1, ptr %9, align 4
  %10 = getelementptr inbounds i8, ptr %_9, i32 16
  store ptr %f, ptr %10, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %f2, ptr align 4 %_9, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_15, ptr align 4 %f2, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %data, ptr align 4 %_15, i32 20, i1 false)
; call std::panicking::try::do_call
  call void @_ZN3std9panicking3try7do_call17h945ff2ec531320c7E(ptr %data)
  store i32 0, ptr %0, align 4
  %_18 = load i32, ptr %0, align 4
  %11 = icmp eq i32 %_18, 0
  br i1 %11, label %bb3, label %bb4

bb3:                                              ; preds = %start
  %12 = load ptr, ptr @0, align 4
  %13 = load ptr, ptr getelementptr inbounds (i8, ptr @0, i32 4), align 4
  store ptr %12, ptr %self1, align 4
  %14 = getelementptr inbounds i8, ptr %self1, i32 4
  store ptr %13, ptr %14, align 4
  store i32 -2147483645, ptr %self, align 4
  store ptr %buf, ptr %_12, align 4
  %15 = load ptr, ptr %_12, align 4
  store ptr %15, ptr %op, align 4
  br label %bb6

bb4:                                              ; preds = %start
  %slot.0 = load ptr, ptr %data, align 4
  %16 = getelementptr inbounds i8, ptr %data, i32 4
  %slot.1 = load ptr, ptr %16, align 4
  store ptr %slot.0, ptr %self1, align 4
  %17 = getelementptr inbounds i8, ptr %self1, i32 4
  store ptr %slot.1, ptr %17, align 4
  %e.0 = load ptr, ptr %self1, align 4
  %18 = getelementptr inbounds i8, ptr %self1, i32 4
  %e.1 = load ptr, ptr %18, align 4
; call core::ops::function::FnOnce::call_once
  call void @_ZN4core3ops8function6FnOnce9call_once17hf04ff4d2515de3eeE(ptr sret([12 x i8]) align 4 %_24, ptr align 1 %e.0, ptr align 4 %e.1) #5
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self, ptr align 4 %_24, i32 12, i1 false)
  store ptr %buf, ptr %_12, align 4
  %19 = load ptr, ptr %_12, align 4
  store ptr %19, ptr %op, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %e, ptr align 4 %self, i32 12, i1 false)
  %_32 = load ptr, ptr %_12, align 4
  %self3 = load ptr, ptr %_12, align 4
  %20 = load ptr, ptr %_12, align 4
  %21 = getelementptr inbounds i8, ptr %20, i32 4
  store i32 0, ptr %21, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_28, ptr align 4 %e, i32 12, i1 false)
  %22 = load ptr, ptr %_12, align 4
; call proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::result::Result<T,E>>::encode
  call void @"_ZN10proc_macro6bridge104_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..result..Result$LT$T$C$E$GT$$GT$6encode17he747a828f2cb7c4dE"(ptr align 4 %_28, ptr align 4 %22, ptr align 1 %_30) #5
  br label %bb6

bb6:                                              ; preds = %bb4, %bb3
; call proc_macro::bridge::symbol::Symbol::invalidate_all
  call void @_ZN10proc_macro6bridge6symbol6Symbol14invalidate_all17h8539ab76ea2c91c0E() #5
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_0, ptr align 4 %buf, i32 20, i1 false)
  ret void
}

; proc_macro::bridge::client::run_client
; Function Attrs: nounwind
define internal void @_ZN10proc_macro6bridge6client10run_client17h23f9ae458dcb8346E(ptr sret([20 x i8]) align 4 %_0, ptr align 4 %config, ptr align 1 %f) unnamed_addr #0 {
start:
  %0 = alloca [4 x i8], align 4
  %op = alloca [4 x i8], align 4
  %_30 = alloca [0 x i8], align 1
  %_28 = alloca [12 x i8], align 4
  %e = alloca [12 x i8], align 4
  %_24 = alloca [12 x i8], align 4
  %_15 = alloca [20 x i8], align 4
  %data = alloca [20 x i8], align 4
  %_12 = alloca [4 x i8], align 4
  %_9 = alloca [20 x i8], align 4
  %f2 = alloca [20 x i8], align 4
  %self1 = alloca [8 x i8], align 4
  %self = alloca [12 x i8], align 4
  %force_show_panics = alloca [1 x i8], align 1
  %buf = alloca [20 x i8], align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %buf, ptr align 4 %config, i32 20, i1 false)
  %1 = getelementptr inbounds i8, ptr %config, i32 20
  %dispatch.0 = load ptr, ptr %1, align 4
  %2 = getelementptr inbounds i8, ptr %1, i32 4
  %dispatch.1 = load ptr, ptr %2, align 4
  %3 = getelementptr inbounds i8, ptr %config, i32 28
  %4 = load i8, ptr %3, align 4
  %5 = trunc i8 %4 to i1
  %6 = zext i1 %5 to i8
  store i8 %6, ptr %force_show_panics, align 1
  store ptr %force_show_panics, ptr %_9, align 4
  %7 = getelementptr inbounds i8, ptr %_9, i32 4
  store ptr %buf, ptr %7, align 4
  %8 = getelementptr inbounds i8, ptr %_9, i32 8
  store ptr %dispatch.0, ptr %8, align 4
  %9 = getelementptr inbounds i8, ptr %8, i32 4
  store ptr %dispatch.1, ptr %9, align 4
  %10 = getelementptr inbounds i8, ptr %_9, i32 16
  store ptr %f, ptr %10, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %f2, ptr align 4 %_9, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_15, ptr align 4 %f2, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %data, ptr align 4 %_15, i32 20, i1 false)
; call std::panicking::try::do_call
  call void @_ZN3std9panicking3try7do_call17h5f365bba4715d707E(ptr %data)
  store i32 0, ptr %0, align 4
  %_18 = load i32, ptr %0, align 4
  %11 = icmp eq i32 %_18, 0
  br i1 %11, label %bb3, label %bb4

bb3:                                              ; preds = %start
  %12 = load ptr, ptr @0, align 4
  %13 = load ptr, ptr getelementptr inbounds (i8, ptr @0, i32 4), align 4
  store ptr %12, ptr %self1, align 4
  %14 = getelementptr inbounds i8, ptr %self1, i32 4
  store ptr %13, ptr %14, align 4
  store i32 -2147483645, ptr %self, align 4
  store ptr %buf, ptr %_12, align 4
  %15 = load ptr, ptr %_12, align 4
  store ptr %15, ptr %op, align 4
  br label %bb6

bb4:                                              ; preds = %start
  %slot.0 = load ptr, ptr %data, align 4
  %16 = getelementptr inbounds i8, ptr %data, i32 4
  %slot.1 = load ptr, ptr %16, align 4
  store ptr %slot.0, ptr %self1, align 4
  %17 = getelementptr inbounds i8, ptr %self1, i32 4
  store ptr %slot.1, ptr %17, align 4
  %e.0 = load ptr, ptr %self1, align 4
  %18 = getelementptr inbounds i8, ptr %self1, i32 4
  %e.1 = load ptr, ptr %18, align 4
; call core::ops::function::FnOnce::call_once
  call void @_ZN4core3ops8function6FnOnce9call_once17hf04ff4d2515de3eeE(ptr sret([12 x i8]) align 4 %_24, ptr align 1 %e.0, ptr align 4 %e.1) #5
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self, ptr align 4 %_24, i32 12, i1 false)
  store ptr %buf, ptr %_12, align 4
  %19 = load ptr, ptr %_12, align 4
  store ptr %19, ptr %op, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %e, ptr align 4 %self, i32 12, i1 false)
  %_32 = load ptr, ptr %_12, align 4
  %self3 = load ptr, ptr %_12, align 4
  %20 = load ptr, ptr %_12, align 4
  %21 = getelementptr inbounds i8, ptr %20, i32 4
  store i32 0, ptr %21, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_28, ptr align 4 %e, i32 12, i1 false)
  %22 = load ptr, ptr %_12, align 4
; call proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::result::Result<T,E>>::encode
  call void @"_ZN10proc_macro6bridge104_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..result..Result$LT$T$C$E$GT$$GT$6encode17he747a828f2cb7c4dE"(ptr align 4 %_28, ptr align 4 %22, ptr align 1 %_30) #5
  br label %bb6

bb6:                                              ; preds = %bb4, %bb3
; call proc_macro::bridge::symbol::Symbol::invalidate_all
  call void @_ZN10proc_macro6bridge6symbol6Symbol14invalidate_all17h8539ab76ea2c91c0E() #5
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_0, ptr align 4 %buf, i32 20, i1 false)
  ret void
}

; proc_macro::bridge::client::run_client
; Function Attrs: nounwind
define internal void @_ZN10proc_macro6bridge6client10run_client17h41a6c15a2bc804bcE(ptr sret([20 x i8]) align 4 %_0, ptr align 4 %config, ptr align 1 %f) unnamed_addr #0 {
start:
  %0 = alloca [4 x i8], align 4
  %op = alloca [4 x i8], align 4
  %_30 = alloca [0 x i8], align 1
  %_28 = alloca [12 x i8], align 4
  %e = alloca [12 x i8], align 4
  %_24 = alloca [12 x i8], align 4
  %_15 = alloca [20 x i8], align 4
  %data = alloca [20 x i8], align 4
  %_12 = alloca [4 x i8], align 4
  %_9 = alloca [20 x i8], align 4
  %f2 = alloca [20 x i8], align 4
  %self1 = alloca [8 x i8], align 4
  %self = alloca [12 x i8], align 4
  %force_show_panics = alloca [1 x i8], align 1
  %buf = alloca [20 x i8], align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %buf, ptr align 4 %config, i32 20, i1 false)
  %1 = getelementptr inbounds i8, ptr %config, i32 20
  %dispatch.0 = load ptr, ptr %1, align 4
  %2 = getelementptr inbounds i8, ptr %1, i32 4
  %dispatch.1 = load ptr, ptr %2, align 4
  %3 = getelementptr inbounds i8, ptr %config, i32 28
  %4 = load i8, ptr %3, align 4
  %5 = trunc i8 %4 to i1
  %6 = zext i1 %5 to i8
  store i8 %6, ptr %force_show_panics, align 1
  store ptr %force_show_panics, ptr %_9, align 4
  %7 = getelementptr inbounds i8, ptr %_9, i32 4
  store ptr %buf, ptr %7, align 4
  %8 = getelementptr inbounds i8, ptr %_9, i32 8
  store ptr %dispatch.0, ptr %8, align 4
  %9 = getelementptr inbounds i8, ptr %8, i32 4
  store ptr %dispatch.1, ptr %9, align 4
  %10 = getelementptr inbounds i8, ptr %_9, i32 16
  store ptr %f, ptr %10, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %f2, ptr align 4 %_9, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_15, ptr align 4 %f2, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %data, ptr align 4 %_15, i32 20, i1 false)
; call std::panicking::try::do_call
  call void @_ZN3std9panicking3try7do_call17h1989b920025453d5E(ptr %data)
  store i32 0, ptr %0, align 4
  %_18 = load i32, ptr %0, align 4
  %11 = icmp eq i32 %_18, 0
  br i1 %11, label %bb3, label %bb4

bb3:                                              ; preds = %start
  %12 = load ptr, ptr @0, align 4
  %13 = load ptr, ptr getelementptr inbounds (i8, ptr @0, i32 4), align 4
  store ptr %12, ptr %self1, align 4
  %14 = getelementptr inbounds i8, ptr %self1, i32 4
  store ptr %13, ptr %14, align 4
  store i32 -2147483645, ptr %self, align 4
  store ptr %buf, ptr %_12, align 4
  %15 = load ptr, ptr %_12, align 4
  store ptr %15, ptr %op, align 4
  br label %bb6

bb4:                                              ; preds = %start
  %slot.0 = load ptr, ptr %data, align 4
  %16 = getelementptr inbounds i8, ptr %data, i32 4
  %slot.1 = load ptr, ptr %16, align 4
  store ptr %slot.0, ptr %self1, align 4
  %17 = getelementptr inbounds i8, ptr %self1, i32 4
  store ptr %slot.1, ptr %17, align 4
  %e.0 = load ptr, ptr %self1, align 4
  %18 = getelementptr inbounds i8, ptr %self1, i32 4
  %e.1 = load ptr, ptr %18, align 4
; call core::ops::function::FnOnce::call_once
  call void @_ZN4core3ops8function6FnOnce9call_once17hf04ff4d2515de3eeE(ptr sret([12 x i8]) align 4 %_24, ptr align 1 %e.0, ptr align 4 %e.1) #5
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self, ptr align 4 %_24, i32 12, i1 false)
  store ptr %buf, ptr %_12, align 4
  %19 = load ptr, ptr %_12, align 4
  store ptr %19, ptr %op, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %e, ptr align 4 %self, i32 12, i1 false)
  %_32 = load ptr, ptr %_12, align 4
  %self3 = load ptr, ptr %_12, align 4
  %20 = load ptr, ptr %_12, align 4
  %21 = getelementptr inbounds i8, ptr %20, i32 4
  store i32 0, ptr %21, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_28, ptr align 4 %e, i32 12, i1 false)
  %22 = load ptr, ptr %_12, align 4
; call proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::result::Result<T,E>>::encode
  call void @"_ZN10proc_macro6bridge104_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..result..Result$LT$T$C$E$GT$$GT$6encode17he747a828f2cb7c4dE"(ptr align 4 %_28, ptr align 4 %22, ptr align 1 %_30) #5
  br label %bb6

bb6:                                              ; preds = %bb4, %bb3
; call proc_macro::bridge::symbol::Symbol::invalidate_all
  call void @_ZN10proc_macro6bridge6symbol6Symbol14invalidate_all17h8539ab76ea2c91c0E() #5
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_0, ptr align 4 %buf, i32 20, i1 false)
  ret void
}

; proc_macro::bridge::client::run_client::{{closure}}
; Function Attrs: inlinehint nounwind
define internal void @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$17h6609405f329e584bE"(ptr align 4 %_1) unnamed_addr #1 {
start:
  %self = alloca [20 x i8], align 4
  %_50 = alloca [40 x i8], align 4
  %v1 = alloca [12 x i8], align 4
  %v = alloca [12 x i8], align 4
  %src = alloca [20 x i8], align 4
  %globals = alloca [12 x i8], align 4
  %_20 = alloca [0 x i8], align 1
  %_18 = alloca [8 x i8], align 4
  %result = alloca [20 x i8], align 4
  %value = alloca [40 x i8], align 4
  %state = alloca [44 x i8], align 4
  %_9 = alloca [0 x i8], align 1
  %_6 = alloca [8 x i8], align 4
  %_21 = load ptr, ptr %_1, align 4
  %0 = load i8, ptr %_21, align 1
  %_3 = trunc i8 %0 to i1
; call proc_macro::bridge::client::maybe_install_panic_hook
  call void @_ZN10proc_macro6bridge6client24maybe_install_panic_hook17h5c87af20ddbf148aE(i1 zeroext %_3) #5
; call proc_macro::bridge::symbol::Symbol::invalidate_all
  call void @_ZN10proc_macro6bridge6symbol6Symbol14invalidate_all17h8539ab76ea2c91c0E() #5
  %1 = getelementptr inbounds i8, ptr %_1, i32 4
  %self2 = load ptr, ptr %1, align 4
  %_24 = load ptr, ptr %self2, align 4
  %2 = getelementptr inbounds i8, ptr %self2, i32 4
  %len = load i32, ptr %2, align 4
  store ptr %_24, ptr %_6, align 4
  %3 = getelementptr inbounds i8, ptr %_6, i32 4
  store i32 %len, ptr %3, align 4
; call <proc_macro::bridge::ExpnGlobals<Span> as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  call void @"_ZN107_$LT$proc_macro..bridge..ExpnGlobals$LT$Span$GT$$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17hddbe6a95510ab15eE"(ptr sret([12 x i8]) align 4 %globals, ptr align 4 %_6, ptr align 1 %_9) #5
; call <proc_macro::bridge::client::TokenStream as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  %input = call i32 @"_ZN103_$LT$proc_macro..bridge..client..TokenStream$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17h9cde09496fe5af2dE"(ptr align 4 %_6, ptr align 1 %_9) #5
  store i32 0, ptr %v, align 4
  %4 = getelementptr inbounds i8, ptr %v, i32 4
  store ptr inttoptr (i32 1 to ptr), ptr %4, align 4
  %5 = getelementptr inbounds i8, ptr %v, i32 8
  store i32 0, ptr %5, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %v1, ptr align 4 %v, i32 12, i1 false)
  %6 = getelementptr inbounds i8, ptr %v1, i32 4
  %self3 = load ptr, ptr %6, align 4
  %7 = getelementptr inbounds i8, ptr %v1, i32 8
  %len4 = load i32, ptr %7, align 4
  %capacity = load i32, ptr %v1, align 4
  store ptr %self3, ptr %src, align 4
  %8 = getelementptr inbounds i8, ptr %src, i32 4
  store i32 %len4, ptr %8, align 4
  %9 = getelementptr inbounds i8, ptr %src, i32 8
  store i32 %capacity, ptr %9, align 4
  %10 = getelementptr inbounds i8, ptr %src, i32 12
  store ptr @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from7reserve17h7f50c2c3b7d04034E", ptr %10, align 4
  %11 = getelementptr inbounds i8, ptr %src, i32 16
  store ptr @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from4drop17h97d84a4094b82bf1E", ptr %11, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %result, ptr align 4 %self2, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self2, ptr align 4 %src, i32 20, i1 false)
  %12 = getelementptr inbounds i8, ptr %_1, i32 8
  %_13.0 = load ptr, ptr %12, align 4
  %13 = getelementptr inbounds i8, ptr %12, i32 4
  %_13.1 = load ptr, ptr %13, align 4
  %14 = getelementptr inbounds i8, ptr %value, i32 20
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %14, ptr align 4 %result, i32 20, i1 false)
  store ptr %_13.0, ptr %value, align 4
  %15 = getelementptr inbounds i8, ptr %value, i32 4
  store ptr %_13.1, ptr %15, align 4
  %16 = getelementptr inbounds i8, ptr %value, i32 8
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %16, ptr align 4 %globals, i32 12, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_50, ptr align 4 %value, i32 40, i1 false)
  store i32 0, ptr %state, align 4
  %17 = getelementptr inbounds i8, ptr %state, i32 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %17, ptr align 4 %_50, i32 40, i1 false)
  %18 = getelementptr inbounds i8, ptr %_1, i32 16
  %_16.0 = load ptr, ptr %18, align 4
; call proc_macro::bridge::client::state::set
  %output = call i32 @_ZN10proc_macro6bridge6client5state3set17hb590b3cf3d76a392E(ptr align 4 %state, ptr align 1 %_16.0, i32 %input) #5
  %self5 = load i32, ptr %state, align 4
  %19 = getelementptr inbounds i8, ptr %state, i32 4
  %20 = getelementptr inbounds i8, ptr %19, i32 20
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self, ptr align 4 %20, i32 20, i1 false)
  %21 = getelementptr inbounds i8, ptr %state, i32 4
  %self6 = load ptr, ptr %21, align 4
  %22 = getelementptr inbounds i8, ptr %state, i32 4
  %23 = getelementptr inbounds i8, ptr %22, i32 4
  %self7 = load ptr, ptr %23, align 4
  %24 = getelementptr inbounds i8, ptr %state, i32 4
  %25 = getelementptr inbounds i8, ptr %24, i32 8
  %self8 = load i32, ptr %25, align 4
  %26 = getelementptr inbounds i8, ptr %state, i32 4
  %27 = getelementptr inbounds i8, ptr %26, i32 8
  %28 = getelementptr inbounds i8, ptr %27, i32 4
  %self9 = load i32, ptr %28, align 4
  %29 = getelementptr inbounds i8, ptr %state, i32 4
  %30 = getelementptr inbounds i8, ptr %29, i32 8
  %31 = getelementptr inbounds i8, ptr %30, i32 8
  %self10 = load i32, ptr %31, align 4
; call core::ptr::drop_in_place<proc_macro::bridge::buffer::Buffer>
  call void @"_ZN4core3ptr55drop_in_place$LT$proc_macro..bridge..buffer..Buffer$GT$17h24f1b8cbac89e26dE"(ptr align 4 %self2) #5
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self2, ptr align 4 %self, i32 20, i1 false)
  %32 = getelementptr inbounds i8, ptr %self2, i32 4
  store i32 0, ptr %32, align 4
  %33 = getelementptr inbounds i8, ptr %_18, i32 4
  store i32 %output, ptr %33, align 4
  store i32 0, ptr %_18, align 4
  %34 = load i32, ptr %_18, align 4
  %35 = getelementptr inbounds i8, ptr %_18, i32 4
  %36 = load i32, ptr %35, align 4
; call proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::result::Result<T,E>>::encode
  call void @"_ZN10proc_macro6bridge104_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..result..Result$LT$T$C$E$GT$$GT$6encode17ha09c881eb3267a40E"(i32 %34, i32 %36, ptr align 4 %self2, ptr align 1 %_20) #5
  ret void
}

; proc_macro::bridge::client::run_client::{{closure}}
; Function Attrs: inlinehint nounwind
define internal void @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$17ha12683d31bb82082E"(ptr align 4 %_1) unnamed_addr #1 {
start:
  %self = alloca [20 x i8], align 4
  %_50 = alloca [40 x i8], align 4
  %v1 = alloca [12 x i8], align 4
  %v = alloca [12 x i8], align 4
  %src = alloca [20 x i8], align 4
  %globals = alloca [12 x i8], align 4
  %_20 = alloca [0 x i8], align 1
  %_18 = alloca [8 x i8], align 4
  %result = alloca [20 x i8], align 4
  %value = alloca [40 x i8], align 4
  %state = alloca [44 x i8], align 4
  %_9 = alloca [0 x i8], align 1
  %_6 = alloca [8 x i8], align 4
  %_21 = load ptr, ptr %_1, align 4
  %0 = load i8, ptr %_21, align 1
  %_3 = trunc i8 %0 to i1
; call proc_macro::bridge::client::maybe_install_panic_hook
  call void @_ZN10proc_macro6bridge6client24maybe_install_panic_hook17h5c87af20ddbf148aE(i1 zeroext %_3) #5
; call proc_macro::bridge::symbol::Symbol::invalidate_all
  call void @_ZN10proc_macro6bridge6symbol6Symbol14invalidate_all17h8539ab76ea2c91c0E() #5
  %1 = getelementptr inbounds i8, ptr %_1, i32 4
  %self2 = load ptr, ptr %1, align 4
  %_24 = load ptr, ptr %self2, align 4
  %2 = getelementptr inbounds i8, ptr %self2, i32 4
  %len = load i32, ptr %2, align 4
  store ptr %_24, ptr %_6, align 4
  %3 = getelementptr inbounds i8, ptr %_6, i32 4
  store i32 %len, ptr %3, align 4
; call <proc_macro::bridge::ExpnGlobals<Span> as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  call void @"_ZN107_$LT$proc_macro..bridge..ExpnGlobals$LT$Span$GT$$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17hddbe6a95510ab15eE"(ptr sret([12 x i8]) align 4 %globals, ptr align 4 %_6, ptr align 1 %_9) #5
; call <proc_macro::bridge::client::TokenStream as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  %input = call i32 @"_ZN103_$LT$proc_macro..bridge..client..TokenStream$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17h9cde09496fe5af2dE"(ptr align 4 %_6, ptr align 1 %_9) #5
  store i32 0, ptr %v, align 4
  %4 = getelementptr inbounds i8, ptr %v, i32 4
  store ptr inttoptr (i32 1 to ptr), ptr %4, align 4
  %5 = getelementptr inbounds i8, ptr %v, i32 8
  store i32 0, ptr %5, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %v1, ptr align 4 %v, i32 12, i1 false)
  %6 = getelementptr inbounds i8, ptr %v1, i32 4
  %self3 = load ptr, ptr %6, align 4
  %7 = getelementptr inbounds i8, ptr %v1, i32 8
  %len4 = load i32, ptr %7, align 4
  %capacity = load i32, ptr %v1, align 4
  store ptr %self3, ptr %src, align 4
  %8 = getelementptr inbounds i8, ptr %src, i32 4
  store i32 %len4, ptr %8, align 4
  %9 = getelementptr inbounds i8, ptr %src, i32 8
  store i32 %capacity, ptr %9, align 4
  %10 = getelementptr inbounds i8, ptr %src, i32 12
  store ptr @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from7reserve17h7f50c2c3b7d04034E", ptr %10, align 4
  %11 = getelementptr inbounds i8, ptr %src, i32 16
  store ptr @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from4drop17h97d84a4094b82bf1E", ptr %11, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %result, ptr align 4 %self2, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self2, ptr align 4 %src, i32 20, i1 false)
  %12 = getelementptr inbounds i8, ptr %_1, i32 8
  %_13.0 = load ptr, ptr %12, align 4
  %13 = getelementptr inbounds i8, ptr %12, i32 4
  %_13.1 = load ptr, ptr %13, align 4
  %14 = getelementptr inbounds i8, ptr %value, i32 20
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %14, ptr align 4 %result, i32 20, i1 false)
  store ptr %_13.0, ptr %value, align 4
  %15 = getelementptr inbounds i8, ptr %value, i32 4
  store ptr %_13.1, ptr %15, align 4
  %16 = getelementptr inbounds i8, ptr %value, i32 8
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %16, ptr align 4 %globals, i32 12, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_50, ptr align 4 %value, i32 40, i1 false)
  store i32 0, ptr %state, align 4
  %17 = getelementptr inbounds i8, ptr %state, i32 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %17, ptr align 4 %_50, i32 40, i1 false)
  %18 = getelementptr inbounds i8, ptr %_1, i32 16
  %_16.0 = load ptr, ptr %18, align 4
; call proc_macro::bridge::client::state::set
  %output = call i32 @_ZN10proc_macro6bridge6client5state3set17hf369f718f964c4baE(ptr align 4 %state, ptr align 1 %_16.0, i32 %input) #5
  %self5 = load i32, ptr %state, align 4
  %19 = getelementptr inbounds i8, ptr %state, i32 4
  %20 = getelementptr inbounds i8, ptr %19, i32 20
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self, ptr align 4 %20, i32 20, i1 false)
  %21 = getelementptr inbounds i8, ptr %state, i32 4
  %self6 = load ptr, ptr %21, align 4
  %22 = getelementptr inbounds i8, ptr %state, i32 4
  %23 = getelementptr inbounds i8, ptr %22, i32 4
  %self7 = load ptr, ptr %23, align 4
  %24 = getelementptr inbounds i8, ptr %state, i32 4
  %25 = getelementptr inbounds i8, ptr %24, i32 8
  %self8 = load i32, ptr %25, align 4
  %26 = getelementptr inbounds i8, ptr %state, i32 4
  %27 = getelementptr inbounds i8, ptr %26, i32 8
  %28 = getelementptr inbounds i8, ptr %27, i32 4
  %self9 = load i32, ptr %28, align 4
  %29 = getelementptr inbounds i8, ptr %state, i32 4
  %30 = getelementptr inbounds i8, ptr %29, i32 8
  %31 = getelementptr inbounds i8, ptr %30, i32 8
  %self10 = load i32, ptr %31, align 4
; call core::ptr::drop_in_place<proc_macro::bridge::buffer::Buffer>
  call void @"_ZN4core3ptr55drop_in_place$LT$proc_macro..bridge..buffer..Buffer$GT$17h24f1b8cbac89e26dE"(ptr align 4 %self2) #5
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self2, ptr align 4 %self, i32 20, i1 false)
  %32 = getelementptr inbounds i8, ptr %self2, i32 4
  store i32 0, ptr %32, align 4
  %33 = getelementptr inbounds i8, ptr %_18, i32 4
  store i32 %output, ptr %33, align 4
  store i32 0, ptr %_18, align 4
  %34 = load i32, ptr %_18, align 4
  %35 = getelementptr inbounds i8, ptr %_18, i32 4
  %36 = load i32, ptr %35, align 4
; call proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::result::Result<T,E>>::encode
  call void @"_ZN10proc_macro6bridge104_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..result..Result$LT$T$C$E$GT$$GT$6encode17ha09c881eb3267a40E"(i32 %34, i32 %36, ptr align 4 %self2, ptr align 1 %_20) #5
  ret void
}

; proc_macro::bridge::client::run_client::{{closure}}
; Function Attrs: inlinehint nounwind
define internal void @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$17hf249c20d06de99baE"(ptr align 4 %_1) unnamed_addr #1 {
start:
  %self = alloca [20 x i8], align 4
  %_50 = alloca [40 x i8], align 4
  %v1 = alloca [12 x i8], align 4
  %v = alloca [12 x i8], align 4
  %src = alloca [20 x i8], align 4
  %globals = alloca [12 x i8], align 4
  %_20 = alloca [0 x i8], align 1
  %_18 = alloca [8 x i8], align 4
  %_16 = alloca [12 x i8], align 4
  %result = alloca [20 x i8], align 4
  %value = alloca [40 x i8], align 4
  %state = alloca [44 x i8], align 4
  %_9 = alloca [0 x i8], align 1
  %_6 = alloca [8 x i8], align 4
  %_21 = load ptr, ptr %_1, align 4
  %0 = load i8, ptr %_21, align 1
  %_3 = trunc i8 %0 to i1
; call proc_macro::bridge::client::maybe_install_panic_hook
  call void @_ZN10proc_macro6bridge6client24maybe_install_panic_hook17h5c87af20ddbf148aE(i1 zeroext %_3) #5
; call proc_macro::bridge::symbol::Symbol::invalidate_all
  call void @_ZN10proc_macro6bridge6symbol6Symbol14invalidate_all17h8539ab76ea2c91c0E() #5
  %1 = getelementptr inbounds i8, ptr %_1, i32 4
  %self2 = load ptr, ptr %1, align 4
  %_24 = load ptr, ptr %self2, align 4
  %2 = getelementptr inbounds i8, ptr %self2, i32 4
  %len = load i32, ptr %2, align 4
  store ptr %_24, ptr %_6, align 4
  %3 = getelementptr inbounds i8, ptr %_6, i32 4
  store i32 %len, ptr %3, align 4
; call <proc_macro::bridge::ExpnGlobals<Span> as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  call void @"_ZN107_$LT$proc_macro..bridge..ExpnGlobals$LT$Span$GT$$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17hddbe6a95510ab15eE"(ptr sret([12 x i8]) align 4 %globals, ptr align 4 %_6, ptr align 1 %_9) #5
; call <(A,B) as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  %4 = call { i32, i32 } @"_ZN77_$LT$$LP$A$C$B$RP$$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17h778a3ee6399de895E"(ptr align 4 %_6, ptr align 1 %_9) #5
  %input.0 = extractvalue { i32, i32 } %4, 0
  %input.1 = extractvalue { i32, i32 } %4, 1
  store i32 0, ptr %v, align 4
  %5 = getelementptr inbounds i8, ptr %v, i32 4
  store ptr inttoptr (i32 1 to ptr), ptr %5, align 4
  %6 = getelementptr inbounds i8, ptr %v, i32 8
  store i32 0, ptr %6, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %v1, ptr align 4 %v, i32 12, i1 false)
  %7 = getelementptr inbounds i8, ptr %v1, i32 4
  %self3 = load ptr, ptr %7, align 4
  %8 = getelementptr inbounds i8, ptr %v1, i32 8
  %len4 = load i32, ptr %8, align 4
  %capacity = load i32, ptr %v1, align 4
  store ptr %self3, ptr %src, align 4
  %9 = getelementptr inbounds i8, ptr %src, i32 4
  store i32 %len4, ptr %9, align 4
  %10 = getelementptr inbounds i8, ptr %src, i32 8
  store i32 %capacity, ptr %10, align 4
  %11 = getelementptr inbounds i8, ptr %src, i32 12
  store ptr @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from7reserve17h7f50c2c3b7d04034E", ptr %11, align 4
  %12 = getelementptr inbounds i8, ptr %src, i32 16
  store ptr @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from4drop17h97d84a4094b82bf1E", ptr %12, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %result, ptr align 4 %self2, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self2, ptr align 4 %src, i32 20, i1 false)
  %13 = getelementptr inbounds i8, ptr %_1, i32 8
  %_13.0 = load ptr, ptr %13, align 4
  %14 = getelementptr inbounds i8, ptr %13, i32 4
  %_13.1 = load ptr, ptr %14, align 4
  %15 = getelementptr inbounds i8, ptr %value, i32 20
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %15, ptr align 4 %result, i32 20, i1 false)
  store ptr %_13.0, ptr %value, align 4
  %16 = getelementptr inbounds i8, ptr %value, i32 4
  store ptr %_13.1, ptr %16, align 4
  %17 = getelementptr inbounds i8, ptr %value, i32 8
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %17, ptr align 4 %globals, i32 12, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_50, ptr align 4 %value, i32 40, i1 false)
  store i32 0, ptr %state, align 4
  %18 = getelementptr inbounds i8, ptr %state, i32 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %18, ptr align 4 %_50, i32 40, i1 false)
  %19 = getelementptr inbounds i8, ptr %_1, i32 16
  %20 = load ptr, ptr %19, align 4
  store ptr %20, ptr %_16, align 4
  %21 = getelementptr inbounds i8, ptr %_16, i32 4
  store i32 %input.0, ptr %21, align 4
  %22 = getelementptr inbounds i8, ptr %21, i32 4
  store i32 %input.1, ptr %22, align 4
; call proc_macro::bridge::client::state::set
  %output = call i32 @_ZN10proc_macro6bridge6client5state3set17hfc791458f0a1c2cdE(ptr align 4 %state, ptr align 4 %_16) #5
  %self5 = load i32, ptr %state, align 4
  %23 = getelementptr inbounds i8, ptr %state, i32 4
  %24 = getelementptr inbounds i8, ptr %23, i32 20
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self, ptr align 4 %24, i32 20, i1 false)
  %25 = getelementptr inbounds i8, ptr %state, i32 4
  %self6 = load ptr, ptr %25, align 4
  %26 = getelementptr inbounds i8, ptr %state, i32 4
  %27 = getelementptr inbounds i8, ptr %26, i32 4
  %self7 = load ptr, ptr %27, align 4
  %28 = getelementptr inbounds i8, ptr %state, i32 4
  %29 = getelementptr inbounds i8, ptr %28, i32 8
  %self8 = load i32, ptr %29, align 4
  %30 = getelementptr inbounds i8, ptr %state, i32 4
  %31 = getelementptr inbounds i8, ptr %30, i32 8
  %32 = getelementptr inbounds i8, ptr %31, i32 4
  %self9 = load i32, ptr %32, align 4
  %33 = getelementptr inbounds i8, ptr %state, i32 4
  %34 = getelementptr inbounds i8, ptr %33, i32 8
  %35 = getelementptr inbounds i8, ptr %34, i32 8
  %self10 = load i32, ptr %35, align 4
; call core::ptr::drop_in_place<proc_macro::bridge::buffer::Buffer>
  call void @"_ZN4core3ptr55drop_in_place$LT$proc_macro..bridge..buffer..Buffer$GT$17h24f1b8cbac89e26dE"(ptr align 4 %self2) #5
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self2, ptr align 4 %self, i32 20, i1 false)
  %36 = getelementptr inbounds i8, ptr %self2, i32 4
  store i32 0, ptr %36, align 4
  %37 = getelementptr inbounds i8, ptr %_18, i32 4
  store i32 %output, ptr %37, align 4
  store i32 0, ptr %_18, align 4
  %38 = load i32, ptr %_18, align 4
  %39 = getelementptr inbounds i8, ptr %_18, i32 4
  %40 = load i32, ptr %39, align 4
; call proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::result::Result<T,E>>::encode
  call void @"_ZN10proc_macro6bridge104_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..result..Result$LT$T$C$E$GT$$GT$6encode17ha09c881eb3267a40E"(i32 %38, i32 %40, ptr align 4 %self2, ptr align 1 %_20) #5
  ret void
}

; proc_macro::bridge::client::run_client::{{closure}}::{{closure}}
; Function Attrs: inlinehint nounwind
define internal i32 @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17h20cb425f88d0691fE"(ptr align 4 %_1) unnamed_addr #1 {
start:
  %_2 = load ptr, ptr %_1, align 4
  %0 = getelementptr inbounds i8, ptr %_1, i32 4
  %_4.0 = load i32, ptr %0, align 4
  %1 = getelementptr inbounds i8, ptr %0, i32 4
  %_4.1 = load i32, ptr %1, align 4
; call proc_macro::bridge::client::Client<(proc_macro::TokenStream,proc_macro::TokenStream),proc_macro::TokenStream>::expand2::{{closure}}::{{closure}}
  %_0 = call i32 @"_ZN10proc_macro6bridge6client97Client$LT$$LP$proc_macro..TokenStream$C$proc_macro..TokenStream$RP$$C$proc_macro..TokenStream$GT$7expand228_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17h3b135dcbaa76b128E"(ptr align 1 %_2, i32 %_4.0, i32 %_4.1) #5
  ret i32 %_0
}

; proc_macro::bridge::client::run_client::{{closure}}::{{closure}}
; Function Attrs: inlinehint nounwind
define internal i32 @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17ha5b4b40010fe8feeE"(ptr align 1 %_1.0, i32 %_1.1) unnamed_addr #1 {
start:
; call proc_macro::bridge::client::Client<proc_macro::TokenStream,proc_macro::TokenStream>::expand1::{{closure}}::{{closure}}
  %_0 = call i32 @"_ZN10proc_macro6bridge6client63Client$LT$proc_macro..TokenStream$C$proc_macro..TokenStream$GT$7expand128_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17h3c226b881b1be7bcE"(ptr align 1 %_1.0, i32 %_1.1) #5
  ret i32 %_0
}

; proc_macro::bridge::client::run_client::{{closure}}::{{closure}}
; Function Attrs: inlinehint nounwind
define internal i32 @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17hb942d567338ec537E"(ptr align 1 %_1.0, i32 %_1.1) unnamed_addr #1 {
start:
; call proc_macro::bridge::client::Client<proc_macro::TokenStream,proc_macro::TokenStream>::expand1::{{closure}}::{{closure}}
  %_0 = call i32 @"_ZN10proc_macro6bridge6client63Client$LT$proc_macro..TokenStream$C$proc_macro..TokenStream$GT$7expand128_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17ha50bb5bbfd2d68bdE"(ptr align 1 %_1.0, i32 %_1.1) #5
  ret i32 %_0
}

; proc_macro::bridge::client::state::BRIDGE_STATE::{{closure}}
; Function Attrs: inlinehint nounwind
define internal ptr @"_ZN10proc_macro6bridge6client5state12BRIDGE_STATE28_$u7b$$u7b$closure$u7d$$u7d$17h415da4bf648fcb00E"(ptr align 1 %_1, ptr align 4 %_2) unnamed_addr #1 {
start:
  ret ptr @"_ZN10proc_macro6bridge6client5state12BRIDGE_STATE28_$u7b$$u7b$closure$u7d$$u7d$3VAL17h35515071b4f0251bE"
}

; proc_macro::bridge::client::state::set
; Function Attrs: nounwind
define internal i32 @_ZN10proc_macro6bridge6client5state3set17hb590b3cf3d76a392E(ptr align 4 %state, ptr align 1 %f.0, i32 %f.1) unnamed_addr #0 {
start:
  %_restore = alloca [4 x i8], align 4
; call std::thread::local::LocalKey<T>::with
  %outer = call ptr @"_ZN3std6thread5local17LocalKey$LT$T$GT$4with17h7ccfc207e6a4b98bE"(ptr align 4 @alloc_b528b4e8f2b0a1f7e7a8d6c80f929ed7, ptr %state) #5
  store ptr %outer, ptr %_restore, align 4
; call proc_macro::bridge::client::run_client::{{closure}}::{{closure}}
  %_0 = call i32 @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17ha5b4b40010fe8feeE"(ptr align 1 %f.0, i32 %f.1) #5
; call core::ptr::drop_in_place<proc_macro::bridge::client::state::set::RestoreOnDrop>
  call void @"_ZN4core3ptr74drop_in_place$LT$proc_macro..bridge..client..state..set..RestoreOnDrop$GT$17h8dd823f67a4c01cbE"(ptr align 4 %_restore) #5
  ret i32 %_0
}

; proc_macro::bridge::client::state::set
; Function Attrs: nounwind
define internal i32 @_ZN10proc_macro6bridge6client5state3set17hf369f718f964c4baE(ptr align 4 %state, ptr align 1 %f.0, i32 %f.1) unnamed_addr #0 {
start:
  %_restore = alloca [4 x i8], align 4
; call std::thread::local::LocalKey<T>::with
  %outer = call ptr @"_ZN3std6thread5local17LocalKey$LT$T$GT$4with17h7ccfc207e6a4b98bE"(ptr align 4 @alloc_b528b4e8f2b0a1f7e7a8d6c80f929ed7, ptr %state) #5
  store ptr %outer, ptr %_restore, align 4
; call proc_macro::bridge::client::run_client::{{closure}}::{{closure}}
  %_0 = call i32 @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17hb942d567338ec537E"(ptr align 1 %f.0, i32 %f.1) #5
; call core::ptr::drop_in_place<proc_macro::bridge::client::state::set::RestoreOnDrop>
  call void @"_ZN4core3ptr74drop_in_place$LT$proc_macro..bridge..client..state..set..RestoreOnDrop$GT$17h8dd823f67a4c01cbE"(ptr align 4 %_restore) #5
  ret i32 %_0
}

; proc_macro::bridge::client::state::set
; Function Attrs: nounwind
define internal i32 @_ZN10proc_macro6bridge6client5state3set17hfc791458f0a1c2cdE(ptr align 4 %state, ptr align 4 %f) unnamed_addr #0 {
start:
  %_restore = alloca [4 x i8], align 4
; call std::thread::local::LocalKey<T>::with
  %outer = call ptr @"_ZN3std6thread5local17LocalKey$LT$T$GT$4with17h7ccfc207e6a4b98bE"(ptr align 4 @alloc_b528b4e8f2b0a1f7e7a8d6c80f929ed7, ptr %state) #5
  store ptr %outer, ptr %_restore, align 4
; call proc_macro::bridge::client::run_client::{{closure}}::{{closure}}
  %_0 = call i32 @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17h20cb425f88d0691fE"(ptr align 4 %f) #5
; call core::ptr::drop_in_place<proc_macro::bridge::client::state::set::RestoreOnDrop>
  call void @"_ZN4core3ptr74drop_in_place$LT$proc_macro..bridge..client..state..set..RestoreOnDrop$GT$17h8dd823f67a4c01cbE"(ptr align 4 %_restore) #5
  ret i32 %_0
}

; proc_macro::bridge::client::Client<proc_macro::TokenStream,proc_macro::TokenStream>::expand1::{{closure}}
; Function Attrs: inlinehint nounwind
define internal void @"_ZN10proc_macro6bridge6client63Client$LT$proc_macro..TokenStream$C$proc_macro..TokenStream$GT$7expand128_$u7b$$u7b$closure$u7d$$u7d$17h3957b62716e0ff30E"(ptr sret([20 x i8]) align 4 %_0, ptr align 1 %_1, ptr align 4 %bridge) unnamed_addr #1 {
start:
; call proc_macro::bridge::client::run_client
  call void @_ZN10proc_macro6bridge6client10run_client17h41a6c15a2bc804bcE(ptr sret([20 x i8]) align 4 %_0, ptr align 4 %bridge, ptr align 1 %_1) #5
  ret void
}

; proc_macro::bridge::client::Client<proc_macro::TokenStream,proc_macro::TokenStream>::expand1::{{closure}}
; Function Attrs: inlinehint nounwind
define internal void @"_ZN10proc_macro6bridge6client63Client$LT$proc_macro..TokenStream$C$proc_macro..TokenStream$GT$7expand128_$u7b$$u7b$closure$u7d$$u7d$17h61fe9898d7f64129E"(ptr sret([20 x i8]) align 4 %_0, ptr align 1 %_1, ptr align 4 %bridge) unnamed_addr #1 {
start:
; call proc_macro::bridge::client::run_client
  call void @_ZN10proc_macro6bridge6client10run_client17h23f9ae458dcb8346E(ptr sret([20 x i8]) align 4 %_0, ptr align 4 %bridge, ptr align 1 %_1) #5
  ret void
}

; proc_macro::bridge::client::Client<proc_macro::TokenStream,proc_macro::TokenStream>::expand1::{{closure}}::{{closure}}
; Function Attrs: inlinehint nounwind
define internal i32 @"_ZN10proc_macro6bridge6client63Client$LT$proc_macro..TokenStream$C$proc_macro..TokenStream$GT$7expand128_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17h3c226b881b1be7bcE"(ptr align 1 %_1, i32 %input) unnamed_addr #1 {
start:
  %_6 = alloca [4 x i8], align 4
  store i32 %input, ptr %_6, align 4
  %_5 = load i32, ptr %_6, align 4
; call core::ops::function::Fn::call
  %_3 = call i32 @_ZN4core3ops8function2Fn4call17hfd20348a4381e4d2E(ptr align 1 %_1, i32 %_5) #5
  ret i32 %_3
}

; proc_macro::bridge::client::Client<proc_macro::TokenStream,proc_macro::TokenStream>::expand1::{{closure}}::{{closure}}
; Function Attrs: inlinehint nounwind
define internal i32 @"_ZN10proc_macro6bridge6client63Client$LT$proc_macro..TokenStream$C$proc_macro..TokenStream$GT$7expand128_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17ha50bb5bbfd2d68bdE"(ptr align 1 %_1, i32 %input) unnamed_addr #1 {
start:
  %_6 = alloca [4 x i8], align 4
  store i32 %input, ptr %_6, align 4
  %_5 = load i32, ptr %_6, align 4
; call core::ops::function::Fn::call
  %_3 = call i32 @_ZN4core3ops8function2Fn4call17h60b53f656913ee7fE(ptr align 1 %_1, i32 %_5) #5
  ret i32 %_3
}

; proc_macro::bridge::client::Client<(proc_macro::TokenStream,proc_macro::TokenStream),proc_macro::TokenStream>::expand2::{{closure}}
; Function Attrs: inlinehint nounwind
define internal void @"_ZN10proc_macro6bridge6client97Client$LT$$LP$proc_macro..TokenStream$C$proc_macro..TokenStream$RP$$C$proc_macro..TokenStream$GT$7expand228_$u7b$$u7b$closure$u7d$$u7d$17h670d352edbf3ccdbE"(ptr sret([20 x i8]) align 4 %_0, ptr align 1 %_1, ptr align 4 %bridge) unnamed_addr #1 {
start:
; call proc_macro::bridge::client::run_client
  call void @_ZN10proc_macro6bridge6client10run_client17h05f2e6744ce9eab5E(ptr sret([20 x i8]) align 4 %_0, ptr align 4 %bridge, ptr align 1 %_1) #5
  ret void
}

; proc_macro::bridge::client::Client<(proc_macro::TokenStream,proc_macro::TokenStream),proc_macro::TokenStream>::expand2::{{closure}}::{{closure}}
; Function Attrs: inlinehint nounwind
define internal i32 @"_ZN10proc_macro6bridge6client97Client$LT$$LP$proc_macro..TokenStream$C$proc_macro..TokenStream$RP$$C$proc_macro..TokenStream$GT$7expand228_$u7b$$u7b$closure$u7d$$u7d$28_$u7b$$u7b$closure$u7d$$u7d$17h3b135dcbaa76b128E"(ptr align 1 %_1, i32 %_2.0, i32 %_2.1) unnamed_addr #1 {
start:
  %_10 = alloca [4 x i8], align 4
  %_8 = alloca [4 x i8], align 4
  store i32 %_2.0, ptr %_8, align 4
  %_7 = load i32, ptr %_8, align 4
  store i32 %_2.1, ptr %_10, align 4
  %_9 = load i32, ptr %_10, align 4
; call core::ops::function::Fn::call
  %_5 = call i32 @_ZN4core3ops8function2Fn4call17ha7522fecc39c0a72E(ptr align 1 %_1, i32 %_7, i32 %_9) #5
  ret i32 %_5
}

; <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
; Function Attrs: inlinehint nounwind
define internal void @"_ZN115_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$9call_once17h552e4c0181ae73c7E"(ptr align 4 %self) unnamed_addr #1 {
start:
  %_3 = alloca [20 x i8], align 4
  %_2 = alloca [0 x i8], align 1
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_3, ptr align 4 %self, i32 20, i1 false)
; call proc_macro::bridge::client::run_client::{{closure}}
  call void @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$17ha12683d31bb82082E"(ptr align 4 %_3) #5
  ret void
}

; <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
; Function Attrs: inlinehint nounwind
define internal void @"_ZN115_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$9call_once17h74a00adfe4a30e62E"(ptr align 4 %self) unnamed_addr #1 {
start:
  %_3 = alloca [20 x i8], align 4
  %_2 = alloca [0 x i8], align 1
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_3, ptr align 4 %self, i32 20, i1 false)
; call proc_macro::bridge::client::run_client::{{closure}}
  call void @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$17hf249c20d06de99baE"(ptr align 4 %_3) #5
  ret void
}

; <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
; Function Attrs: inlinehint nounwind
define internal void @"_ZN115_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$9call_once17ha5dbcbcb20adf254E"(ptr align 4 %self) unnamed_addr #1 {
start:
  %_3 = alloca [20 x i8], align 4
  %_2 = alloca [0 x i8], align 1
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %_3, ptr align 4 %self, i32 20, i1 false)
; call proc_macro::bridge::client::run_client::{{closure}}
  call void @"_ZN10proc_macro6bridge6client10run_client28_$u7b$$u7b$closure$u7d$$u7d$17h6609405f329e584bE"(ptr align 4 %_3) #5
  ret void
}

; std::thread::local::LocalKey<T>::with
; Function Attrs: nounwind
define internal ptr @"_ZN3std6thread5local17LocalKey$LT$T$GT$4with17h7ccfc207e6a4b98bE"(ptr align 4 %self, ptr %f) unnamed_addr #0 {
start:
  %_5 = alloca [0 x i8], align 1
  %self1 = alloca [8 x i8], align 4
; call std::thread::local::LocalKey<T>::try_with
  %0 = call { i32, ptr } @"_ZN3std6thread5local17LocalKey$LT$T$GT$8try_with17hca3fe0b8361812faE"(ptr align 4 %self, ptr %f) #5
  %1 = extractvalue { i32, ptr } %0, 0
  %2 = extractvalue { i32, ptr } %0, 1
  store i32 %1, ptr %self1, align 4
  %3 = getelementptr inbounds i8, ptr %self1, i32 4
  store ptr %2, ptr %3, align 4
  %_4 = load i32, ptr %self1, align 4
  %4 = icmp eq i32 %_4, 0
  br i1 %4, label %bb4, label %bb3

bb4:                                              ; preds = %start
  %5 = getelementptr inbounds i8, ptr %self1, i32 4
  %t = load ptr, ptr %5, align 4
  ret ptr %t

bb3:                                              ; preds = %start
; call core::result::unwrap_failed
  call void @_ZN4core6result13unwrap_failed17hc04de2441a7172b3E(ptr align 1 @alloc_2ee7ba9733a263ad3a32ba87b5ec3eae, i32 70, ptr align 1 %_5, ptr align 4 @vtable.0, ptr align 4 @alloc_e6c9f9962a69f7b89e88e08c7a757000) #6
  unreachable

bb2:                                              ; No predecessors!
  unreachable
}

; std::thread::local::LocalKey<T>::try_with
; Function Attrs: inlinehint nounwind
define internal { i32, ptr } @"_ZN3std6thread5local17LocalKey$LT$T$GT$8try_with17hca3fe0b8361812faE"(ptr align 4 %self, ptr %f) unnamed_addr #1 {
start:
  %self2 = alloca [4 x i8], align 4
  %self1 = alloca [4 x i8], align 4
  %_3 = alloca [4 x i8], align 4
  %_0 = alloca [8 x i8], align 4
  %_7 = load ptr, ptr %self, align 4
  %self3 = call ptr %_7(ptr align 4 null) #5
  %_14 = ptrtoint ptr %self3 to i32
  %0 = icmp eq i32 %_14, 0
  br i1 %0, label %bb4, label %bb5

bb4:                                              ; preds = %start
  store ptr null, ptr %self2, align 4
  store ptr null, ptr %self1, align 4
  store i32 1, ptr %_0, align 4
  br label %bb3

bb5:                                              ; preds = %start
  store ptr %self3, ptr %self2, align 4
  %v = load ptr, ptr %self2, align 4
  store ptr %v, ptr %self1, align 4
  %v4 = load ptr, ptr %self1, align 4
  store ptr %v4, ptr %_3, align 4
  %thread_local = load ptr, ptr %_3, align 4
; call std::thread::local::LocalKey<core::cell::Cell<T>>::replace::{{closure}}
  %_9 = call ptr @"_ZN3std6thread5local41LocalKey$LT$core..cell..Cell$LT$T$GT$$GT$7replace28_$u7b$$u7b$closure$u7d$$u7d$17haad74f68232631f4E"(ptr %f, ptr align 4 %thread_local) #5
  %1 = getelementptr inbounds i8, ptr %_0, i32 4
  store ptr %_9, ptr %1, align 4
  store i32 0, ptr %_0, align 4
  br label %bb3

bb3:                                              ; preds = %bb5, %bb4
  %2 = load i32, ptr %_0, align 4
  %3 = getelementptr inbounds i8, ptr %_0, i32 4
  %4 = load ptr, ptr %3, align 4
  %5 = insertvalue { i32, ptr } poison, i32 %2, 0
  %6 = insertvalue { i32, ptr } %5, ptr %4, 1
  ret { i32, ptr } %6
}

; std::thread::local::LocalKey<core::cell::Cell<T>>::replace::{{closure}}
; Function Attrs: inlinehint nounwind
define internal ptr @"_ZN3std6thread5local41LocalKey$LT$core..cell..Cell$LT$T$GT$$GT$7replace28_$u7b$$u7b$closure$u7d$$u7d$17haad74f68232631f4E"(ptr %_1, ptr align 4 %cell) unnamed_addr #1 {
start:
  %result = load ptr, ptr %cell, align 4
  store ptr %_1, ptr %cell, align 4
  ret ptr %result
}

; std::panicking::try::do_call
; Function Attrs: inlinehint nounwind
define internal void @_ZN3std9panicking3try7do_call17h1989b920025453d5E(ptr %data) unnamed_addr #1 {
start:
  %f = alloca [20 x i8], align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %f, ptr align 4 %data, i32 20, i1 false)
; call <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
  call void @"_ZN115_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$9call_once17ha5dbcbcb20adf254E"(ptr align 4 %f) #5
  ret void
}

; std::panicking::try::do_call
; Function Attrs: inlinehint nounwind
define internal void @_ZN3std9panicking3try7do_call17h5f365bba4715d707E(ptr %data) unnamed_addr #1 {
start:
  %f = alloca [20 x i8], align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %f, ptr align 4 %data, i32 20, i1 false)
; call <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
  call void @"_ZN115_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$9call_once17h552e4c0181ae73c7E"(ptr align 4 %f) #5
  ret void
}

; std::panicking::try::do_call
; Function Attrs: inlinehint nounwind
define internal void @_ZN3std9panicking3try7do_call17h945ff2ec531320c7E(ptr %data) unnamed_addr #1 {
start:
  %f = alloca [20 x i8], align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %f, ptr align 4 %data, i32 20, i1 false)
; call <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
  call void @"_ZN115_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$9call_once17h74a00adfe4a30e62E"(ptr align 4 %f) #5
  ret void
}

; std::panicking::try::do_catch
; Function Attrs: inlinehint nounwind
define internal void @_ZN3std9panicking3try8do_catch17h89a32180197e332bE(ptr %data, ptr %payload) unnamed_addr #1 {
start:
; call std::panicking::try::cleanup
  %0 = call { ptr, ptr } @_ZN3std9panicking3try7cleanup17h8463d18afe7024c0E(ptr %payload) #5
  %obj.0 = extractvalue { ptr, ptr } %0, 0
  %obj.1 = extractvalue { ptr, ptr } %0, 1
  store ptr %obj.0, ptr %data, align 4
  %1 = getelementptr inbounds i8, ptr %data, i32 4
  store ptr %obj.1, ptr %1, align 4
  ret void
}

; std::panicking::try::do_catch
; Function Attrs: inlinehint nounwind
define internal void @_ZN3std9panicking3try8do_catch17hb568cad190f85545E(ptr %data, ptr %payload) unnamed_addr #1 {
start:
; call std::panicking::try::cleanup
  %0 = call { ptr, ptr } @_ZN3std9panicking3try7cleanup17h8463d18afe7024c0E(ptr %payload) #5
  %obj.0 = extractvalue { ptr, ptr } %0, 0
  %obj.1 = extractvalue { ptr, ptr } %0, 1
  store ptr %obj.0, ptr %data, align 4
  %1 = getelementptr inbounds i8, ptr %data, i32 4
  store ptr %obj.1, ptr %1, align 4
  ret void
}

; std::panicking::try::do_catch
; Function Attrs: inlinehint nounwind
define internal void @_ZN3std9panicking3try8do_catch17hef3f0d644fceee9eE(ptr %data, ptr %payload) unnamed_addr #1 {
start:
; call std::panicking::try::cleanup
  %0 = call { ptr, ptr } @_ZN3std9panicking3try7cleanup17h8463d18afe7024c0E(ptr %payload) #5
  %obj.0 = extractvalue { ptr, ptr } %0, 0
  %obj.1 = extractvalue { ptr, ptr } %0, 1
  store ptr %obj.0, ptr %data, align 4
  %1 = getelementptr inbounds i8, ptr %data, i32 4
  store ptr %obj.1, ptr %1, align 4
  ret void
}

; core::ops::function::Fn::call
; Function Attrs: inlinehint nounwind
define internal i32 @_ZN4core3ops8function2Fn4call17h60b53f656913ee7fE(ptr align 1 %_1, i32 %0) unnamed_addr #1 {
start:
  %_2 = alloca [4 x i8], align 4
  store i32 %0, ptr %_2, align 4
  %1 = load i32, ptr %_2, align 4
; call simple_test_macro::simple_test
  %_0 = call i32 @_ZN17simple_test_macro11simple_test17h5a5a53a270de6560E(i32 %1) #5
  ret i32 %_0
}

; core::ops::function::Fn::call
; Function Attrs: inlinehint nounwind
define internal i32 @_ZN4core3ops8function2Fn4call17ha7522fecc39c0a72E(ptr align 1 %_1, i32 %0, i32 %1) unnamed_addr #1 {
start:
  %_2 = alloca [8 x i8], align 4
  store i32 %0, ptr %_2, align 4
  %2 = getelementptr inbounds i8, ptr %_2, i32 4
  store i32 %1, ptr %2, align 4
  %3 = load i32, ptr %_2, align 4
  %4 = getelementptr inbounds i8, ptr %_2, i32 4
  %5 = load i32, ptr %4, align 4
; call simple_test_macro::simple_attr
  %_0 = call i32 @_ZN17simple_test_macro11simple_attr17hdc19acd8708e9f69E(i32 %3, i32 %5) #5
  ret i32 %_0
}

; core::ops::function::Fn::call
; Function Attrs: inlinehint nounwind
define internal i32 @_ZN4core3ops8function2Fn4call17hfd20348a4381e4d2E(ptr align 1 %_1, i32 %0) unnamed_addr #1 {
start:
  %_2 = alloca [4 x i8], align 4
  store i32 %0, ptr %_2, align 4
  %1 = load i32, ptr %_2, align 4
; call simple_test_macro::simple_bang
  %_0 = call i32 @_ZN17simple_test_macro11simple_bang17hbf3aef8ad5965e72E(i32 %1) #5
  ret i32 %_0
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint nounwind
define internal ptr @_ZN4core3ops8function6FnOnce9call_once17h724cd20ab485ddabE(ptr align 4 %0) unnamed_addr #1 {
start:
  %_2 = alloca [4 x i8], align 4
  %_1 = alloca [0 x i8], align 1
  store ptr %0, ptr %_2, align 4
  %1 = load ptr, ptr %_2, align 4
; call proc_macro::bridge::client::state::BRIDGE_STATE::{{closure}}
  %_0 = call ptr @"_ZN10proc_macro6bridge6client5state12BRIDGE_STATE28_$u7b$$u7b$closure$u7d$$u7d$17h415da4bf648fcb00E"(ptr align 1 %_1, ptr align 4 %1) #5
  ret ptr %_0
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint nounwind
define internal void @_ZN4core3ops8function6FnOnce9call_once17hf04ff4d2515de3eeE(ptr sret([12 x i8]) align 4 %_0, ptr align 1 %0, ptr align 4 %1) unnamed_addr #1 {
start:
  %_2 = alloca [8 x i8], align 4
  store ptr %0, ptr %_2, align 4
  %2 = getelementptr inbounds i8, ptr %_2, i32 4
  store ptr %1, ptr %2, align 4
  %3 = load ptr, ptr %_2, align 4
  %4 = getelementptr inbounds i8, ptr %_2, i32 4
  %5 = load ptr, ptr %4, align 4
; call <proc_macro::bridge::rpc::PanicMessage as core::convert::From<alloc::boxed::Box<dyn core::any::Any+core::marker::Send>>>::from
  call void @"_ZN155_$LT$proc_macro..bridge..rpc..PanicMessage$u20$as$u20$core..convert..From$LT$alloc..boxed..Box$LT$dyn$u20$core..any..Any$u2b$core..marker..Send$GT$$GT$$GT$4from17h1eea7de01293fee1E"(ptr sret([12 x i8]) align 4 %_0, ptr align 1 %3, ptr align 4 %5) #5
  ret void
}

; core::ptr::drop_in_place<proc_macro::LexError>
; Function Attrs: inlinehint nounwind
define internal void @"_ZN4core3ptr41drop_in_place$LT$proc_macro..LexError$GT$17h87910e5893b3dabdE"(ptr align 1 %_1) unnamed_addr #1 {
start:
  ret void
}

; core::ptr::drop_in_place<alloc::string::String>
; Function Attrs: nounwind
define internal void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h57b2015ae50065a7E"(ptr align 4 %_1) unnamed_addr #0 {
start:
; call core::ptr::drop_in_place<alloc::vec::Vec<u8>>
  call void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17h19668702caabe506E"(ptr align 4 %_1) #5
  ret void
}

; core::ptr::drop_in_place<proc_macro::TokenStream>
; Function Attrs: nounwind
define internal void @"_ZN4core3ptr44drop_in_place$LT$proc_macro..TokenStream$GT$17ha4f053ae0cdcf5e0E"(ptr align 4 %_1) unnamed_addr #0 {
start:
; call core::ptr::drop_in_place<core::option::Option<proc_macro::bridge::client::TokenStream>>
  call void @"_ZN4core3ptr88drop_in_place$LT$core..option..Option$LT$proc_macro..bridge..client..TokenStream$GT$$GT$17h9772691adb390339E"(ptr align 4 %_1) #5
  ret void
}

; core::ptr::drop_in_place<alloc::vec::Vec<u8>>
; Function Attrs: nounwind
define internal void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17h19668702caabe506E"(ptr align 4 %_1) unnamed_addr #0 {
start:
; call <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h270b2613cc3e127fE"(ptr align 4 %_1) #5
; call core::ptr::drop_in_place<alloc::raw_vec::RawVec<u8>>
  call void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17h19d889917ecbdfc5E"(ptr align 4 %_1) #5
  ret void
}

; core::ptr::drop_in_place<std::thread::local::AccessError>
; Function Attrs: inlinehint nounwind
define internal void @"_ZN4core3ptr52drop_in_place$LT$std..thread..local..AccessError$GT$17h151106fd7d38e900E"(ptr align 1 %_1) unnamed_addr #1 {
start:
  ret void
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<u8>>
; Function Attrs: nounwind
define internal void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17h19d889917ecbdfc5E"(ptr align 4 %_1) unnamed_addr #0 {
start:
; call <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h57f67b216edb4637E"(ptr align 4 %_1) #5
  ret void
}

; core::ptr::drop_in_place<proc_macro::bridge::buffer::Buffer>
; Function Attrs: nounwind
define internal void @"_ZN4core3ptr55drop_in_place$LT$proc_macro..bridge..buffer..Buffer$GT$17h24f1b8cbac89e26dE"(ptr align 4 %_1) unnamed_addr #0 {
start:
; call <proc_macro::bridge::buffer::Buffer as core::ops::drop::Drop>::drop
  call void @"_ZN76_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..ops..drop..Drop$GT$4drop17hfcaabeb5a18c3087E"(ptr align 4 %_1) #5
  ret void
}

; core::ptr::drop_in_place<proc_macro::bridge::rpc::PanicMessage>
; Function Attrs: nounwind
define internal void @"_ZN4core3ptr58drop_in_place$LT$proc_macro..bridge..rpc..PanicMessage$GT$17h0843de3e9e4d056aE"(ptr align 4 %_1) unnamed_addr #0 {
start:
  %0 = load i32, ptr %_1, align 4
  %1 = sub i32 %0, -2147483648
  %2 = icmp ule i32 %1, 2
  %_2 = select i1 %2, i32 %1, i32 1
  %3 = icmp eq i32 %_2, 1
  br i1 %3, label %bb2, label %bb1

bb2:                                              ; preds = %start
; call core::ptr::drop_in_place<alloc::string::String>
  call void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h57b2015ae50065a7E"(ptr align 4 %_1) #5
  br label %bb1

bb1:                                              ; preds = %bb2, %start
  ret void
}

; core::ptr::drop_in_place<proc_macro::bridge::client::TokenStream>
; Function Attrs: nounwind
define internal void @"_ZN4core3ptr60drop_in_place$LT$proc_macro..bridge..client..TokenStream$GT$17hc817b4bff710af79E"(ptr align 4 %_1) unnamed_addr #0 {
start:
; call <proc_macro::bridge::client::TokenStream as core::ops::drop::Drop>::drop
  call void @"_ZN81_$LT$proc_macro..bridge..client..TokenStream$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd91f8cba53cc4621E"(ptr align 4 %_1) #5
  ret void
}

; core::ptr::drop_in_place<proc_macro::bridge::client::state::set::RestoreOnDrop>
; Function Attrs: nounwind
define internal void @"_ZN4core3ptr74drop_in_place$LT$proc_macro..bridge..client..state..set..RestoreOnDrop$GT$17h8dd823f67a4c01cbE"(ptr align 4 %_1) unnamed_addr #0 {
start:
; call <proc_macro::bridge::client::state::set::RestoreOnDrop as core::ops::drop::Drop>::drop
  call void @"_ZN95_$LT$proc_macro..bridge..client..state..set..RestoreOnDrop$u20$as$u20$core..ops..drop..Drop$GT$4drop17h9279ee21d250538dE"(ptr align 4 %_1) #5
  ret void
}

; core::ptr::drop_in_place<core::option::Option<proc_macro::bridge::client::TokenStream>>
; Function Attrs: nounwind
define internal void @"_ZN4core3ptr88drop_in_place$LT$core..option..Option$LT$proc_macro..bridge..client..TokenStream$GT$$GT$17h9772691adb390339E"(ptr align 4 %_1) unnamed_addr #0 {
start:
  %0 = load i32, ptr %_1, align 4
  %1 = icmp eq i32 %0, 0
  %_2 = select i1 %1, i32 0, i32 1
  %2 = icmp eq i32 %_2, 0
  br i1 %2, label %bb1, label %bb2

bb1:                                              ; preds = %bb2, %start
  ret void

bb2:                                              ; preds = %start
; call core::ptr::drop_in_place<proc_macro::bridge::client::TokenStream>
  call void @"_ZN4core3ptr60drop_in_place$LT$proc_macro..bridge..client..TokenStream$GT$17hc817b4bff710af79E"(ptr align 4 %_1) #5
  br label %bb1
}

; core::str::<impl str>::parse
; Function Attrs: inlinehint nounwind
define internal { i32, i32 } @"_ZN4core3str21_$LT$impl$u20$str$GT$5parse17hd0580d600884e0c5E"(ptr align 1 %self.0, i32 %self.1) unnamed_addr #1 {
start:
; call <proc_macro::TokenStream as core::str::traits::FromStr>::from_str
  %0 = call { i32, i32 } @"_ZN70_$LT$proc_macro..TokenStream$u20$as$u20$core..str..traits..FromStr$GT$8from_str17hf6b6a2d5891bda9fE"(ptr align 1 %self.0, i32 %self.1) #5
  %_0.0 = extractvalue { i32, i32 } %0, 0
  %_0.1 = extractvalue { i32, i32 } %0, 1
  %1 = insertvalue { i32, i32 } poison, i32 %_0.0, 0
  %2 = insertvalue { i32, i32 } %1, i32 %_0.1, 1
  ret { i32, i32 } %2
}

; <proc_macro::LexError as core::fmt::Debug>::fmt
; Function Attrs: inlinehint nounwind
define internal zeroext i1 @"_ZN57_$LT$proc_macro..LexError$u20$as$u20$core..fmt..Debug$GT$3fmt17h528a9c6ca545d38cE"(ptr align 1 %self, ptr align 4 %f) unnamed_addr #1 {
start:
; call core::fmt::Formatter::write_str
  %_0 = call zeroext i1 @_ZN4core3fmt9Formatter9write_str17hb77f51236005fc7aE(ptr align 4 %f, ptr align 1 @alloc_ce18dc9b9ca144fde65711b7622c392e, i32 8) #5
  ret i1 %_0
}

; <() as proc_macro::bridge::rpc::Encode<S>>::encode
; Function Attrs: nounwind
define internal void @"_ZN69_$LT$$LP$$RP$$u20$as$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$GT$6encode17heb8703baf7784cd9E"(ptr align 4 %_2, ptr align 1 %_3) unnamed_addr #0 {
start:
  ret void
}

; <proc_macro::bridge::buffer::Buffer as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint nounwind
define internal void @"_ZN76_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..ops..drop..Drop$GT$4drop17hfcaabeb5a18c3087E"(ptr align 4 %self) unnamed_addr #1 {
start:
  %v1 = alloca [12 x i8], align 4
  %v = alloca [12 x i8], align 4
  %src = alloca [20 x i8], align 4
  %b = alloca [20 x i8], align 4
  store i32 0, ptr %v, align 4
  %0 = getelementptr inbounds i8, ptr %v, i32 4
  store ptr inttoptr (i32 1 to ptr), ptr %0, align 4
  %1 = getelementptr inbounds i8, ptr %v, i32 8
  store i32 0, ptr %1, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %v1, ptr align 4 %v, i32 12, i1 false)
  %2 = getelementptr inbounds i8, ptr %v1, i32 4
  %self2 = load ptr, ptr %2, align 4
  %3 = getelementptr inbounds i8, ptr %v1, i32 8
  %len = load i32, ptr %3, align 4
  %capacity = load i32, ptr %v1, align 4
  store ptr %self2, ptr %src, align 4
  %4 = getelementptr inbounds i8, ptr %src, i32 4
  store i32 %len, ptr %4, align 4
  %5 = getelementptr inbounds i8, ptr %src, i32 8
  store i32 %capacity, ptr %5, align 4
  %6 = getelementptr inbounds i8, ptr %src, i32 12
  store ptr @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from7reserve17h7f50c2c3b7d04034E", ptr %6, align 4
  %7 = getelementptr inbounds i8, ptr %src, i32 16
  store ptr @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from4drop17h97d84a4094b82bf1E", ptr %7, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %b, ptr align 4 %self, i32 20, i1 false)
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %self, ptr align 4 %src, i32 20, i1 false)
  %8 = getelementptr inbounds i8, ptr %b, i32 16
  %_4 = load ptr, ptr %8, align 4
  call void %_4(ptr align 4 %b) #5
  ret void
}

; <(A,B) as proc_macro::bridge::rpc::DecodeMut<S>>::decode
; Function Attrs: nounwind
define internal { i32, i32 } @"_ZN77_$LT$$LP$A$C$B$RP$$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17h778a3ee6399de895E"(ptr align 4 %r, ptr align 1 %s) unnamed_addr #0 {
start:
; call <proc_macro::bridge::client::TokenStream as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  %_3 = call i32 @"_ZN103_$LT$proc_macro..bridge..client..TokenStream$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17h9cde09496fe5af2dE"(ptr align 4 %r, ptr align 1 %s) #5
; call <proc_macro::bridge::client::TokenStream as proc_macro::bridge::rpc::DecodeMut<S>>::decode
  %_4 = call i32 @"_ZN103_$LT$proc_macro..bridge..client..TokenStream$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17h9cde09496fe5af2dE"(ptr align 4 %r, ptr align 1 %s) #5
  %0 = insertvalue { i32, i32 } poison, i32 %_3, 0
  %1 = insertvalue { i32, i32 } %0, i32 %_4, 1
  ret { i32, i32 } %1
}

; <proc_macro::bridge::rpc::PanicMessage as proc_macro::bridge::rpc::Encode<S>>::encode
; Function Attrs: nounwind
define internal void @"_ZN98_$LT$proc_macro..bridge..rpc..PanicMessage$u20$as$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$GT$6encode17hf798b9715d063dc7E"(ptr align 4 %self, ptr align 4 %w, ptr align 1 %s) unnamed_addr #0 {
start:
  %_5 = alloca [8 x i8], align 4
  %0 = load i32, ptr %self, align 4
  %1 = sub i32 %0, -2147483648
  %2 = icmp ule i32 %1, 2
  %_7 = select i1 %2, i32 %1, i32 1
  switch i32 %_7, label %bb4 [
    i32 0, label %bb7
    i32 1, label %bb6
    i32 2, label %bb5
  ]

bb4:                                              ; preds = %start
  unreachable

bb7:                                              ; preds = %start
  %s1 = getelementptr inbounds i8, ptr %self, i32 4
  %3 = getelementptr inbounds i8, ptr %self, i32 4
  %_11.0 = load ptr, ptr %3, align 4
  %4 = getelementptr inbounds i8, ptr %3, i32 4
  %_11.1 = load i32, ptr %4, align 4
  store ptr %_11.0, ptr %_5, align 4
  %5 = getelementptr inbounds i8, ptr %_5, i32 4
  store i32 %_11.1, ptr %5, align 4
  br label %bb3

bb6:                                              ; preds = %start
  %6 = getelementptr inbounds i8, ptr %self, i32 4
  %self2 = load ptr, ptr %6, align 4
  %7 = getelementptr inbounds i8, ptr %self, i32 8
  %len = load i32, ptr %7, align 4
  store ptr %self2, ptr %_5, align 4
  %8 = getelementptr inbounds i8, ptr %_5, i32 4
  store i32 %len, ptr %8, align 4
  br label %bb3

bb5:                                              ; preds = %start
  %9 = load ptr, ptr @0, align 4
  %10 = load i32, ptr getelementptr inbounds (i8, ptr @0, i32 4), align 4
  store ptr %9, ptr %_5, align 4
  %11 = getelementptr inbounds i8, ptr %_5, i32 4
  store i32 %10, ptr %11, align 4
  br label %bb3

bb3:                                              ; preds = %bb5, %bb6, %bb7
  %12 = load ptr, ptr %_5, align 4
  %13 = getelementptr inbounds i8, ptr %_5, i32 4
  %14 = load i32, ptr %13, align 4
; call proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::option::Option<T>>::encode
  call void @"_ZN10proc_macro6bridge100_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..option..Option$LT$T$GT$$GT$6encode17hf331ec7a40d39c3aE"(ptr align 1 %12, i32 %14, ptr align 4 %w, ptr align 1 %s) #5
; call core::ptr::drop_in_place<proc_macro::bridge::rpc::PanicMessage>
  call void @"_ZN4core3ptr58drop_in_place$LT$proc_macro..bridge..rpc..PanicMessage$GT$17h0843de3e9e4d056aE"(ptr align 4 %self) #5
  ret void
}

; simple_test_macro::simple_test
; Function Attrs: nounwind
define internal i32 @_ZN17simple_test_macro11simple_test17h5a5a53a270de6560E(i32 %0) unnamed_addr #0 {
start:
  %e.i = alloca [0 x i8], align 1
  %self.i = alloca [8 x i8], align 4
  %_input = alloca [4 x i8], align 4
  store i32 %0, ptr %_input, align 4
; call core::str::<impl str>::parse
  %1 = call { i32, i32 } @"_ZN4core3str21_$LT$impl$u20$str$GT$5parse17hd0580d600884e0c5E"(ptr align 1 @alloc_1e51fcb9bd01d98adcecd30e41bb5e77, i32 37) #5
  %_2.0 = extractvalue { i32, i32 } %1, 0
  %_2.1 = extractvalue { i32, i32 } %1, 1
  store i32 %_2.0, ptr %self.i, align 4
  %2 = getelementptr inbounds i8, ptr %self.i, i32 4
  store i32 %_2.1, ptr %2, align 4
  %_2.i = load i32, ptr %self.i, align 4
  %3 = icmp eq i32 %_2.i, 0
  br i1 %3, label %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7372f27f517f860fE.exit", label %bb2.i

bb2.i:                                            ; preds = %start
; call core::result::unwrap_failed
  call void @_ZN4core6result13unwrap_failed17hc04de2441a7172b3E(ptr align 1 @alloc_00ae4b301f7fab8ac9617c03fcbd7274, i32 43, ptr align 1 %e.i, ptr align 4 @vtable.1, ptr align 4 @alloc_719f6a58c1831a79b91c1db0675fe4c4) #6
  unreachable

"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7372f27f517f860fE.exit": ; preds = %start
  %4 = getelementptr inbounds i8, ptr %self.i, i32 4
  %t.i = load i32, ptr %4, align 4
; call core::ptr::drop_in_place<proc_macro::TokenStream>
  call void @"_ZN4core3ptr44drop_in_place$LT$proc_macro..TokenStream$GT$17ha4f053ae0cdcf5e0E"(ptr align 4 %_input) #5
  ret i32 %t.i
}

; simple_test_macro::simple_attr
; Function Attrs: nounwind
define internal i32 @_ZN17simple_test_macro11simple_attr17hdc19acd8708e9f69E(i32 %0, i32 %item) unnamed_addr #0 {
start:
  %_attr = alloca [4 x i8], align 4
  store i32 %0, ptr %_attr, align 4
; call core::ptr::drop_in_place<proc_macro::TokenStream>
  call void @"_ZN4core3ptr44drop_in_place$LT$proc_macro..TokenStream$GT$17ha4f053ae0cdcf5e0E"(ptr align 4 %_attr) #5
  ret i32 %item
}

; simple_test_macro::simple_bang
; Function Attrs: nounwind
define internal i32 @_ZN17simple_test_macro11simple_bang17hbf3aef8ad5965e72E(i32 %0) unnamed_addr #0 {
start:
  %e.i = alloca [0 x i8], align 1
  %self.i = alloca [8 x i8], align 4
  %_input = alloca [4 x i8], align 4
  store i32 %0, ptr %_input, align 4
; call core::str::<impl str>::parse
  %1 = call { i32, i32 } @"_ZN4core3str21_$LT$impl$u20$str$GT$5parse17hd0580d600884e0c5E"(ptr align 1 @alloc_6d61909b84c2d9e08f37facf1736c7b0, i32 5) #5
  %_2.0 = extractvalue { i32, i32 } %1, 0
  %_2.1 = extractvalue { i32, i32 } %1, 1
  store i32 %_2.0, ptr %self.i, align 4
  %2 = getelementptr inbounds i8, ptr %self.i, i32 4
  store i32 %_2.1, ptr %2, align 4
  %_2.i = load i32, ptr %self.i, align 4
  %3 = icmp eq i32 %_2.i, 0
  br i1 %3, label %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7372f27f517f860fE.exit", label %bb2.i

bb2.i:                                            ; preds = %start
; call core::result::unwrap_failed
  call void @_ZN4core6result13unwrap_failed17hc04de2441a7172b3E(ptr align 1 @alloc_00ae4b301f7fab8ac9617c03fcbd7274, i32 43, ptr align 1 %e.i, ptr align 4 @vtable.1, ptr align 4 @alloc_82198754260bc90b7e65095d98d42cbc) #6
  unreachable

"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7372f27f517f860fE.exit": ; preds = %start
  %4 = getelementptr inbounds i8, ptr %self.i, i32 4
  %t.i = load i32, ptr %4, align 4
; call core::ptr::drop_in_place<proc_macro::TokenStream>
  call void @"_ZN4core3ptr44drop_in_place$LT$proc_macro..TokenStream$GT$17ha4f053ae0cdcf5e0E"(ptr align 4 %_input) #5
  ret i32 %t.i
}

; <proc_macro::bridge::client::Span as proc_macro::bridge::rpc::DecodeMut<S>>::decode
; Function Attrs: nounwind
declare dso_local i32 @"_ZN96_$LT$proc_macro..bridge..client..Span$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17ha0c787454e2dc58cE"(ptr align 4, ptr align 1) unnamed_addr #0

; proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::option::Option<T>>::encode
; Function Attrs: nounwind
declare dso_local void @"_ZN10proc_macro6bridge100_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..option..Option$LT$T$GT$$GT$6encode17hf7a1b2022cc87ae9E"(i32, ptr align 4, ptr align 1) unnamed_addr #0

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i32(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i32, i1 immarg) #2

; <proc_macro::bridge::buffer::Buffer as core::convert::From<alloc::vec::Vec<u8>>>::from::reserve
; Function Attrs: nounwind
declare dso_local void @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from7reserve17h7f50c2c3b7d04034E"(ptr sret([20 x i8]) align 4, ptr align 4, i32) unnamed_addr #0

; <proc_macro::bridge::buffer::Buffer as core::convert::From<alloc::vec::Vec<u8>>>::from::drop
; Function Attrs: nounwind
declare dso_local void @"_ZN107_$LT$proc_macro..bridge..buffer..Buffer$u20$as$u20$core..convert..From$LT$alloc..vec..Vec$LT$u8$GT$$GT$$GT$4from4drop17h97d84a4094b82bf1E"(ptr align 4) unnamed_addr #0

; proc_macro::bridge::symbol::Symbol::invalidate_all
; Function Attrs: nounwind
declare dso_local void @_ZN10proc_macro6bridge6symbol6Symbol14invalidate_all17h8539ab76ea2c91c0E() unnamed_addr #0

; proc_macro::bridge::client::maybe_install_panic_hook
; Function Attrs: nounwind
declare dso_local void @_ZN10proc_macro6bridge6client24maybe_install_panic_hook17h5c87af20ddbf148aE(i1 zeroext) unnamed_addr #0

; <proc_macro::bridge::client::TokenStream as proc_macro::bridge::rpc::DecodeMut<S>>::decode
; Function Attrs: nounwind
declare dso_local i32 @"_ZN103_$LT$proc_macro..bridge..client..TokenStream$u20$as$u20$proc_macro..bridge..rpc..DecodeMut$LT$S$GT$$GT$6decode17h9cde09496fe5af2dE"(ptr align 4, ptr align 1) unnamed_addr #0

; <std::thread::local::AccessError as core::fmt::Debug>::fmt
; Function Attrs: nounwind
declare dso_local zeroext i1 @"_ZN68_$LT$std..thread..local..AccessError$u20$as$u20$core..fmt..Debug$GT$3fmt17h0f90263692302964E"(ptr align 1, ptr align 4) unnamed_addr #0

; core::result::unwrap_failed
; Function Attrs: cold noinline noreturn nounwind
declare dso_local void @_ZN4core6result13unwrap_failed17hc04de2441a7172b3E(ptr align 1, i32, ptr align 1, ptr align 4, ptr align 4) unnamed_addr #3

; std::panicking::try::cleanup
; Function Attrs: cold minsize nounwind optsize
declare dso_local { ptr, ptr } @_ZN3std9panicking3try7cleanup17h8463d18afe7024c0E(ptr) unnamed_addr #4

; <proc_macro::bridge::rpc::PanicMessage as core::convert::From<alloc::boxed::Box<dyn core::any::Any+core::marker::Send>>>::from
; Function Attrs: nounwind
declare dso_local void @"_ZN155_$LT$proc_macro..bridge..rpc..PanicMessage$u20$as$u20$core..convert..From$LT$alloc..boxed..Box$LT$dyn$u20$core..any..Any$u2b$core..marker..Send$GT$$GT$$GT$4from17h1eea7de01293fee1E"(ptr sret([12 x i8]) align 4, ptr align 1, ptr align 4) unnamed_addr #0

; <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: nounwind
declare dso_local void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h270b2613cc3e127fE"(ptr align 4) unnamed_addr #0

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: nounwind
declare dso_local void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h57f67b216edb4637E"(ptr align 4) unnamed_addr #0

; <proc_macro::bridge::client::TokenStream as core::ops::drop::Drop>::drop
; Function Attrs: nounwind
declare dso_local void @"_ZN81_$LT$proc_macro..bridge..client..TokenStream$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd91f8cba53cc4621E"(ptr align 4) unnamed_addr #0

; <proc_macro::bridge::client::state::set::RestoreOnDrop as core::ops::drop::Drop>::drop
; Function Attrs: nounwind
declare dso_local void @"_ZN95_$LT$proc_macro..bridge..client..state..set..RestoreOnDrop$u20$as$u20$core..ops..drop..Drop$GT$4drop17h9279ee21d250538dE"(ptr align 4) unnamed_addr #0

; <proc_macro::TokenStream as core::str::traits::FromStr>::from_str
; Function Attrs: nounwind
declare dso_local { i32, i32 } @"_ZN70_$LT$proc_macro..TokenStream$u20$as$u20$core..str..traits..FromStr$GT$8from_str17hf6b6a2d5891bda9fE"(ptr align 1, i32) unnamed_addr #0

; core::fmt::Formatter::write_str
; Function Attrs: nounwind
declare dso_local zeroext i1 @_ZN4core3fmt9Formatter9write_str17hb77f51236005fc7aE(ptr align 4, ptr align 1, i32) unnamed_addr #0

; proc_macro::bridge::<impl proc_macro::bridge::rpc::Encode<S> for core::option::Option<T>>::encode
; Function Attrs: nounwind
declare dso_local void @"_ZN10proc_macro6bridge100_$LT$impl$u20$proc_macro..bridge..rpc..Encode$LT$S$GT$$u20$for$u20$core..option..Option$LT$T$GT$$GT$6encode17hf331ec7a40d39c3aE"(ptr align 1, i32, ptr align 4, ptr align 1) unnamed_addr #0

attributes #0 = { nounwind "target-cpu"="generic" }
attributes #1 = { inlinehint nounwind "target-cpu"="generic" }
attributes #2 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #3 = { cold noinline noreturn nounwind "target-cpu"="generic" }
attributes #4 = { cold minsize nounwind optsize "target-cpu"="generic" }
attributes #5 = { nounwind }
attributes #6 = { noreturn nounwind }

!llvm.ident = !{!0}

!0 = !{!"rustc version 1.84.1 (b8bf17f81 2025-10-30)"}
