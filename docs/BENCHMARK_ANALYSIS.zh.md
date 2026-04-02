# Benchmark 鍒嗘瀽

鑻辨枃鐗堬細`docs/BENCHMARK_ANALYSIS.md`

鐩稿叧浼樺寲浠诲姟娓呭崟锛?- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

## 鐩殑

鏈枃妗ｅ畾涔?`mquickjs-rs` 褰撳墠鐨?benchmark 宸ヤ綔娴佷笌瑙ｈ瑙勫垯銆?
瀹冨彧闈㈠悜寮曟搸灞傦紝涓嶈鐩?`led-runtime` 鐨勪骇鍝佸眰琛屼负銆?
## 瑙勮寖 Canary 闆嗗悎

Phase 1 褰撳墠绾﹀畾鐨勮鑼?canary 闆嗗悎鏄細

- `method_chain`
- `runtime_string_pressure`
- `for_of_array`
- `deep_property`

瀹冧滑鍞竴鐨勬満鍣ㄥ彲璇绘潵婧愭槸锛?
- `benches/manifests/canary_benchmarks.txt`

鏈湴 helper 鍜?CI 瀵规瘮娴佺▼閮藉簲娑堣垂杩欎唤 manifest锛屼笉瑕佸湪鑴氭湰鎴栨枃妗ｉ噷鍐嶇淮鎶ょ浜屽纭紪鐮佸垪琛ㄣ€?
## Benchmark 瑙掕壊鍒嗗伐

褰撳墠 benchmark 鍒嗘瀽鐢卞洓鏉′簰琛ユ祦绋嬬粍鎴愩€?
### 1. 鏈湴蹇€?canary 閲嶈窇

鍛戒护锛?
```bash
bash benches/run_canaries.sh
```

鐢ㄩ€旓細

- 鍙噸璺戝洓涓紭鍖?canary
- 蹇€熼獙璇佺儹鐐规敼鍔?- 閬垮厤鏃ュ父杩唬鏃舵瘡娆￠兘璺戝畬鏁?benchmark 璇枡

杈呭姪妯″紡锛?
```bash
bash benches/run_canaries.sh --list
```

瀹冧細鎵撳嵃褰撳墠瑙勮寖 canary manifest锛岃€屼笉浼氱湡姝ｆ墽琛?benchmark銆?
### 2. 鏈湴瑙勮寖 Rust-vs-C 瀵规瘮

鍛戒护锛?
```bash
bash benches/compare.sh
```

鐢ㄩ€旓細

- 瀵规瘮 Rust 寮曟搸涓庝粨搴撳唴缃殑 C 鍙傝€冨疄鐜?- 鍦ㄤ笌 CI 鐩稿悓鐨?canary 闆嗗悎涓婅　閲忚法瀹炵幇宸窛
- 鐢ㄤ竴鏉＄鍒扮鏁板瓧鍚屾椂瑕嗙洊鍚姩銆佸姞杞姐€佺紪璇戝拰鎵ц鎴愭湰

閲嶈璇存槑锛?
- 瑙勮寖瀵规瘮榛樿瑕佹眰鍙傝€冩爲浣嶄簬 `contrib/mquickjs/`
- 濡傛灉鍙傝€冩爲涓嶅彲鐢紝瑙勮寖妯″紡搴旀槑纭け璐ワ紝鑰屼笉鏄倓鎮勯€€鍖栨垚 Rust-only
- Rust-only 绛夋樉寮忛潪瑙勮寖妯″紡浠嶅彲鐢ㄤ簬璇婃柇锛屼絾瀹冧滑涓嶅睘浜?Phase 1 baseline 鍚堢害

### 3. 鏈湴 execution-only Rust-vs-C 瀵规瘮

鍛戒护锛?
```bash
bash benches/compare.sh --execution-only
```

鐢ㄩ€旓細

- 鍦?compile-once / execute-many 鍙ｅ緞涓嬪姣?Rust 涓?C
- 璁╄法瀹炵幇瀵规瘮鏇存帴杩?Criterion 鐨勮繍琛屾椂鍏虫敞鐐?- 鏇存竻妤氬湴鍖哄垎 Rust-vs-C 宸窛涓昏鏉ヨ嚜绔埌绔噯澶囬樁娈碉紝杩樻槸鏉ヨ嚜绋虫€佹墽琛岄樁娈?
閲嶈璇存槑锛?
- 杩欎釜妯″紡鐩墠鏄湰鍦拌瘖鏂伐鍏凤紝杩樹笉鏄?CI 閲岀殑瑙勮寖鍚堢害
- helper 浠嶇劧浼氬湪姣忔杩唬閲屽垱寤?fresh context锛屾墍浠ュ畠鏄€滃亸杩愯鏃垛€濈殑瀵规瘮锛岃€屼笉鏄崟 context 鐨勬瀬灏忓瀷 microbenchmark

### 4. CI benchmark 鎽樿

宸ヤ綔娴侊細

- `.github/workflows/bench.yml`

鐢ㄩ€旓細

- 鍦?GitHub Actions 涓彂甯冭鑼?Rust-vs-C canary 瀵规瘮琛?- 鍗曠嫭鍙戝竷 Rust-only 鐨?Criterion 琛?- 璁?push / PR 鍙互鍦ㄤ笉鏀瑰彉鏈湴娴佺▼瀹氫箟鐨勫墠鎻愪笅寰楀埌鍙鍙嶉

CI 瀵规瘮琛ㄥ簲琚涓轰笌鏈湴 `compare.sh` 鐩稿悓鐨?canary 鍚堢害锛岃€屼笉鏄彟涓€濂楃嫭绔?benchmark 鍒楄〃銆?
## 鍙傝€冩爲鍋囪

褰撳墠浠撳簱鍐呯疆鐨?C 鍙傝€冩爲璺緞鏄細

- `contrib/mquickjs/`

瑙勮寖 benchmark 宸ュ叿搴斿彧浠庤繖閲屽鎵?Rust-vs-C 瀵规瘮鎵€闇€鐨勫弬鑰冨疄鐜般€?
## 瑙ｈ瑙勫垯

### 涓夌鏃堕棿鍙ｅ緞鍥炵瓟鐨勬槸涓嶅悓闂

- Criterion 鍥炵瓟锛氣€滆繖娆?Rust 渚ф敼鍔ㄦ槸鍚︾湡鐨勬敼鍠勪簡鐩爣鐑偣锛熲€?- 鏈湴 / CI Rust-vs-C 瀵规瘮鍥炵瓟锛氣€滅湡瀹炵鍒扮璺緞璺濈 C 鍙傝€冨疄鐜拌繕鏈夊杩滐紵鈥?- execution-only Rust-vs-C 瀵规瘮鍥炵瓟锛氣€滃湪寮卞寲缂栬瘧 / 鍑嗗鍣０涔嬪悗锛岃繖涓樊璺濊繕鍓╁灏戯紵鈥?
涓嶈鎶婅繖涓夌被鏁板瓧娣锋垚涓€涓€滄€诲垎鈥濄€?
### 濡備綍瑙ｈ绔埌绔?vs execution-only

- 濡傛灉绔埌绔槑鏄捐惤鍚庯紝浣?execution-only 宸茬粡鍋ュ悍锛屽綋鍓嶅樊璺濇洿鍙兘鍦ㄥ惎鍔ㄣ€佹枃浠跺姞杞姐€佽В鏋愩€佺紪璇戞垨瀛楄妭鐮佽杞?- 濡傛灉绔埌绔拰 execution-only 閮借惤鍚庯紝褰撳墠宸窛鏇村彲鑳藉湪绋虫€佽繍琛屾椂鎵ц
- 濡傛灉 Criterion 鍙樺ソ浜嗭紝浣?execution-only Rust-vs-C 娌″彉锛岃鏄?Rust 渚т紭鍖栧彲鑳界湡瀹炴湁鏁堬紝浣嗙 C 浠嶆湁鏇村ぇ鐨勭粨鏋勬€у樊璺?
### Canary 鏀剁泭涓嶇瓑浜庡畬鏁存€ц兘缁撹

canary 闆嗗悎鏄儹鐐瑰伐浣滅殑鏈€蹇俊鍙凤紝浣嗗畠涓嶆槸鍏ㄩ儴鎬ц兘鏁呬簨銆?
鍋氬畬涓€娆℃湁鎰忎箟鐨勪紭鍖栧悗锛屾帹鑽愰『搴忔槸锛?
1. 鍏堥噸璺戠浉鍏?canary
2. 鍐嶇湅瀵瑰簲鍥炲綊娴嬭瘯
3. 鍙湁褰撳眬閮ㄧ粨鏋滃€煎緱缁х画鏃讹紝鍐嶆墿澶?benchmark 瑕嗙洊闈?
### 杩欏洓涓?canary 鏄満鍒跺鍚戠殑

- `method_chain` 渚ч噸鍥炶皟瀵嗛泦鐨勬暟缁?pipeline 涓?call / builtin 寮€閿€
- `runtime_string_pressure` 渚ч噸杩愯鏃跺瓧绗︿覆璺緞
- `for_of_array` 渚ч噸 iterator 涓庢暟缁勮凯浠ｈ矾寰?- `deep_property` 渚ч噸閲嶅灞炴€ц闂矾寰?
鍥犳锛屽畠浠€傚悎浣滀负 VM銆乸roperty銆乥uiltin 鍜?runtime-string 鏀瑰姩鐨勯粯璁ゆ€ц兘 smoke test銆?
## 缁存姢鑰呭伐浣滄祦

褰撲綘淇敼 benchmark 鏁忔劅鐨勫紩鎿庝唬鐮佹椂锛?
1. 鍏堣繍琛岀浉鍏崇殑瀹氬悜娴嬭瘯銆?2. 鐢?`bash benches/run_canaries.sh` 閲嶈窇瑙勮寖 canary銆?3. 濡傛灉鏀瑰姩鐩爣鏄敼鍠?Rust-vs-C 璺濈锛屽啀杩愯 `bash benches/compare.sh`銆?4. 濡傛灉鏀瑰姩鐩爣鏄敼鍠勭ǔ鎬佽繍琛屾椂璺濈锛屽啀棰濆杩愯 `bash benches/compare.sh --execution-only`銆?5. 濡傛灉 benchmark 宸ヤ綔娴佹垨瑙ｈ瑙勫垯鍙戠敓鍙樺寲锛屽強鏃舵洿鏂版湰鏂囨。銆?
## 澶囨敞

- 鏈枃妗ｅ埢鎰忚仛鐒﹀湪鈥滃伐浣滄祦鈥濆拰鈥滆В璇荤邯寰嬧€濅笂銆?- 鍘嗗彶 benchmark 蹇収浠嶇劧鏈夊弬鑰冧环鍊硷紝浣嗕笉鑳借鐩栧綋鍓嶈繖濂楄鑼冩祦绋嬨€?
