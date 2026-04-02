# 寮曟搸浼樺寲浠诲姟娓呭崟

鏈枃妗ｆ槸 `mquickjs-rs` **浠呴潰鍚戝紩鎿?*鐨勪紭鍖栧緟鍔炴竻鍗曘€?
瀹冪洿鎺ユ簮鑷?`IMPLEMENTATION_PLAN.md` 涓皻鏈畬鎴愮殑绗?9 闃舵锛?- `9.1 Profile and optimize hot paths`
- `9.2 Optimize GC performance`
- `9.3 Reduce memory usage`

鏈枃妗ｄ笉鍖呭惈 `led-runtime` 浜у搧灞傚伐浣溿€?
鐩稿叧 benchmark 鍒嗘瀽锛?- `docs/BENCHMARK_ANALYSIS.md`
- `docs/BENCHMARK_ANALYSIS.zh.md`

## 閫傜敤鑼冨洿

鏈枃妗ｅ彧瑕嗙洊锛?
- `mquickjs-rs` 鐨?parser / compiler / VM / runtime
- benchmark 鐨勬纭€т笌鎬ц兘鍒嗘瀽
- 寮曟搸鑷韩鐨?GC 涓庡唴瀛樿涓?
鏈枃妗ｄ笉瑕嗙洊锛?
- `led-runtime` 涓绘満 API 浜轰綋宸ュ
- effect 鑴氭湰/浜у搧璇箟
- GUI / demo / 浜у搧灞傞泦鎴?
## 褰撳墠浼樺寲涓婚

缁撳悎褰撳墠浠ｇ爜鍜?benchmark 褰㈢姸锛屾渶鍊煎緱鍏虫敞鐨勫紩鎿庣儹鐐规槸锛?
- `src/vm/interpreter.rs` 涓殑璋冪敤涓庢柟娉曞垎鍙?- `src/vm/interpreter.rs` 鍜?`src/vm/natives.rs` 涓殑 native / builtin 鍙傛暟鏁寸悊
- `src/vm/interpreter.rs` 鍜?`src/vm/property.rs` 涓殑 dense array 璁块棶
- `src/vm/interpreter.rs` 涓殑 opcode dispatch 寮€閿€
- `src/vm/gc.rs` 涓殑 GC 瀹炵幇璐ㄩ噺锛圥lan B mark-sweep锛夛紱`src/gc/collector.rs` 涓?Plan C 鍗犱綅绗?- `src/vm/types.rs`銆乣src/context.rs` 鍜?`src/runtime/*` 涓殑杩愯鏃跺垎閰嶄笌瀹瑰櫒甯冨眬

## 浼樺厛绾ф€荤粨

### P0

- benchmark 鐪熷疄鍩虹嚎娓呯悊
- 璋冪敤璺緞鐑矾寰勪紭鍖?- Native/builtin 璋冪敤鍙傛暟浼犻€掍紭鍖?- Dense array 蹇€熻矾寰?
### P1

- 鏈€鐑?opcode 鐨?dispatch 绠€鍖?- GC锛氬仠姝繚瀹堢殑 `mark_all` 琛屼负锛岃縼绉诲埌鐪熸鐨?root-based marking
- 杩愯鏃跺垎閰嶄笌鍐呭瓨鍗犵敤璇勪及

### P2

- Builtin/runtime 杈圭晫缁撴瀯娓呯悊
- 鏂?benchmark 楠岃瘉鍚庣殑绗簩杞井浼樺寲

## 璇︾粏浠诲姟娓呭崟

## 9.1 鍒嗘瀽骞朵紭鍖栫儹璺緞

### 9.1.1 Benchmark 鍩虹嚎娓呯悊

**浼樺厛绾?*: P0

**鍘熷洜**

- 浼樺寲宸ヤ綔鍙湁鍦?benchmark 鏁版嵁鍙俊鏃舵墠鏈夋剰涔夈€?- benchmark 宸ヤ綔娴佸拰鏈湴瀵规瘮鑴氭湰姝ゅ墠瀛樺湪涓嶄竴鑷淬€?- 閮ㄥ垎鍘嗗彶 benchmark 缁撹鍩轰簬鏃犳晥鐨勫姣旂洰鏍囥€?
**浠诲姟**

- 淇濇寔鍗曚竴鍙俊鐨勬湰鍦?benchmark 娴佺▼銆?- 浣?CI benchmark 琛屼负涓庢湰鍦?benchmark 琛屼负淇濇寔涓€鑷淬€?- 灏嗚繘绋嬪惎鍔ㄥ紑閿€涓庡噣鑴氭湰鎵ц鏃堕棿鍒嗗紑銆?- 缁存姢浠ヤ笅鍩哄噯鐨勬潈濞佸熀绾胯〃锛?  - `fib`
  - `loop`
  - `array`
  - `sieve`
  - `json`

**楠岃瘉**

- benchmark 缁撴灉鍦ㄥ娆¤繍琛屼腑鍙鐜般€?- `docs/BENCHMARK_ANALYSIS.md` 鍐呴儴涓€鑷淬€?
**褰撳墠宸插畬鎴?*

- 2026-03-16锛氳鑼?benchmark 闆嗗悎宸插畾涔夈€?- 2026-03-16锛氭湰鍦?Criterion銆佹湰鍦?Rust vs C 瀵规瘮銆丆I 鎽樿鐨勮亴璐ｅ凡鍒嗙骞惰褰曘€?- 2026-03-16锛歚.github/workflows/bench.yml` 鐜板凡鍚屾椂鍙戝竷 Rust vs C 瀵规瘮琛ㄥ拰绾?Rust Criterion 琛ㄣ€?- 2026-03-16锛歚docs/BENCHMARK_ANALYSIS.md` 宸查噸鍐欎负褰撳墠鍩虹嚎鍙傝€冦€?- 2026-03-17锛氬褰撳墠宸ヤ綔鏍戠殑涓?benchmark 闆嗗悎鍋氫簡涓€杞畬鏁存湰鍦?Criterion 閲嶉獙璇併€?- 2026-03-17锛氭湰鍦?Criterion harness 宸叉敼涓衡€滈缂栬瘧涓€娆°€佸湪鏂?context 涓婇噸澶嶆祴鎵ц鈥濓紝浠ュ噺寮?parser/compiler 瀵硅繍琛屾椂浼樺寲鍒ゆ柇鐨勬薄鏌撱€?- 2026-03-17锛歚docs/BENCHMARK_ANALYSIS.md` / `docs/BENCHMARK_ANALYSIS.zh.md` 宸叉洿鏂颁负鍖哄垎鏂扮殑鎵ц鏈熷揩鐓у拰鏃х殑 Criterion 浠ｉ檯鏁版嵁銆?- 鐘舵€侊細閲嶆柊鎵撳紑锛涘湪褰撳墠 head 涓庢枃妗ｉ噸鏂扮ǔ瀹氬悓姝ヤ箣鍓嶏紝benchmark 鍩虹嚎娓呯悊涓嶈兘鍐嶈涓哄畬鎴愩€?
### 9.1.2 璋冪敤璺緞鐑矾寰勪紭鍖?[宸插交搴曞畬鎴怾

**浼樺厛绾?*: P0

**鐑偣鏂囦欢**

- `src/vm/interpreter.rs`
- `src/vm/stack.rs`

**鍘熷洜**

- `fib` 鍜?`loop` 寮虹儓琛ㄦ槑璋冪敤寮€閿€鍜岄珮棰?dispatch 寮€閿€浠嶇劧鏄富瑕佹垚鏈€?- 褰撳墠 `Call` 璺緞鏈夋墍鏀硅繘锛屼絾浠嶄娇鐢?`remove_at_offset()`锛屽畠濮旀墭缁?`Vec::remove()`锛屼細瀵艰嚧鍏冪礌绉诲姩銆?
**浠诲姟**

- 閲嶆瀯璋冪敤鏍堝竷灞€锛岄伩鍏嶇儹璋冪敤璺緞涓婄殑 `Vec::remove()`銆?- 鍒嗗埆鐗瑰寲 `Call`銆乣CallMethod` 鍜?`CallConstructor`銆?- 鍑忓皯鏅€?JS 鍑芥暟璋冪敤涓殑涓存椂鍙傛暟閲嶅銆?- 閲嶆柊妫€鏌ヨ皟鐢ㄨ矾寰勪腑鐨勫瓧绗︿覆鎻愬崌鎴愭湰銆?
**棰勬湡鏀剁泭**

- `fib` 鐨勪富瑕佹敼杩涚洰鏍?- `loop` 鐨勬瑕佹敼杩?
**褰撳墠宸插畬鎴?*

- 2026-03-16锛氬畬鎴愪簡 `method_chain` 鐩稿叧浼樺寲鐨勭涓€杞紝閫氳繃鍘婚櫎鍥炶皟瀵嗛泦鍨嬫暟缁?builtin 涓瘡涓厓绱犵殑涓存椂 `Vec<Value>` 鍙傛暟鍒嗛厤鏉ュ畬鎴愩€?- 涓洪摼寮?`map().filter().reduce()` 琛屼负娣诲姞浜嗗洖褰掕鐩栥€?- Benchmark 缁撴灉锛歚method_chain 5k` 鍦?Criterion 涓粠绾?`1.88鈥?.54 ms` 鎻愬崌鍒?`0.80鈥?.82 ms`銆?- 2026-03-17锛氬皢 `Call` / `CallMethod` 鐑矾寰勪腑鍩轰簬 `Vec::remove()` 鐨勭洰鏍囨彁鍙栨敼涓哄崟娆″熬閮ㄧ揣缂╋紝骞惰鏅€?JS 鏂规硶璋冪敤缁х画鐩存帴澶嶇敤鏍堜笂鐨勫弬鏁帮紝鑰屼笉鏄噸鏂版墦鍖呮垚涓存椂 `Vec<Value>`銆?- 2026-03-17锛氬皢鍚屾牱鐨勨€滃弬鏁板師鍦颁繚鐣欌€濇€濊矾鎵╁睍鍒颁簡 `CallConstructor`锛屼娇鏅€?JS 鏋勯€犲櫒璋冪敤涔熶笉鍐嶉€氳繃涓存椂 `Vec<Value>` 閲嶅缓鍙傛暟鍒楄〃銆?- 閲嶆柊璺戜簡鐩存帴鍑芥暟璋冪敤銆佸鍙傛暟 `Array.prototype.push` 椤哄簭銆佷互鍙婇摼寮?`map().filter().reduce()` 鐨勫洖褰掕鐩栵紝缁撴灉鍧囬€氳繃銆?- 閲嶆柊璺戜簡鏋勯€犲櫒璇箟鐩稿叧鍥炲綊瑕嗙洊锛坄new`銆乣instanceof`銆佺畝鍗曟瀯閫犲櫒鍦烘櫙锛夛紝缁撴灉涔熼€氳繃銆?- 鍦ㄦ柊鐨勨€滈缂栬瘧涓€娆°€侀噸澶嶆祴鎵ц鈥滳riterion 鍙ｅ緞涓嬶紝褰撳墠鏈湴蹇収涓猴細
  - `fib_iter 1k`锛歚2.330鈥?.379 ms`
  - `loop 10k`锛歚0.472鈥?.485 ms`
  - `array push 10k`锛歚0.614鈥?.633 ms`
- 褰撳墠瑙ｈ锛氳皟鐢ㄨ矾寰勮繖杞伐浣滀緷鐒舵槸鐪熷疄鏈夋晥鐨勶紝浣嗗悗缁瘮杈冨繀椤讳弗鏍奸檺瀹氬湪鏂扮殑鎵ц鏈?benchmark 浠ｉ檯鍐呴儴杩涜銆?- 2026-03-17锛氬皢 `map`銆乣filter`銆乣forEach`銆乣reduce`銆乣find`銆乣findIndex`銆乣some`銆乣every` 杩欎簺鐑暟缁?builtin 浠庘€滄暣鏁扮粍 clone鈥濇敼鎴愨€滈暱搴﹀揩鐓?+ 瀹炴椂鍏冪礌璇诲彇鈥濄€?- 娣诲姞浜嗗洖褰掕鐩栦互閿佸畾锛?  - 鍥炶皟閲?`push()` 涓嶄細鏀瑰彉鏈疆閬嶅巻闀垮害
  - `map()` 鑳借瀵熷埌鍓嶉潰鍥炶皟瀵瑰悗缁厓绱犵殑鏇存柊
- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝鏈€鏂板畬鏁撮噸璺戣褰曪細
  - `method_chain 5k`锛歚0.699鈥?.707 ms`
  - `runtime_string_pressure 4k`锛歚1.237鈥?.269 ms`
- 褰撳墠瑙ｈ锛氳繖涓€杞槑鏄炬敼鍠勪簡鍥炶皟瀵嗛泦鍨嬫暟缁勭绾匡紝骞堕€氳繃涓撻棬鐨?`.length` 蹇矾寰勫拰鏇翠綆鐨?builtin 寮€閿€锛岄『甯︽媺浣庝簡 runtime-string-heavy 寰幆鐨勬墽琛屾垚鏈€?- 2026-03-17锛氭柊澧炰簡涓撻棬鐨?`CallArrayMap1` / `CallArrayFilter1` / `CallArrayReduce2` opcode锛屼娇鏈€鐑殑鍗曞洖璋冩暟缁勯珮闃舵柟娉曡皟鐢ㄥ舰鐘跺湪 `GetField2` 涔嬪悗涓嶅啀缁х画鏀粯閫氱敤 `CallMethod` 鐨勫弬鏁伴噸鎺掓垚鏈€?- 琛ュ厖浜?fallback 鍥炲綊瑕嗙洊锛岀‘璁ら潪鏁扮粍 receiver 鍙鑷甫 `map` 鏂规硶锛屼粛鐒朵繚鎸侀€氱敤鏂规硶璋冪敤璇箟銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `method_chain 5k`锛歚0.611鈥?.628 ms`
  - `runtime_string_pressure 4k`锛歚1.190鈥?.216 ms`
  - `array push 10k`锛歚0.575鈥?.600 ms`
- 褰撳墠瑙ｈ锛氳繖鏄竴娆″緢鍏稿瀷鐨勨€滄寜瀛楄妭鐮佸舰鐘朵笓闂ㄤ紭鍖栨暟缁?builtin 璋冪敤閾锯€濈殑鏀剁泭妗堜緥锛岃€屼笖娌℃湁鎵╁ぇ閫氱敤璋冪敤璺緞鐨勫鏉傚害锛屾敹鐩婁篃鍚戦檮杩戠殑鏁扮粍瀵嗛泦璺緞澶栨孩銆?- 2026-03-17锛氭柊澧炰簡涓撻棬鐨?`CallArrayPush1` opcode锛岀洿鎺ヨ鐩栨渶鐑殑鍗曞弬鏁?`.push(arg)` 鏂规硶璋冪敤褰㈢姸锛涘畠淇濈暀 `GetField2` 鐨勭粺涓€鏍堢害瀹氾紝浣嗚鏁扮粍鏋勫缓寰幆涓嶅啀涓鸿繖鏉′富鐑矾寰勭户缁敮浠橀€氱敤 `CallMethod` 鐨勬暣鐞嗘垚鏈€?- 琛ュ厖浜?fallback 鍥炲綊瑕嗙洊锛岀‘璁ら潪鏁扮粍 receiver 鍙鑷甫 `push` 鏂规硶锛屼粛鐒朵繚鎸侀€氱敤鏂规硶璋冪敤璇箟銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `array push 10k`锛歚0.491鈥?.502 ms`
  - `method_chain 5k`锛歚0.585鈥?.600 ms`
  - `runtime_string_pressure 4k`锛歚1.177鈥?.197 ms`
- 褰撳墠瑙ｈ锛氳繖鏄涓€杞妸 `method_chain` 绋冲畾鍘嬪埌 `<= 0.60 ms` 鎴愬姛绾胯竟缂樼殑浼樺寲锛岃€屼笖鏀剁泭鏉ユ簮寰堟竻妤氾紝灏辨槸缁х画缂╂帀浜嗗湪楂橀樁鏁扮粍閾捐皟鐢ㄤ箣鍓嶄粛鐒跺崰涓诲鐨勬暟缁勬瀯寤哄墠缂€銆?- 鐘舵€侊細浣滀负鈥滆皟鐢ㄨ矾寰勭儹璺緞浼樺寲鈥濊繖涓€闃舵锛岃繖閮ㄥ垎鐜板湪鍙互瑙嗕负瀹屾垚锛涘悗缁鏋滆繕鏈夋敹鐩婏紝涔熷簲褰掔被涓哄悗缁井璋冿紝鑰屼笉鏄牳蹇冭皟鐢ㄨ矾寰勬竻鐞嗘湭瀹屾垚銆?
### 9.1.3 Native/builtin 璋冪敤鍙傛暟鏁寸悊浼樺寲

**浼樺厛绾?*: P0

**鐑偣鏂囦欢**

- `src/vm/interpreter.rs`
- `src/vm/natives.rs`

**鍘熷洜**

- Native 鍜?builtin 璋冪敤浠嶇劧鏋勫缓涓存椂 `Vec<Value>` 缂撳啿鍖哄苟杩涜鍙嶈浆銆?- 姝よ矾寰勫奖鍝?`Math.*`銆乣JSON.*`銆佹暟缁勬柟娉曞拰鍏朵粬 builtin銆?
**浠诲姟**

- 涓?0/1/2 涓弬鏁扮殑 native 璋冪敤娣诲姞涓撶敤蹇€熻矾寰勩€?- 閬垮厤涓虹煭 native/builtin 鍙傛暟鍒楄〃杩涜鍫嗗垎閰嶃€?- 鍑忓皯鎴栨秷闄?native/builtin 璋冪敤鍑嗗涓殑 `reverse()`銆?- 鑰冭檻鍦ㄥ畨鍏ㄧ殑鍦版柟浼犻€掓爤鏀寔鐨勫弬鏁板垏鐗囥€?
**棰勬湡鏀剁泭**

- 鏀瑰杽 builtin 瀵嗛泦鍨嬭剼鏈?- 甯姪 `array`銆乣json` 鍜屾暟瀛﹀瘑闆嗗瀷宸ヤ綔璐熻浇

**褰撳墠宸插畬鎴?*

- 2026-03-16锛氶€氳繃鍦?`argc <= 2` 鐨?native 鏂规硶璺緞涓婄Щ闄や复鏃跺弬鏁?`Vec` 鍒嗛厤锛屼负灏忓弬鏁版暟閲忔坊鍔犱簡 `CallMethod` native 蹇€熻矾寰勩€?- 涓哄鍙傛暟 `Array.prototype.push` 鍙傛暟椤哄簭娣诲姞浜嗗洖褰掕鐩栥€?- Benchmark 缁撴灉锛歚array push 10k` 鍦?Criterion 涓粠绾?`0.897鈥?.911 ms` 鎻愬崌鍒?`0.672鈥?.691 ms`銆?- Benchmark 缁撴灉锛歚method_chain 5k` 鍦?Criterion 涓繘涓€姝ヤ粠绾?`0.986鈥?.182 ms` 鎻愬崌鍒?`0.720鈥?.763 ms`銆?- 2026-03-16锛氬湪 `CallMethod` 涓负 `Array.prototype.push` 娣诲姞浜嗙洿鎺?native 蹇€熻矾寰勶紝甯︽湁涓撶敤鐨?`argc == 1` 鎹峰緞锛屼粠鐑暟缁勫垵濮嬪寲璺緞涓Щ闄や簡閫氱敤 native 璋冪敤寮€閿€銆?- 澶嶇敤鐜版湁鐨?`Array.prototype.push` 鍥炲綊瑕嗙洊鏉ラ獙璇佽涔夈€?- Benchmark 缁撴灉锛歚sieve 10k` 鍦?Criterion 涓粠绾?`2.038鈥?.078 ms` 鎻愬崌鍒?`2.014鈥?.074 ms`銆?- 2026-03-17锛氬皢鏁扮粍 `.push` 鐨勫睘鎬ц鍙栨敼涓虹洿鎺ヨ繑鍥炵紦瀛樼殑 native 鍑芥暟绱㈠紩锛岃€屼笉鏄瘡娆￠兘鎸夊悕瀛楃嚎鎬ф壂鎻?native 娉ㄥ唽琛ㄣ€?- 閲嶆柊璺戜簡 `Array.prototype.push` 鐨勫洖褰掕鐩栵紝缁撴灉閫氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `array push 10k`锛歚0.589鈥?.602 ms`
  - `method_chain 5k`锛歚0.654鈥?.668 ms`
- 褰撳墠瑙ｈ锛氳繖鏄竴娆″皬浣嗙湡瀹炵殑鐑偣鏁扮粍鏂规硶灞炴€у垎鍙戞竻鐞嗭紝涓嶈繃浠嶅簲浠呭湪褰撳墠 benchmark 浠ｉ檯鍐呴儴瑙ｈ銆?- 2026-03-17锛氳 `Array.prototype.push` 鐨?native 蹇矾寰勮兘澶熺洿鎺ュ悶鎺夊悗缁殑 `Drop`锛屼娇璇彞浣嶇疆鐨?`arr.push(...)` 涓嶅啀鐧界櫧鍘嬪叆涓€涓┈涓婂氨浼氳涓㈠純鐨勮繑鍥為暱搴︺€?- 閲嶆柊璺戜簡 `Array.prototype.push` 杩斿洖鍊艰涔夌浉鍏冲洖褰掕鐩栵紝缁撴灉閫氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `array push 10k`锛歚0.532鈥?.539 ms`
  - `sieve 10k`锛歚1.640鈥?.670 ms`
  - `method_chain 5k`锛歚0.606鈥?.618 ms`
- 褰撳墠瑙ｈ锛氳繖鏄竴娆″緢鍊肩殑绐勮寖鍥翠紭鍖栵紝鍥犱负瀹冩涓暟缁勬瀯寤哄惊鐜噷鏈€鐑殑璇彞褰㈡€侊紝鍚屾椂鍙堜笉鏀瑰彉琛ㄨ揪寮忎綅缃殑璇箟銆?- 2026-03-17锛氭妸 `Call` / `CallMethod` / builtin-as-function 鐨?native/builtin 灏忓弬鏁板揩璺緞浠?`argc <= 2` 鎵╁埌 `argc == 3`锛岀户缁幓鎺変簡涓夊弬鏁板師鐢熻皟鐢ㄥ舰鐘朵笂娈嬬暀鐨勪竴灞?`Vec<Value>` 鍒嗛厤銆?- 琛ュ厖浜嗕笁鍙傛暟 native 璋冪敤椤哄簭鐨勫洖褰掕鐩栵紙`Math.max(1, 4, 2)`锛夈€?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `array push 10k`锛歚0.472鈥?.481 ms`
  - `json parse 1k`锛歚0.732鈥?.749 ms`
  - `method_chain 5k`锛歚0.590鈥?.604 ms`
- 褰撳墠瑙ｈ锛氬綋鍓嶄富 benchmark 闆嗗悎杩樻病鏈夋樉绀哄嚭涓€鏉″叏鏂扮殑銆佸彧灞炰簬 `json` 杩欎竴绫荤殑鐙珛鐖嗗彂寮忔敹鐩婏紝浣嗚繖娆℃敼鍔ㄧ‘瀹炶ˉ涓婁簡涓€涓槑鏄炬畫鐣欑殑灏忓弬鏁版暣鐞嗙己鍙ｏ紝鑰屼笖娌℃湁鎷栧潖闄勮繎鐨勮皟鐢ㄥ瘑闆?benchmark銆?
### 9.1.4 Dense array 蹇€熻矾寰?
**浼樺厛绾?*: P0

**鐑偣鏂囦欢**

- `src/vm/interpreter.rs`
- `src/vm/property.rs`
- `src/runtime/array.rs`

**鍘熷洜**

- `array` 鍜?`sieve` 鏄粡鍏哥殑 dense array benchmark銆?- 褰撳墠璁块棶浠嶇劧缁忚繃澶氫釜閫氱敤灞傘€?
**浠诲姟**

- 缂╃煭 `GetArrayEl`銆乣GetArrayEl2` 鍜?`PutArrayEl` 璺緞銆?- 瀵?dense 鏁存暟绱㈠紩璁块棶杩涜鐗瑰寲銆?- 瀵规槑鏄剧殑鏁扮粍鎿嶄綔閬垮厤閫氱敤灞炴€ф煡鎵俱€?- 鍒嗗埆瀹℃煡 `push`銆佺储寮曡鍜岀储寮曞啓璺緞銆?
**棰勬湡鏀剁泭**

- `array` 鐨勪富瑕佹敼杩涚洰鏍?- `sieve` 鐨勫己鍔涢鏈熸敹鐩?
**褰撳墠宸插畬鎴?*

- 2026-03-16锛氬畬鎴愪簡绗竴涓繁灞傚睘鎬т紭鍖栵紝閫氳繃涓哄父瑙勫璞″睘鎬ф煡鎵炬坊鍔犲皬瀵硅薄蹇€熻矾寰勫苟缁熶竴 `GetField` / `GetField2` 灞炴€у垎鍙戙€?- 涓烘繁灞傚睘鎬ч摼璁块棶娣诲姞浜嗗洖褰掕鐩栥€?- Benchmark 缁撴灉锛歚deep_property 200k` 鍦?Criterion 涓粠绾?`28鈥?9 ms` 鎻愬崌鍒?`15.7鈥?7.0 ms`銆?- 閲嶈瑙ｈ锛氳繖閮ㄥ垎宸插畬鎴愬伐浣滀富瑕佸睘浜庘€滄櫘閫氬璞″睘鎬ц闂紭鍖栤€濓紝骞朵笉鎰忓懗鐫€ dense array 鐨勪笓鐢ㄨ鍐欏揩閫熻矾寰勫伐浣滃凡缁忓畬鎴愩€?- 2026-03-17锛氫负 `PutArrayEl + Drop` 娣诲姞浜?peephole 蹇矾寰勶紝浣胯鍙ヤ綅缃殑鏁扮粍璧嬪€间笉鍐嶆妸涓€涓殢鍚庣珛鍒讳涪寮冪殑缁撴灉鍊煎帇鍥炴爤涓娿€?- 閲嶆柊璺戜簡鏁扮粍璧嬪€艰鍙ュ拰璧嬪€艰〃杈惧紡鐩稿叧鍥炲綊瑕嗙洊锛岀粨鏋滈€氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `array push 10k`锛歚0.609鈥?.621 ms`
  - `sieve 10k`锛歚2.045鈥?.084 ms`
- 褰撳墠瑙ｈ锛氳繖鏄竴娆″皬浣嗗共鍑€鐨?dense-array 鍐欒矾寰勪紭鍖栵紝鐗瑰埆閽堝 `sieve` 閲?`primes[j] = false;` 杩欑楂橀璇彞褰㈢姸銆?- 2026-03-18锛氭妸鏇寸獎鐨?`PushFalse; PutArrayEl; Drop` 璇彞褰㈢姸杩涗竴姝ヤ笓闂ㄥ寲鎴愪簡 `PutArrayElFalseDrop`锛岀洿鎺ョ瀯鍑?`sieve` 鍐呭眰寰幆閲屾渶鐑殑甯冨皵鍐欏叆妯″紡銆?- 琛ュ厖浜?compiler 鍥炲綊瑕嗙洊锛岀‘璁?`arr[idx] = false;` 鐜板湪浼氬彂鍑鸿繖涓柊 opcode锛涘悓鏃跺凡鏈夌殑璧嬪€艰〃杈惧紡鍜屾暟缁勬潯浠惰涔夊洖褰掍粛鐒朵繚鎸侀€氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `sieve 10k`锛歚1.638鈥?.668 ms`
  - `array push 10k`锛歚0.556鈥?.563 ms`
- 褰撳墠瑙ｈ锛氳繖鏄涓€鏉＄湡姝ｇ簿鍑嗚惤鍦?`sieve` 鏍稿績 `primes[j] = false;` 鍐欒矾寰勪笂鐨?dense-array 涓撻棬鍖栵紝鑰屼笖瀹冨張鎶婅繖涓?benchmark 寰€涓嬫帹浜嗕竴姝ワ紝鍚屾椂娌℃湁鏀瑰彉璧嬪€艰〃杈惧紡璇箟銆?- 2026-03-18锛氭妸鏈€鐑殑 `.push` 灞炴€ц鍙栦粠閫氱敤 `GetField2` 閲屾媶鎴愪簡涓撻棬鐨?`GetArrayPush2` opcode锛屼娇鏁扮粍鍒濆鍖栫儹寰幆鍦ㄨ繘鍏?`CallArrayPush1` 涔嬪墠涓嶅啀鏀粯閫氱敤瀛楃涓插睘鎬у垎鍙戞垚鏈€?- 琛ュ厖浜?compiler 鍥炲綊瑕嗙洊锛岀‘璁?`arr.push(...)` 鐜板湪浼氬彂鍑?`GetArrayPush2`锛涘悓鏃舵柊澧炰簡 eval 鍥炲綊锛岄攣瀹?`obj.push(side_effect())` 鍦?`obj` 涓?`null` 鏃朵粛浼氬厛鎶涢敊锛岃€屼笉浼氬厛鎵ц鍙傛暟鍓綔鐢ㄣ€?- 閲嶆柊璺戜簡閽堝鎬х殑 push/fallback 鍥炲綊銆佸叏閲忓紩鎿庢祴璇曚互鍙?`clippy -D warnings`锛岀粨鏋滈兘閫氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `array push 10k`锛歚0.593鈥?.716 ms`
  - `sieve 10k`锛歚1.718鈥?.753 ms`
  - `dense array bool read branch 10k`锛歚1.139鈥?.153 ms`
  - `dense array false write only 10k`锛歚1.500鈥?.770 ms`
- 褰撳墠瑙ｈ锛氳繖娆℃洿鍍忔槸 dense-array 鍒濆鍖栬矾寰勭殑鏀剁泭锛岃€屼笉鏄?`GetArrayEl` 璇昏矾寰勫凡缁忚鐪熸鎵撻€氾紱浣嗗畠骞插噣鍦板幓鎺変簡 `sieve` 椋庢牸鏁扮粍棰勭儹閲屾渶鐑殑閭ｅ眰閫氱敤 `.push` 灞炴€ц鍙栵紝瀵规柊澧炵殑 dense-array 璇婃柇 benchmark 涔熸湁鏄庣‘甯姪銆?- 2026-03-18锛氭柊澧炰簡涓€缁?`dense_array_bool_condition_only_hot` 璇婃柇鑴氭湰/benchmark锛岀敤鏉ユ妸鍙嶅鎵ц鐨?`GetArrayEl + IfFalse` 鎵弿褰㈢姸鍗曠嫭鎷庡嚭鏉ワ紝涓嶅啀娣峰叆 `count = count + 1` 杩欏眰棰濆绱姞宸ヤ綔銆?- `dump_bytecode` 鐜板湪鍙互鐩存帴鐪嬪埌锛岃繖涓?benchmark 浼氳缂栬瘧鎴愭渶鎺ヨ繎鐩爣鐑偣鐨?`GetArrayEl; IfFalse; IncLoc; Goto` 褰㈢姸锛屽畠鏄綋鍓嶆渶骞插噣鐨?dense-array 璇讳晶鍒嗘敮澶嶇幇鍣ㄣ€?- 褰撳墠瑙ｈ锛氬悗缁?dense-array 璇讳晶璋冧紭锛岀幇鍦ㄥ簲璇ュ悓鏃跺弬鑰冧笁绉嶇敱鈥滄贩鍚堚€濆埌鈥滅函鈥濈殑璇婃柇褰㈢姸锛歚dense array bool read branch 10k`銆乣dense array bool read hot`銆乣dense array bool condition only hot`銆?- 2026-03-18锛氱户缁柊澧炰簡涓€缁?`dense_array_read_only_hot` 璇婃柇鑴氭湰/benchmark锛岀敤鏉ユ妸鍙嶅鎵ц鐨?`GetArrayEl; Drop; IncLoc; Goto` 绾鍙栨壂鎻忓崟鐙嫀鍑烘潵锛屽畬鍏ㄤ笉鍐嶆贩鍏ユ潯浠跺垎鏀€?- `dump_bytecode` 鐜板湪鍙互鐩存帴鐪嬪埌锛岃繖涓?benchmark 鏄綋鍓嶆渶骞插噣鐨勨€滅函鏁扮粍璇诲彇鎴愭湰鈥濆鐜板櫒锛屽彲浠ュ拰 truthiness 鍒嗘敮銆佽鏁扮疮鍔犺繖涓ゅ眰瀹屽叏鍒嗗紑鐪嬨€?- 褰撳墠瑙ｈ锛氬悗缁?dense-array 璇讳晶璋冧紭锛岀幇鍦ㄥ簲璇ュ悓鏃跺弬鑰冨洓绉嶇敱鈥滄贩鍚堚€濆埌鈥滅函鈥濈殑璇婃柇褰㈢姸锛?  - `dense array bool read branch 10k`
  - `dense array bool read hot`
  - `dense array bool condition only hot`
  - `dense array read only hot`
- 2026-03-19锛氫繚鐣欎簡 `GetArrayElDiscard`锛屼笓闂ㄦ壙鎺ヨ绔嬪埢涓㈠純缁撴灉鐨勮鍙ュ舰鏁扮粍璇诲彇锛坄arr[idx];`锛夛紝杩欐牱鈥滅函璇诲彇鈥濊瘖鏂矾寰勪笉鍐嶉澶栨敮浠橀€氱敤 `Drop` 鐨勬垚鏈€?- 琛ュ厖浜?compiler/eval 鍥炲綊瑕嗙洊锛岀‘璁よ涓㈠純鐨勬暟缁勮鍙栫幇鍦ㄤ細闄嶅埌杩欎釜涓撻棬 opcode锛屽悓鏃跺懆鍥寸▼搴忚涓轰繚鎸佷笉鍙樸€?- 2026-03-19锛氫繚鐣欎簡涓撻棬鐨?`GetArrayElDiscard` 璇彞褰㈣鍙?opcode锛岀敤鏉ユ壙鎺ヨ绔嬪埢涓㈠純缁撴灉鐨?`arr[idx];`銆傝繖鏍峰彲浠ュ湪涓嶆敼鍙樿〃杈惧紡鎴栧垎鏀涔夌殑鍓嶆彁涓嬶紝淇濈暀涓€鏉℃洿绾殑鈥滃彧鐪嬭鍙栤€濊瘖鏂矾寰勩€?- 琛ュ厖浜嗗洖褰掕鐩栵紝纭琚涪寮冪殑鏁扮粍璇诲彇浠嶇劧浼氭甯告眰鍊煎苟缁х画鎵ц鍚庣画璇彞銆?- 2026-03-18锛氭渶鍚庡張鏂板浜嗕竴缁?`dense_array_loop_only_hot` 璇婃柇鑴氭湰/benchmark锛屾妸鏁扮粍璇诲彇鏈韩涔熷畬鍏ㄥ墺鎺夛紝鍙祴鍙嶅鎵ц鐨?`Lte; IncLoc; Goto` 绾惊鐜鏋躲€?- `dump_bytecode` 鐜板湪鍙互鐩存帴鐪嬪埌锛岃繖涓熀绾夸細琚紪璇戞垚鏈€绾殑 `PushI16; Lte; IfFalse; IncLoc; Goto` 褰㈢姸銆?- 褰撳墠瑙ｈ锛歞ense-array 璇讳晶鐜板湪宸茬粡鍙互琚垎瑙ｆ垚浜斿眰璇婃柇锛?  - `dense array loop only hot`
  - `dense array read only hot`
  - `dense array bool condition only hot`
  - `dense array bool read hot`
  - `dense array bool read branch 10k`
  姣忓線鍚庝竴灞傦紝閮藉彧姣斿墠涓€灞傚鍑虹湡瀹炲伐浣滆礋杞戒腑鐨勪竴鍧楁垚鏈€?- 2026-03-19锛氱户缁柊澧炰簡鎴愬鐨?`arg0` / `local1` 瀵圭収璇婃柇锛岀敤鏉ユ妸鈥滄暟缁勫湪 `local0`鈥濆拰鈥滄暟缁勫湪闈?0 妲戒綅鈥濆垎寮€娴嬶細
  - `dense array read only hot arg0`
  - `dense array read only hot local1`
  - `dense array bool condition only hot arg0`
  - `dense array bool condition only hot local1`
- 褰撳墠瑙ｈ锛氬湪鎺у埗浜嗗嚱鏁板瓧鑺傜爜褰㈢姸鍜屽弬鏁颁釜鏁颁箣鍚庯紝`local0` 涓庨潪 0 妲戒綅涔嬮棿鍓╀綑鐨勫樊鍒幇鍦ㄧ湅璧锋潵鏇村儚灏忚€屽櫔澹板寲鐨勫洜绱狅紝`GetLoc0` 杩樹笉瓒充互琚垽鏂负褰撳墠璇讳晶鐨勪富鐡堕銆?- 2026-03-19锛氭柊澧炰簡 `analyze_dense_array_layers`锛岀敤鏉ュ湪涓€娆¤繍琛岄噷鐩存帴缁欏嚭鍚勮瘖鏂眰涔嬮棿鐨?delta锛岄伩鍏嶅悗缁?read-side 浼樺寲鍐嶆墜宸ユ嫾澶氱粍 benchmark 杈撳嚭鏉ュ仛褰掑洜銆?- 褰撳墠瑙ｈ锛氬悗缁湪缁х画鎵?`GetArrayEl` 涔嬪墠锛屼紭鍏堢敤 `analyze_dense_array_layers` 鍒ゆ柇鈥滅湡姝ｈ繕璐电殑鏄摢涓€灞傗€濓紝鍐嶅喅瀹氳涓嶈鍔ㄦ墽琛屽櫒銆?- 2026-03-19锛氭妸 dense-array 绱㈠紩璁块棶閲屽弽澶嶅嚭鐜扮殑鈥滄暟缁勫€?+ 闈炶礋鏁村瀷绱㈠紩鈥濊В鐮佹敹鎴愪簡涓€鏉″叡浜殑 raw 蹇矾寰?helper锛屽苟璁?`GetArrayEl`銆乣GetArrayElDiscard`銆乣PutArrayEl`銆乣PutArrayElFalseDrop` 鍏辩敤瀹冦€?- 閲嶆柊璺戜簡 `cargo check -p mquickjs-rs`銆佸叏閲?`cargo test -p mquickjs-rs` 浠ュ強 `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`锛岀粨鏋滈兘閫氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `sieve 10k`锛歚1.3173鈥?.3389 ms`
  - `dense array bool read hot`锛歚68.603鈥?9.468 ms`
  - `dense array bool condition only hot`锛歚57.001鈥?8.300 ms`
- 褰撳墠瑙ｈ锛氳繖娆℃敹绱х粓浜庣洿鎺ヨ惤鍦?dense-array 绱㈠紩璇诲啓 opcode 鏈綋鍐呴儴锛岃€屼笉鍐嶅彧鏄洿缁曞惊鐜鏋舵垨鏁扮粍鍒濆鍖栬矾寰勫仛澶栧洿浼樺寲銆?- 2026-03-19锛氱户缁妸 `GetArrayEl` 鐨勫垎鏀?bookkeeping 鏀剁揣浜嗕竴灞傦紝鎶婅瀺鍚堝悗鐨?`GetArrayEl + IfFalse/IfTrue` 鐑矾寰勯噷涓存椂鐨?`Option<(bool, i32)>` 鍒嗘敮绐ユ帰鐘舵€佹敼鎴愪簡鏇寸洿鎺ョ殑 opcode/offset 灞€閮ㄥ彉閲忓舰鐘躲€?- 閲嶆柊璺戜簡鍏ㄩ噺 `cargo test -p mquickjs-rs` 鍜?`cargo clippy -p mquickjs-rs --all-targets -- -D warnings`锛岀粨鏋滈兘閫氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `dense array bool read hot`锛歚81.896鈥?7.290 ms`
  - `dense array bool condition only hot`锛歚71.628鈥?7.040 ms`
  - `sieve 10k`锛歚1.6203鈥?.7606 ms`
- 褰撳墠瑙ｈ锛氳繖鏄竴鏉￠潪甯哥獎鐨?`GetArrayEl` 鍐呴儴鏁寸悊锛屽畠瀵硅瀵嗛泦鐨勮瀺鍚堝垎鏀?workload 鏈夋鍚戝府鍔╋紝鍚屾椂娌℃湁鎶婃洿骞夸箟鐨?`sieve` 涓昏矾寰勬嫋鎴愬彲娴嬪洖褰掋€?- 2026-03-19锛氱户缁妸铻嶅悎鍚庣殑 `GetArrayEl + IfFalse/IfTrue` 鍒嗘敮绐ユ帰鍐嶆敹绱т簡涓€灞傦紝鎶婄儹璺緞閲岀殑 opcode/offset 瑙ｇ爜鏀规垚浜嗏€滃崟娆￠暱搴︽鏌?+ unchecked 鍙栧瓧鑺傗€濈殑褰㈢姸銆?- 閲嶆柊璺戜簡 `cargo check -p mquickjs-rs`銆佸叏閲?`cargo test -p mquickjs-rs` 浠ュ強 `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`锛岀粨鏋滈兘閫氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `sieve 10k`锛歚1.3077鈥?.3308 ms`
  - `dense array bool read hot`锛歚73.169鈥?4.204 ms`
  - `dense array bool condition only hot`锛歚59.915鈥?0.797 ms`
- 褰撳墠瑙ｈ锛氳繖鍙堟槸涓€鏉″緢绐勭殑 `GetArrayEl` bookkeeping 鏀剁泭锛岃€屼笖杩欐鍦ㄢ€滅函鏉′欢鎵弿鈥濆拰鈥滆 + 绱姞鈥濅袱绫?dense-array 璇讳晶 workload 涓婇兘鑳界ǔ瀹氱湅鍒版敼鍠勩€?- 2026-03-19锛氭妸 `GetArrayElDiscard` 鐨?dense-array 蹇矾寰勫垽瀹氫粠浼氳繑鍥?tuple 鐨?`dense_array_access()` 閲屾媶浜嗗嚭鏉ワ紝鍗曠嫭鍋氭垚浜嗘洿杞荤殑甯冨皵鍒ゆ柇锛岃繖鏍疯绔嬪埢涓㈠純缁撴灉鐨勭函璇诲彇璺緞涓嶅啀涓虹敤涓嶄笂鐨?`(arr_idx, index)` 瑙ｇ爜浠樿垂銆?- 閲嶆柊璺戜簡 `cargo check -p mquickjs-rs`銆佸叏閲?`cargo test -p mquickjs-rs` 浠ュ強 `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`锛岀粨鏋滈兘閫氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `dense array read only hot`锛歚51.587鈥?2.737 ms`
  - `dense array read only hot arg0`锛歚50.887鈥?1.752 ms`
  - `dense array read only hot local1`锛歚52.082鈥?2.993 ms`
  - `sieve 10k`锛歚1.3022鈥?.3268 ms`
- 褰撳墠瑙ｈ锛氳繖鏄竴鏉″皬鑰岀湡瀹炵殑绾鍙栬矾寰勬敹鐩婏紝瀹冪户缁妸鏇寸函鐨?`GetArrayElDiscard` 璇婃柇鍩虹嚎寰€涓嬪帇锛屽悓鏃舵病鏈夋壈鍔ㄦ洿骞夸箟鐨勬暟缁勮涔夈€?- 2026-03-19锛氱户缁斁瀹戒簡 `GetArrayElDiscard` 鐨?dense-array 蹇矾寰勫垽瀹氾紝璁╁畠鍦ㄦ櫘閫氭暟缁勪笂閬囧埌鈥滀换鎰忔暣鍨嬬储寮曗€濈殑琚涪寮冭鍙栨椂閮藉彲浠ョ洿鎺ヨ浣?no-op锛岃€屼笉鍐嶅潥鎸佹部鐢ㄤ骇鐢熷€肩殑鏁扮粍璇诲彇閭ｅ鈥滃繀椤诲厛瑙ｇ爜闈炶礋绱㈠紩鈥濈殑鏉′欢銆?- 琛ュ厖浜?eval 鍥炲綊瑕嗙洊锛岀‘璁よ涓㈠純鐨勮礋绱㈠紩璇诲彇锛坄arr[-1];`锛変粛鐒跺彧浼氱瓑浠蜂簬涓€娆¤蹇界暐鐨?`undefined` 璇诲彇锛岀▼搴忎細姝ｅ父缁х画鎵ц銆?- 閲嶆柊璺戜簡 `cargo check -p mquickjs-rs`銆侀拡瀵规€х殑 eval 鍥炲綊浠ュ強 `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`锛岀粨鏋滈兘閫氳繃銆傚叏閲?`cargo test -p mquickjs-rs` 涔熷凡閫氳繃锛屽彧鏄悗缁張閬囧埌浜嗕笌杩欐鏀瑰姩鏃犲叧鐨?Windows linker/pagefile 娉㈠姩銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `dense array read only hot`锛歚51.587鈥?2.737 ms`
  - `dense array read only hot arg0`锛歚50.887鈥?1.752 ms`
  - `dense array read only hot local1`锛歚52.082鈥?2.993 ms`
  - `sieve 10k`锛歚1.3022鈥?.3268 ms`
- 褰撳墠瑙ｈ锛氳繖鍙堟槸涓€鏉￠潪甯哥獎浣嗙ǔ瀹氱殑 `GetArrayElDiscard` 绾璺緞鏀剁泭锛屽畠缁х画鐬勫噯鐨勬槸鏈€绾殑璇诲彇璇婃柇鍩虹嚎锛岃€屼笉鏄洿骞夸箟鐨勫垎鏀瀷 `GetArrayEl` workload銆?- 2026-03-18锛氭柊澧炰簡涓€缁?`IncLoc*Drop` 涓撻棬 opcode锛岀敤鏉ユ壙鎺モ€滅粨鏋滀細琚珛鍒讳涪寮冪殑 `local = local + 1` 璇彞鏇存柊鈥濓紝骞舵妸寰幆澧為噺鍜?dense-array 璇讳晶璁℃暟閲屾渶鐑殑閭ｆ壒灏惧反閲嶅啓鎴愯繖缁勬寚浠ゃ€?- 琛ュ厖浜?compiler 鍥炲綊瑕嗙洊锛岀‘璁?`var i = 0; i = i + 1;` 鐜板湪浼氶檷鍒拌繖缁勪笓闂?opcode锛涘悓鏃舵柊澧炰簡璇箟鍥炲綊锛岄攣瀹?`var x = 'a'; x = x + 1;` 浠嶇劧淇濈暀瀛楃涓叉嫾鎺ヨ涔夈€?- `dump_bytecode` 鐜板湪鍙互鐩存帴鐪嬪埌 dense-array 璇讳晶璇婃柇鏍蜂緥宸茬粡琚紪鎴愭洿绱у噾鐨?`IncLoc{1,2,3,4}Drop` 灏惧反锛岃€屼笉鍐嶅弽澶嶅嚭鐜?`Push1; Add; Dup; PutLocX; Drop` 杩欎覆楠ㄦ灦銆?- 閲嶆柊璺戜簡鍏ㄩ噺寮曟搸娴嬭瘯浠ュ強 `clippy -D warnings`锛岀粨鏋滈兘閫氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `loop 10k`锛歚0.416鈥?.477 ms`
  - `sieve 10k`锛歚1.496鈥?.514 ms`
  - `dense array bool read branch 10k`锛歚0.804鈥?.822 ms`
  - `dense array bool read hot`锛歚69.67鈥?0.63 ms`
  - `dense array false write then read hot`锛歚60.76鈥?1.67 ms`
- 褰撳墠瑙ｈ锛氳繖鏄綋鍓?dense-array 闃舵閲岀涓€鏉＄湡姝ｆ妸鍓╀綑鈥滆渚у惊鐜鏋垛€濇槑鏄惧帇涓嬪幓鐨勪紭鍖栵紱瀹冨缁忓吀 `loop` 鍩虹嚎涔熸湁甯姪锛屼絾鏈€澶х殑鏀惰幏鏄?`GetArrayEl` 鍛ㄥ洿鍘熸湰瑕佹敮浠樼殑澶ч噺灞€閮ㄨ嚜澧?bookkeeping 鎴愭湰鐜板湪鏄庢樉鏇翠綆浜嗐€?- 2026-03-18锛氱户缁敹绱т簡 `GetArrayEl` 鑷韩鐨勫垎鏀瀺鍚堝揩璺緞锛岃鏈€鐑殑鈥滄暟缁?+ 鏁存暟绱㈠紩 + 鏉′欢鍒嗘敮鈥濆舰鐘跺厛鐩存帴澶勭悊 `true` / `false` / `null` / `undefined` / int 杩欎簺甯歌 truthiness锛屽啀鍥為€€鍒伴€氱敤 helper銆?- 閲嶆柊璺戜簡鍏ㄩ噺寮曟搸娴嬭瘯浠ュ強 `clippy -D warnings`锛岀粨鏋滈兘閫氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `sieve 10k`锛歚1.795鈥?.085 ms`锛堟棤鏄捐憲鍙樺寲锛?  - `dense array bool read branch 10k`锛歚0.919鈥?.118 ms`锛堟棤鏄捐憲鍙樺寲锛?  - `dense array bool read hot`锛歚72.35鈥?3.37 ms`
  - `dense array bool condition only hot`锛歚69.42鈥?8.77 ms`
- 褰撳墠瑙ｈ锛氳繖鏄竴鏉″緢灏忕殑璇讳晶 truthiness 浼樺寲锛屽畠鏈€鏄庢樉鐨勬敹鐩婂嚭鐜板湪鏇寸函鐨?`GetArrayEl + IfFalse` 璇婃柇鍩虹嚎涓婏紝鑰屽湪鏇村ぇ鐨勬贩鍚堝舰鐘?benchmark 涓婂熀鏈繚鎸佷腑鎬с€?
### 9.1.5 Opcode dispatch 鏀剁揣 [宸插交搴曞畬鎴怾

**浼樺厛绾?*: P1

**鐑偣鏂囦欢**

- `src/vm/interpreter.rs`

**鍘熷洜**

- `loop` 浠嶇劧琛ㄦ槑鏈夋剰涔夌殑鎸囦护 dispatch 寮€閿€銆?- 鍩轰簬澶у瀷 match 鐨?dispatch 鏄纭笖鍙淮鎶ょ殑锛屼絾鍦ㄦ渶鐑矾寰勪笂浠嶇劧鏄傝吹銆?
**浠诲姟**

- 閫氳繃鍩哄噯椹卞姩鐨勬€ц兘鍒嗘瀽璇嗗埆鏈€鐑殑 10鈥?0 涓?opcode銆?- 鍑忓皯 dispatch 寰幆涓瘡娆¤凯浠ｇ殑宸ヤ綔閲忋€?- 鍑忓皯鐑寚浠や腑閲嶅鐨勮В鐮?鍒嗘敮/閿欒璺緞寮€閿€銆?- 瀵圭畻鏈€佸眬閮ㄥ彉閲忋€佽烦杞拰璋冪敤鎸囦护棣栭€夋湰鍦板揩閫熻矾寰勩€?
**棰勬湡鏀剁泭**

- `loop` 鐨勬渶浣虫瑕佺洰鏍?- 璺ㄥ涓?benchmark 鐨勫箍娉涙敹鐩?
**褰撳墠宸插畬鎴?*

- 2026-03-16锛氭坊鍔犱簡閲嶅 throw/catch 鎺у埗娴佺殑 `try_catch` benchmark 瑕嗙洊銆?- 2026-03-16锛氶€氳繃缁熶竴寮傚父鍒嗗彂骞跺皢閲嶅鐨勫熀浜?pop 鐨勫睍寮€寰幆鏇挎崲涓哄熀浜?`truncate` / `drop_n` 鐨勫睍寮€锛屽噺灏戜簡寮傚父璺敱寮€閿€銆?- 涓哄惊鐜唴閲嶅 throw/catch 娣诲姞浜嗗洖褰掕鐩栥€?- Benchmark 缁撴灉锛歚try_catch 5k` 鍩虹嚎鍦?Criterion 涓褰曚负 `340鈥?49 渭s`銆?- 2026-03-16锛氬湪 `dump` feature 涓嬫坊鍔犱簡鍔熻兘闂ㄦ帶鐨勮繍琛屾椂 opcode 璁℃暟鍣紝骞堕€氳繃 `Context` 瀵瑰鏆撮湶浠ヤ緵鎬ц兘鍒嗘瀽宸ヤ綔浣跨敤銆?- 娣诲姞浜?`dump` 妯″紡鍥炲綊娴嬭瘯锛岀‘淇?opcode 璁℃暟璁板綍鐪熷疄鎵ц鎯呭喌銆?- 杩愯鏃剁儹鐐瑰彂鐜帮細
  - `loop` 鐢?`GetLoc1`銆乣Goto`銆乣Add`銆乣Dup`銆乣Drop`銆乣GetLoc0`銆乣PutLoc0`銆乣PutLoc1`銆乣Lt`銆乣IfFalse` 涓诲銆?  - `sieve` 鐢?`Goto`銆乣Drop`銆乣IfFalse`銆乣GetLoc3`銆乣Add`銆乣Dup`銆乣GetLoc0`銆乣Lte`銆乣GetLoc2`銆乣PutArrayEl`銆乣PutLoc3`銆乣GetArrayEl`銆乣CallMethod` 涓诲銆?- 褰撳墠瑙ｈ锛氫笅涓€涓熀浜庤瘉鎹殑浼樺寲鐩爣鏇村彲鑳芥槸 `Dup/Drop` + 鏈湴瀛樺偍浣跨敤妯″紡鎴栧垎鏀?鎺у埗娴佹垚鏈紝鑰屼笉鏄彟涓€涓复鏃剁殑绠楁湳杈呭姪鍑芥暟璋冩暣銆?- 2026-03-16锛氫负甯歌璇彞鏇存柊妯″紡锛堝 `i = i + 1;`锛夊畬鎴愪簡 `Dup + PutLocX + Drop` peephole 蹇€熻矾寰勩€?- 娣诲姞浜嗗眬閮ㄨ祴鍊艰鍙ユ洿鏂扮殑鍥炲綊瑕嗙洊锛屽悓鏃朵繚鐣欎簡璧嬪€艰〃杈惧紡琛屼负銆?- Benchmark 缁撴灉锛歚loop 10k` 鍦?Criterion 涓粠绾?`0.513鈥?.525 ms` 鎻愬崌鍒?`0.486鈥?.492 ms`銆?- Benchmark 缁撴灉锛歚sieve 10k` 鍦?Criterion 涓粠绾?`2.257鈥?.310 ms` 鎻愬崌鍒?`2.152鈥?.191 ms`銆?- 2026-03-16锛氶€氳繃灏嗛€氱敤妫€鏌ヨ緟鍔╁嚱鏁版浛鎹负鐩存帴蹇€熻矾寰勬爤鎿嶄綔锛屼紭鍖栦簡鐑?`Dup` / `Drop` opcode 澶勭悊绋嬪簭鏈韩銆?- 澶嶇敤浜嗙浉鍚岀殑灞€閮ㄨ祴鍊煎拰璧嬪€艰〃杈惧紡鍥炲綊瑕嗙洊鏉ラ獙璇佹洿鏀广€?- 鏈疆涔嬪悗鐨勫綋鍓嶅熀绾胯褰曞湪 `docs/BENCHMARK_ANALYSIS.md` 涓€?- 2026-03-16锛氫负 `Lt/Lte` 鍚庣揣璺?`IfFalse` / `IfTrue` 娣诲姞浜嗗垎鏀瀺鍚堝揩閫熻矾寰勶紝鍏佽姣旇緝缁撴灉鐩存帴鍒嗘敮鑰屾棤闇€鍦ㄦ爤涓婂疄鍖栦复鏃跺竷灏斿€笺€?- 澶嶇敤浜嗙幇鏈夌殑 `while`銆乣switch` 鍜?`try_catch` 鎺у埗娴佸洖褰掕鐩栨潵楠岃瘉璇箟銆?- Benchmark 缁撴灉锛歚loop 10k` 鍦?Criterion 涓粠绾?`0.502鈥?.514 ms` 鎻愬崌鍒?`0.484鈥?.499 ms`銆?- Benchmark 缁撴灉锛歚sieve 10k` 鍦?Criterion 涓粠绾?`2.164鈥?.207 ms` 鎻愬崌鍒?`2.038鈥?.078 ms`銆?- 2026-03-17锛氬湪 dump 妯″紡 profiling 鏄庣‘鎸囧嚭 `sieve` 褰撳墠鏈€鐑殑灞€閮ㄦ洿鏂板舰鐘舵槸 `Add; Dup; PutLoc3; Drop` 涓?`Add; Dup; PutLoc8 4; Drop` 涔嬪悗锛屾柊澧炰簡鍙鐩栬繖涓ょ褰㈢姸鐨勭獎鑼冨洿 peephole锛岃€屾病鏈夐噸鏂板紩鍏ヤ箣鍓嶄細鍥炲綊鐨勬硾鍖栫増鏈€?- 琛ュ厖浜?`PutLoc8` 璇彞鏇存柊褰㈢姸鐨勫洖褰掕鐩栵紝鍚屾椂淇濇寔璧嬪€艰〃杈惧紡璇箟涓嶅彉銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `loop 10k`锛歚0.493鈥?.503 ms`
  - `sieve 10k`锛歚1.832鈥?.860 ms`
- 褰撳墠瑙ｈ锛氳繖鍐嶆璇存槑锛屽綋鍓嶉樁娈电殑 opcode / local-store 浼樺寲鍦ㄦ湁鏄庣‘瀛楄妭鐮佸舰鐘惰瘉鎹椂鏁堟灉鏈€濂斤紝涓嶉€傚悎鐢ㄨ繃娉涚殑閫氱敤蹇矾寰勫幓瑕嗙洊銆?- 2026-03-17锛氱户缁敹绱т簡鍘熷 `Goto` / `IfFalse` / `IfTrue` 澶勭悊鍣ㄦ湰韬紝鎶婃渶鐑矾寰勪笂鐨勬搷浣滄暟瑙ｇ爜鍜屽垎鏀€煎脊鏍堟敼鎴愭洿鐩存帴鐨?unchecked 蹇矾寰勩€?- 鍙樻洿鍚庨噸鏂拌窇浜嗗叏閲忓紩鎿庢祴璇曚互鍙?`clippy -D warnings`锛岀粨鏋滈€氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `loop 10k`锛歚0.461鈥?.476 ms`
  - `sieve 10k`锛歚1.704鈥?.740 ms`
- 褰撳墠瑙ｈ锛氬湪鎸夊叿浣撳瓧鑺傜爜褰㈢姸浼樺寲瀹屽眬閮ㄦ洿鏂颁箣鍚庯紝鐪熸鍓╀笅鐨勪笅涓€灞傜摱棰堝氨鏄帶鍒舵祦楠ㄦ灦鏈韩锛涙妸 `Goto/IfFalse/IfTrue` 鍐嶆敹绱т竴杞箣鍚庯紝`loop` 鍜?`sieve` 閮藉張涓嬩簡涓€涓彴闃躲€?- 2026-03-17锛氭柊澧炰簡涓撻棬鐨?`GetLoc4` / `PutLoc4` 鐭?opcode锛屼娇褰撳墠鏈€鐑殑鈥滈澶栧眬閮ㄦЫ浣嶁€濅笉鍐嶈蛋閫氱敤鐨?`GetLoc8` / `PutLoc8` 璺緞銆?- 琛ュ厖浜?compiler 鍥炲綊瑕嗙洊锛岀‘淇濈 5 涓眬閮ㄦЫ浣嶇幇鍦ㄧ‘瀹炰細鍙戝嚭鐭?opcode銆?- 鍙樻洿鍚庨噸鏂拌窇浜嗗叏閲忓紩鎿庢祴璇曚互鍙?`clippy -D warnings`锛岀粨鏋滈€氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `loop 10k`锛歚0.449鈥?.459 ms`
  - `sieve 10k`锛歚1.686鈥?.714 ms`
- 褰撳墠瑙ｈ锛氬湪鎺у埗娴侀鏋舵敹绱т箣鍚庯紝涓嬩竴灞傜湡瀹炵摱棰堢‘瀹炲氨鏄渶鐑殑閭ｄ釜闈炲唴鑱斿眬閮ㄦЫ浣嶏紱缁?slot 4 琛ヤ笂涓撻棬 opcode 涔嬪悗锛宍loop` 鍜?`sieve` 閮藉張寰€涓嬭蛋浜嗕竴姝ャ€?- 2026-03-17锛氬湪棰濆楠岃瘉涔嬪悗锛屼繚鐣欎簡 slot 4 鐭?opcode 杩欐潯浼樺寲锛屽苟鍦ㄥ綋鍓嶅伐浣滄爲涓婇噸鏂拌窇浜嗘湰鍦?benchmark 瀵圭収銆?- 褰撳墠閫夊畾鐨勬墽琛屾湡閲嶈窇缁撴灉涓猴細
  - `loop 10k`锛歚0.444鈥?.451 ms`
  - `sieve 10k`锛歚1.663鈥?.709 ms`
- 褰撳墠瑙ｈ锛歴lot 4 鐭?opcode 杩欐潯绾垮湪澶嶈窇鍚庝粛鐒舵垚绔嬶紝搴旇涓虹ǔ瀹氱殑 opcode/local-slot 浼樺寲鎴愭灉锛岃€屼笉鏄竴娆℃€х殑娴嬮噺娉㈠姩銆?- 2026-03-18锛氭柊澧炰簡涓€涓皬鍨?`dump_bytecode` 寮€鍙戣€呭伐鍏凤紝浣跨儹鐐?benchmark 鑴氭湰鐜板湪鍙互鐩存帴缂栬瘧骞跺弽姹囩紪锛岃€屼笉鍐嶅彧鑳介潬 opcode 璁℃暟鍘诲弽鎺ㄥ瓧鑺傜爜褰㈢姸銆?- 2026-03-18锛氶噸鏂拌皟鏁翠簡 C-style `for` 鐨?lowering 鏂瑰紡锛氬閲忔浠嶇劧鍙紪璇戜竴娆★紝浣嗘敼涓鸿拷鍔犲埌 body 涔嬪悗锛屼粠鑰屽幓鎺変簡棣栬疆鎵ц鏃堕偅鏉♀€滃厛璺宠繃澧為噺娈碘€濈殑棰濆 `Goto`銆?- 琛ュ厖浜?compiler 鍥炲綊瑕嗙洊锛岀‘璁ょ畝鍗?`for (...)` 寰幆鐜板湪鍙細鍙戝嚭涓€鏉″洖杈?`Goto`锛涘悓鏃堕噸鏂拌窇閫氫簡 `for` / `continue` / `break` / 鈥渓oop 鍐?switch鈥?鐩稿叧鍥炲綊銆?- 鍙樻洿鍚庨噸鏂拌窇浜嗗叏閲忓紩鎿庢祴璇曚互鍙?`clippy -D warnings`锛岀粨鏋滈€氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `loop 10k`锛歚0.555鈥?.659 ms`
  - `sieve 10k`锛歚1.917鈥?.219 ms`
  - `dense array bool read branch 10k`锛歚1.367鈥?.669 ms`
  - `dense array bool read hot`锛歚131.93鈥?52.53 ms`
  - `dense array false write then read hot`锛歚87.45鈥?00.89 ms`
- 褰撳墠瑙ｈ锛氳繖娆℃洿鍍忔槸鎺у埗娴侀鏋跺眰闈㈢殑鏀剁泭锛岃€屼笉鏄彧鎵撲腑浜?`GetArrayEl` 鏈綋锛涗絾瀹冩濂借惤鍦?dense-array 璇讳晶璇婃柇閲岀幇鍦ㄦ渶閲嶇殑閭ｇ被瀛楄妭鐮佸舰鐘朵笂锛屼篃鎶婃墍鏈?C-style `for` 寰幆閲屼竴涓湡瀹炲瓨鍦ㄧ殑缁撴瀯鎬у啑浣欏幓鎺変簡銆?- 2026-03-18锛氭柊澧炰簡 `IncLoc*Drop` 杩欎竴缁勨€滆鍙ュ舰灞€閮ㄨ嚜澧炩€?opcode锛屽苟鎶婃渶鐑殑銆佺粨鏋滀細琚涪寮冪殑 `local = local + 1` 瀛楄妭鐮佸熬宸撮噸鍐欐垚杩欑粍涓撻棬鎸囦护锛屽悓鏃朵繚鐣欎簡瀛楃涓插眬閮ㄥ彉閲忎笂鐨勫畬鏁?`+` 璇箟銆?- 琛ュ厖浜?compiler 鍥炲綊瑕嗙洊锛岀‘璁?`var i = 0; i = i + 1;` 鐜板湪浼氶檷鍒颁笓闂?opcode锛涘悓鏃舵柊澧炰簡璇箟鍥炲綊锛岄攣瀹?`var x = 'a'; x = x + 1;` 浠嶇劧浼氬緱鍒板瓧绗︿覆缁撴灉銆?- `dump_bytecode` 鐜板湪鍙互鐩存帴鐪嬪埌 dense-array 璇讳晶璇婃柇鏍蜂緥宸茬粡琚紪鎴愭洿绱у噾鐨?`IncLoc{1,2,3,4}Drop` 灏惧反锛岃€屼笉鍐嶅弽澶嶅嚭鐜?`Push1; Add; Dup; PutLocX; Drop` 杩欎竴鏁翠覆楠ㄦ灦銆?- 鍙樻洿鍚庨噸鏂拌窇浜嗗叏閲忓紩鎿庢祴璇曚互鍙?`clippy -D warnings`锛岀粨鏋滈€氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `loop 10k`锛歚0.483鈥?.602 ms`
  - `sieve 10k`锛歚1.633鈥?.726 ms`
  - `dense array bool read branch 10k`锛歚0.803鈥?.822 ms`
  - `dense array bool read hot`锛歚86.07鈥?05.53 ms`
  - `dense array false write then read hot`锛歚64.41鈥?7.69 ms`
- 褰撳墠瑙ｈ锛氳繖鏄綋鍓?dense-array 闃舵閲岀涓€鏉＄湡姝ｆ妸鍓╀綑鈥滆渚у惊鐜鏋垛€濇槑鏄惧帇涓嬪幓鐨勪紭鍖栵紱瀹冨缁忓吀 `loop` 鍩虹嚎涔熸湁甯姪锛屼絾鏈€澶х殑鏀惰幏鏄柊澧炵殑 dense-array 璇讳晶璇婃柇 benchmark 鐜板湪鍦?`GetArrayEl` 鍛ㄥ洿鏀粯鐨勫眬閮ㄨ嚜澧?bookkeeping 鎴愭湰鏄庢樉鏇翠綆浜嗐€?- 鐘舵€侊細浣滀负褰撳墠杩欎竴杞?dispatch 鏀剁揣宸ヤ綔锛岃繖閮ㄥ垎鐜板湪鍙互瑙嗕负瀹屾垚锛涘彧鏈夊湪鏂扮殑 profiling 鏄庣‘鎸囧嚭鍙︿竴缁?materially different 鐑?opcode 鏃讹紝鎵嶉渶瑕侀噸鏂版墦寮€銆?
### 9.1.6 绠楁湳/姣旇緝寰紭鍖栬疆娆?[宸插交搴曞畬鎴怾

**浼樺厛绾?*: P1

**鐑偣鏂囦欢**

- `src/vm/ops.rs`

**鍘熷洜**

- 鏍稿績绠楁湳鍜屾瘮杈冭緟鍔╁嚱鏁板凡缁忛儴鍒嗗唴鑱斻€?- 姝ら鍩熶粛鐒堕噸瑕侊紝浣嗗叾鍙兘鐨勬敹鐩婁綆浜庤皟鐢?鏁扮粍/native 鐑矾寰勩€?
**浠诲姟**

- 瀹¤鍓╀綑鐨勭儹 `op_*` 杈呭姪鍑芥暟鏄惁鐪熸鍙楃泭浜庡唴鑱斻€?- 鍑忓皯甯歌 int/int 鍜?int/float 璺緞涓婄殑閲嶅鏁板€煎己鍒惰浆鎹€?- benchmark 娓呯悊鍚庨噸鏂版鏌ョ浉绛夋€у拰姣旇緝蹇€熻矾寰勩€?
**棰勬湡鏀剁泭**

- 灏忓箙浣嗗箍娉涚殑鏀瑰杽

**褰撳墠宸插畬鎴?*

- 2026-03-16锛氶€氳繃灏嗘渶缁堣繍琛屾椂瀛楃涓叉瀯寤哄湪鍗曚釜杈撳嚭缂撳啿鍖轰腑锛岃€屼笉鏄厛灏嗕袱涓搷浣滄暟瀹炲寲涓轰复鏃舵嫢鏈夌殑 `String` 鍊硷紝鏀瑰杽浜嗗瓧绗︿覆鎷兼帴鐑矾寰勩€?- 涓烘贩鍚堝瓧绗︿覆/鏁板瓧閾惧紡鎷兼帴褰㈢姸娣诲姞浜嗗洖褰掕鐩栥€?- Benchmark 缁撴灉锛歚runtime_string_pressure 4k` 鍦?Criterion 涓粠绾?`2.89鈥?.38 ms` 鎻愬崌鍒?`1.53鈥?.55 ms`銆?- 2026-03-17锛氫负鏈€甯歌鐨?`string + int` / `int + string` 鎷兼帴褰㈢姸娣诲姞浜嗘洿绐勭殑 `Add` 蹇矾寰勶紝璁╂贩鍚堢紪璇戞湡瀛楃涓茬墖娈靛拰鍗佽繘鍒跺惊鐜储寮曠殑杩愯鏃跺瓧绗︿覆鐑偣涓嶅啀璧伴€氱敤鐨勯暱搴︿及绠楀姞杩藉姞璺緞銆?- 閲嶆柊璺戜簡閽堝鎬х殑 concat 褰㈢姸鍥炲綊瑕嗙洊锛岀粨鏋滈€氳繃銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `runtime_string_pressure 4k`锛歚1.091鈥?.117 ms`
  - `string concat 1k`锛歚151.87鈥?57.61 碌s`
  - `method_chain 5k`锛歚587.80鈥?99.99 碌s`
- 褰撳墠瑙ｈ锛氳繖鏄竴鏉″鈥滅紪璇戞湡瀛楃涓茬墖娈?+ 鍗佽繘鍒跺惊鐜储寮曗€濆舰鐘堕潪甯告湁鏁堢殑杩愯鏃跺瓧绗︿覆浼樺寲锛涜€屾洿绠€鍗曠殑 `string concat 1k` benchmark 杩欒疆鍩烘湰娌℃湁鏄捐憲鍙樺寲銆?- 2026-03-17锛氭柊澧炰簡瀛楄妭鐮佺骇鐨?`AddConstStringLeft` / `AddConstStringRight` 涓撻棬鍖栵紝璁?concat 閾鹃噷鈥滅紪璇戞湡瀛楃涓插湪 `+` 宸︿晶鎴栧彸渚р€濈殑褰㈢姸涓嶅啀缁х画璧伴€氱敤 `Add` opcode銆?- 琛ュ厖浜?compiler 鍥炲綊瑕嗙洊锛岀‘璁?`"x" + value` 鍜?`value + "x"` 杩欎袱绫诲舰鐘剁幇鍦ㄩ兘浼氬彂鍑轰笓闂ㄥ瓧鑺傜爜锛涘悓鏃堕噸鏂拌窇浜嗛拡瀵规€х殑 concat 褰㈢姸鍥炲綊瑕嗙洊銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `runtime_string_pressure 4k`锛歚1.055鈥?.077 ms`
  - `string concat 1k`锛歚141.41鈥?45.80 碌s`
  - `method_chain 5k`锛歚587.46鈥?01.19 碌s`
- 褰撳墠瑙ｈ锛氳繖鏄涓€杞湡姝ｆ洿鎴愪綋绯荤殑 concat 閾句紭鍖栵紝涓嶅啀鍙槸鎵ц鍣ㄩ噷鐨?`Add` 灏忓垎鏀井璋冿紱瀹冨 runtime-string 鍘嬪姏璺緞缁欏嚭浜嗘槑纭敹鐩婏紝鍚屾椂娌℃湁鏄庢樉鎷栧潖闄勮繎鐨?`method_chain` 宸ヤ綔璐熻浇銆?- 2026-03-17锛氬湪杩欏眰 lowering 鐨勫熀纭€涓婏紝缁х画鍔犲叆浜嗙浉閭诲瓧绗︿覆瀛楅潰閲忕殑缂栬瘧鏈熸姌鍙狅紝浠ュ強 `const + value + const` 鐨勪笓闂?`AddConstStringSurround` 褰㈢姸锛岃繘涓€姝ュ幓鎺変簡鐩爣 concat 閾鹃噷鐨勪竴娆¤繍琛屾椂瀛楃涓插垎閰嶃€?- 琛ュ厖浜?compiler 鍥炲綊瑕嗙洊锛岀‘璁?surround 涓撻棬鍖栧拰鐩搁偦瀛楃涓插父閲忔姌鍙犻兘宸茬敓鏁堛€?- 褰撳墠宸ヤ綔鏍戜笂鐨?dump 妯″紡鐑偣鎺㈡祴鏄剧ず锛宍runtime_string_pressure` 鐨?concat 杩愯鏃跺瓧绗︿覆鍒涘缓娆℃暟宸茬粡浠?`12001` 闄嶅埌 `8001`锛宍Add` 鎵ц娆℃暟涔熶粠 `24001` 闄嶅埌 `16000`銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `runtime_string_pressure 4k`锛歚0.899鈥?.915 ms`
  - `string concat 1k`锛歚166.97鈥?71.99 碌s`
  - `method_chain 5k`锛歚624.57鈥?38.70 碌s`
- 褰撳墠瑙ｈ锛氳繖鏄竴鏉℃洿寮恒€佹洿缁撴瀯鍖栫殑 concat 閾句紭鍖栵紝瀵圭洰鏍?runtime-string benchmark 鐨勬敹鐩婇潪甯告槑纭紱浣嗗畠鐪嬭捣鏉ヤ細鎷栨參鏇寸畝鍗曠殑 `string concat 1k` 寰熀鍑嗭紝鎵€浠ュ悗缁渶瑕佷笓闂ㄨВ閲婂苟鏀跺洖杩欐潯鍥炲綊锛岃€屼笉鑳芥妸瀛楃涓茶矾寰勭洿鎺ヨ涓衡€滃凡缁忔墦瀹屸€濄€?- 2026-03-18锛氭柊澧炰簡涓€涓潪甯哥獎鐨勮鍙ョ骇 `AppendConstStringToLoc0` lowering锛屽苟閰嶅寮曞叆浜嗕粎鏈嶅姟浜庡眬閮ㄦЫ浣?`0` 鐨?per-frame builder锛屼笓闂ㄨ鐩?`var s = ''; s = s + 'x';` 杩欎竴涓儹鐐瑰舰鐘躲€?- 琛ュ厖浜?compiler 鍥炲綊瑕嗙洊锛岀‘璁よ繖涓€涓簿纭舰鐘剁幇鍦ㄤ細鍙戝嚭鏂?lowering锛涘苟閲嶆柊璺戜簡瀵瑰簲鐨?eval 鍥炲綊銆?- 褰撳墠宸ヤ綔鏍戜笂鐨?dump 妯″紡鐑偣鎺㈡祴鏄剧ず锛宍string_concat` 鐨?concat 杩愯鏃跺瓧绗︿覆鍒涘缓娆℃暟宸茬粡浠?`1000` 闄嶅埌 `1`銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `string concat 1k`锛歚81.24鈥?3.26 碌s`
  - `runtime_string_pressure 4k`锛歚909.74鈥?21.95 碌s`
  - `method_chain 5k`锛歚708.67鈥?24.79 碌s`
- 褰撳墠瑙ｈ锛氳繖鏉″熀浜?builder 鐨勫眬閮ㄨ嚜鎷兼帴浼樺寲锛岀粓浜庢妸 `string concat 1k` 杩欐潯寰熀鍑嗙湡姝ｆ媺涓嬫潵浜嗭紝鑰屼笖娌℃湁鍐嶅洖鍒板墠闈㈤偅浜涢€氱敤杩愯鏃?peephole 鐨勫け璐ヨ矾寰勶紱鍚屾椂鏇村箍涔夌殑 `runtime_string_pressure` 浠嶇劧鍋滅暀鍦ㄥ悓涓€涓簹姣閲忕骇锛岃€?`method_chain` 鐜板湪鏇撮€傚悎鎻忚堪涓衡€滀粛鐒朵繚鎸佸湪浜氭绉掑尯闂粹€濓紝鑰屼笉鏄户缁部鐢ㄦ洿鏃╅偅鏉¤创鐫€ `0.60 ms` 鐨勮娉曘€?- 2026-03-18锛氭妸 concat-chain lowering 鍐嶅線鍓嶆帹浜嗕竴姝ワ紝鏂板浜?`AddConstStringSurroundValue`锛岃鏈€鐑殑 `const + value + const + value` 褰㈢姸涔熻兘鍐嶅皯鎺変竴涓腑闂?concat 缁撴灉銆?- 琛ュ厖浜?compiler 鍥炲綊瑕嗙洊锛岀‘璁?`'a' + x + 'b' + y` 鐜板湪浼氬彂鍑鸿繖涓洓娈?lowering銆?- 褰撳墠宸ヤ綔鏍戜笂鐨?dump 妯″紡鐑偣鎺㈡祴鏄剧ず锛宍runtime_string_pressure` 鐨?concat 杩愯鏃跺瓧绗︿覆鍒涘缓娆℃暟宸茬粡浠?`8001` 杩涗竴姝ラ檷鍒?`4001`锛宍Add` 鎬绘墽琛屾鏁颁篃浠?`16000` 闄嶅埌 `12000`銆?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `runtime_string_pressure 4k`锛歚841.41鈥?55.24 碌s`
  - `string concat 1k`锛歚82.79鈥?4.89 碌s`
  - `string concat ephemeral 1k`锛歚113.15鈥?18.99 碌s`
  - `method_chain 5k`锛歚736.57鈥?51.78 碌s`
- 褰撳墠瑙ｈ锛氳繖璁╁瓧绗︿覆涓荤嚎鍙堟敹绐勪簡涓€灞傦紝鍓╀笅鐪熸娌¤В鍐崇殑閮ㄥ垎鏇村姞闆嗕腑鍒扳€滈€氱敤澧為暱瀛楃涓茶〃绀衡€濇湰韬€傜洰鏍?runtime-string benchmark 鏄庢樉鍙楃泭锛岃€?`string concat` 鍜?`method_chain` 浠嶇劧鐣欏湪鍙帴鍙楃殑鍖洪棿閲屻€?- 2026-03-18锛氬紩鍏ヤ簡涓€涓渶灏忓彲鐢ㄧ殑寤惰繜 `RuntimeString` 鍖呰灞傦紝骞惰 `.length` 鐩存帴璇诲彇 cached runtime-string length锛岃€屼笉鏄厛鎶婂欢杩?concat 鑺傜偣寮哄埗 flatten銆?- 杩欎竴姝ヤ繚浣忎簡鍓嶉潰閭ｄ簺灞€閮ㄨ嚜鎷兼帴鍜?concat-chain lowering 鐨勬敹鐩婏紝鍚屾椂鎶婁箣鍓嶁€滃彇 `.length` 鏃惰寮哄埗鐗╁寲鈥濆甫鍥炴潵鐨勫洖褰掗噸鏂板帇浜嗕笅鍘汇€?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝閫夊畾閲嶈窇缁撴灉涓猴細
  - `runtime_string_pressure 4k`锛歚869.62鈥?87.11 碌s`
  - `string concat 1k`锛歚78.61鈥?0.20 碌s`
  - `string concat ephemeral 1k`锛歚118.18鈥?21.41 碌s`
  - `method_chain 5k`锛歚715.62鈥?30.29 碌s`
- 褰撳墠瑙ｈ锛氬瓧绗︿覆涓荤嚎鐜板湪缁堜簬鏇村儚涓€涓畬鏁寸殑浣撶郴浜嗐€傚眬閮ㄨ嚜鎷兼帴宸茬粡瑙ｅ喅锛岀洰鏍?runtime-string benchmark 鍦?cached-length 璺緞涓嬮噸鏂板彉蹇紝鑰屽墿涓嬬湡姝ｉ渶瑕佸喅绛栫殑锛屾槸涓嶆槸瑕佹妸寤惰繜瀛楃涓茶〃绀虹户缁帹骞匡紝鑰屼笉鍐嶆槸鈥滃綋鍓嶈繖鏉＄獎璺緞鏈夋病鏈夋槑鏄惧洖褰掆€濄€?- 鐘舵€侊細浣滀负褰撳墠杩欎竴杞畻鏈?瀛楃涓叉嫾鎺ュ井浼樺寲闃舵锛岃繖閮ㄥ垎鐜板湪鍙互瑙嗕负瀹屾垚锛涘悗缁鏋滅户缁仛瀛楃涓蹭富绾匡紝搴斿綊绫讳负鏇村箍涔夌殑瀛楃涓茶〃绀烘敼閫犻」鐩紝鑰屼笉鏄户缁妸瀹冨綋浣滈浂纰庣殑鐑?opcode 娓呯悊銆?- 2026-03-16锛氶€氳繃涓哄悓鍊笺€佹暣鏁板拰甯冨皵姣旇緝娣诲姞鐩存帴蹇€熻矾寰勶紙鍦ㄥ洖閫€鍒拌緝鎱㈢殑閫氱敤澶勭悊涔嬪墠锛夛紝鏀瑰杽浜?`StrictEq` / `StrictNeq` 鐑?opcode 澶勭悊銆?- 鐜版湁鐨?switch 璇箟鍥炲綊娴嬭瘯宸叉垚鍔熼噸鏂拌繍琛屻€?- Benchmark 缁撴灉锛歚switch 1k` 鍦?Criterion 涓粠绾?`145鈥?49 渭s` 閲忕骇鎻愬崌鍒?`132鈥?36 渭s`銆?
## 9.2 浼樺寲 GC 鎬ц兘

### 9.2.1 鏇挎崲淇濆畧鐨?`mark_all` 琛屼负 [宸插畬鎴怾

**浼樺厛绾?*: P1

**鐑偣鏂囦欢**

- `src/vm/gc.rs`锛圥lan B 娲昏穬锛?- `src/context.rs`

**鍘熷洜**

- 淇濆畧鐨?`mark_all` 鏂规硶浼氭爣璁版墍鏈夊璞¤€岄潪鍙戠幇鐪熸鐨勬牴銆?
**浠诲姟锛堝凡瀹屾垚锛?*

- 鐢ㄧ湡姝ｇ殑 root 鍙戠幇鏇挎崲 `mark_all()`锛岄€氳繃 `gc_mark_roots_iterative()` 瀹炵幇銆?- 瀹氫箟骞堕亶鍘嗙湡姝ｇ殑 root锛?  - 鏍?/ 娲昏穬璋冪敤甯?  - 鍏ㄥ眬鍙橀噺
  - 閫氳繃 var_cells 鎹曡幏鐨勯棴鍖?  - timers.callback
- GC phase 浠ｉ檯鏍囪 + generation 鏁扮粍銆?- 鍫嗗垎閰嶈凯浠ｅ紡宸ヤ綔闃熷垪锛堟棤鏍堟孩鍑猴級銆?
**棰勬湡鏀剁泭**

- 闄嶄綆 GC 鏆傚仠鎴愭湰
- 鍦ㄥ璞″瘑闆嗗瀷鑴氭湰涓婃湁鏇村ソ鐨勬墿灞曟€?
**宸插畬鎴愯繘搴?*

- 2026-03-19: Plan B GC 鍔熻兘瀹屾暣銆傛墍鏈?GC 鎵樼瀹瑰櫒閫氳繃 `alloc_slot()` 绌洪棽閾捐〃澶嶇敤銆俙Context::gc()` 鍜?native `gc()` 鍧囪Е鍙戠湡瀹炴敹闆嗐€傝嚜鍔?GC 鍦ㄧ湡瀹?GC 鍒嗛厤鐐硅璐广€俙src/gc/collector.rs` 鐜颁负 Plan C stub銆?
### 9.2.2 娴嬮噺 GC 瑙﹀彂琛屼负 [宸插畬鎴怾

**浼樺厛绾?*: P1

**鍘熷洜**

- GC 鎴愭湰涓嶄粎鍙栧喅浜庢敹闆嗗櫒瀹炵幇锛岃繕鍙栧喅浜庤Е鍙戦鐜囥€?
**浠诲姟锛堝凡瀹屾垚锛?*

- 閫氳繃 `gc_count` 娴嬮噺 benchmark 宸ヤ綔璐熻浇鏈熼棿鐨?GC 棰戠巼銆?- `gc_overhead_probe` 浜岃繘鍒舵枃浠舵祴閲忚繍琛屾椂 GC 寮€閿€銆?- 2026-03-21: 鑷姩 GC 瑙﹀彂浠庨€氱敤 JS `Call`/`CallMethod`/`CallConstructor` 璺緞绉诲埌鐪熷疄 GC 鎵樼鍒嗛厤鐐癸紙closures, var_cells, arrays, objects, iterators, typed_arrays, array_buffers, regex, error_objects锛夈€傝繖浠庨珮璋冪敤/浣庡垎閰嶇殑 `fib_iter` 宸ヤ綔璐熻浇涓Щ闄や簡 GC 璁拌处寮€閿€銆?- 閫氳繃 `test_gc_auto_triggers_during_js_function_workload` 楠岃瘉瑙﹀彂琛屼负銆?
### 9.2.3 鍑忓皯寮曟搸鎷ユ湁瀹瑰櫒鐨勬壂鎻忔垚鏈?
**浼樺厛绾?*: P2

**鐑偣鏂囦欢**

- `src/vm/types.rs`
- `src/context.rs`

**浠诲姟**

- 瀹℃煡鎵弿杩愯鏃跺悜閲忕殑鎴愭湰锛?  - `objects`
  - `closures`
  - `runtime_strings`
  - `typed_arrays`
  - `array_buffers`
- 鍦ㄦ湁鐢ㄧ殑鍦版柟灏嗙儹娲诲姩鏁版嵁涓庨暱鏈熷瓨鍦ㄧ殑鍏冩暟鎹垎寮€銆?
## 9.3 鍑忓皯鍐呭瓨浣跨敤

### 9.3.1 棣栧厛鏀瑰杽娴嬮噺 [宸插交搴曞畬鎴怾

**浼樺厛绾?*: P0

**鐑偣鏂囦欢**

- `src/context.rs`
- `src/vm/types.rs`

**鍘熷洜**

- `MemoryStats` 宸茬粡寰堟湁鐢紝浣嗕紭鍖栧簲鍩轰簬瀹為檯鐨勪富瀵兼《銆?
**浠诲姟**

- 灏?`MemoryStats` 浣滀负鍩虹嚎娴嬮噺鏉ユ簮銆?- 璁板綍 benchmark 鑴氭湰鐨勫璞?瀛楃涓?闂寘/typed-array 鏁伴噺銆?- 鍦ㄩ噸鏂拌璁″竷灞€涔嬪墠锛屽厛璇嗗埆鏈€澶х殑鍐呭瓨绫诲埆銆?
**褰撳墠宸插畬鎴?*

- 2026-03-16锛氬皢 `MemoryStats` / `InterpreterStats` 鎵╁睍鍒板璞℃暟閲忎箣澶栵紝鍖呮嫭锛?  - `runtime_string_bytes`
  - `array_elements`
  - `object_properties`
  - `typed_array_bytes`
  - `array_buffers`
  - `array_buffer_bytes`
- 鏇存柊浜?CLI dump 杈撳嚭浠ユ樉绀烘柊鐨勫唴瀛樺垎绫汇€?- 娣诲姞浜嗕互涓嬪洖褰掕鐩栵細
  - 鏁扮粍/瀵硅薄褰㈢姸鎸囨爣
  - 杩愯鏃跺瓧绗︿覆瀛楄妭缁熻
- 鐘舵€侊細姝ゆ祴閲忓熀纭€鐜板湪宸茶冻澶熷紑濮嬪熀浜庤瘉鎹殑 9.3 宸ヤ綔銆?
### 9.3.2 鍑忓皯鐑墽琛岃矾寰勪腑鐨勪复鏃跺垎閰?
**浼樺厛绾?*: P0

**鍘熷洜**

- 涓存椂鍚戦噺鍜岀灛鎬侀噸濉戜細澧炲姞 CPU 鍜屽唴瀛樼殑娉㈠姩銆?
**浠诲姟**

- 浠庣儹璋冪敤璺緞涓Щ闄ゅ墿浣欑殑涓存椂 `Vec<Value>` 鍒嗛厤銆?- 瀹℃煡鏁扮粍/builtin 瀵嗛泦鍨嬫墽琛屼腑鐨勭煭鏈熷垎閰嶆ā寮忋€?- 鍦ㄥ畨鍏ㄧ殑鍦版柟棣栭€変繚鐣欐爤鐨勫竷灞€鍜屽€熺敤鏁版嵁銆?
### 9.3.3 瀹℃煡杩愯鏃跺瓧绗︿覆澧為暱 [宸插交搴曞畬鎴怾

**浼樺厛绾?*: P1

**鐑偣鏂囦欢**

- `src/vm/interpreter.rs`
- `src/context.rs`

**鍘熷洜**

- 杩愯鏃跺瓧绗︿覆鍦?`MemoryStats` 涓鏄庣‘璁℃暟锛屽彲鑳介殢鏃堕棿鎮勬倓澧為暱銆?
**浠诲姟**

- 娴嬮噺 benchmark 宸ヤ綔璐熻浇涓?`runtime_strings` 鐨勫闀裤€?- 妫€鏌ュ瓧绗︿覆鎻愬崌鍦ㄧ儹璺緞涓槸鍚﹁繃浜庣Н鏋併€?- 瀵绘壘閲嶅瀛楃涓插垱寤虹殑鏈轰細銆?
**褰撳墠宸插畬鎴?*

- 2026-03-16锛氭坊鍔犱簡浠呴檺 dump 妯″紡鐨勮繍琛屾椂瀛楃涓叉潵婧愯鏁板櫒锛岃嚦灏戝尯鍒嗭細
  - 鎬昏繍琛屾椂瀛楃涓插垱寤鸿姹?  - concat 椹卞姩鐨勫垱寤?  - for-in key 鍒涘缓
  - 鍏朵粬鍒涘缓璺緞
- 鍦?`dump` feature 涓嬮€氳繃 `Context` 瀵瑰鏆撮湶浜嗚鏁板櫒銆?- 娣诲姞浜?dump 妯″紡鍥炲綊瑕嗙洊锛岀‘淇濊繍琛屾椂瀛楃涓叉潵婧愮粺璁¤璁板綍銆?- 2026-03-17锛氬皢鏉ユ簮妗舵墿灞曞埌鑷冲皯鍖哄垎 `json`銆乣object_keys`銆乣object_entries`銆乣error_string` 鍜?`type_string`锛岄櫎浜?`concat`銆乣for_in_key` 鍜?`other`銆?- 鐘舵€侊細浣滀负鈥滃鏌?娴嬮噺杩愯鏃跺瓧绗︿覆澧為暱鈥濊繖涓€浠诲姟锛岃繖閮ㄥ垎鐜板湪鍙互瑙嗕负瀹屾垚锛涘悗缁槸鍚﹀仛澶嶇敤/鍘婚噸锛屽睘浜庢柊鐨勪紭鍖栧喅绛栵紝鑰屼笉鏄鏌ュ伐浣滄湭瀹屾垚銆?- 宓屽叆璇存槑锛氭殏涓嶅湪寮曟搸涓‖缂栫爜杩愯鏃跺瓧绗︿覆瀛楄妭棰勭畻锛涙渶缁堥檺鍒跺皢鍦?ESP32 绾у埆鐩爣鐨勭湡瀹炶澶囬泦鎴愭湡闂撮€夋嫨銆?- 2026-03-16锛氬湪 `for-in` key 璺緞涓婏紝杩愯鏃跺瓧绗︿覆鑰楀敖鐜板湪鍙樹负鍙楁帶寮曟搸閿欒锛坄runtime string table exhausted`锛夎€屼笉鏄?debug 鏃剁殑婧㈠嚭 panic銆?- 娣诲姞浜嗗洖褰掕鐩栵紝閿佸畾閲嶅 `for-in` key 鐢熸垚鐨勬柊鍙楁帶閿欒琛屼负銆?- 绠€鑰岃█涔嬶細涔嬪墠鍦?`for-in` key 璺緞涓婂穿婧冪殑杩愯鏃跺瓧绗︿覆婧㈠嚭锛岀幇鍦ㄩ檷绾т负鍙楁帶寮曟搸閿欒锛岃€屼笉鏄?panic 杩涚▼銆?
### 9.3.4 瀹℃煡瀵硅薄鍜屾暟缁勫竷灞€寮€閿€

**浼樺厛绾?*: P1

**鐑偣鏂囦欢**

- `src/runtime/object.rs`
- `src/runtime/array.rs`
- `src/vm/types.rs`

**浠诲姟**

- 姣旇緝 dense array 涓庨€氱敤瀵硅薄鏀寔璁块棶鐨勫唴瀛樻垚鏈€?- 妫€鏌ラ绻佸垱寤虹殑杩愯鏃剁粨鏋勬槸鍚﹀彲浠ュ彉灏忋€?- 浠呭湪娴嬮噺涔嬪悗鎵嶉€夋嫨閽堝鎬х殑甯冨眬鍙樻洿銆?
## 杈呭姪寮曟搸浠诲姟

### S1. 淇濇寔 builtin/runtime 杈圭晫璇氬疄

**浼樺厛绾?*: P2

**鍘熷洜**

- `src/builtins/` 鐩墠澶ч儴鍒嗘槸缁撴瀯鎬у崰浣嶄唬鐮併€?- 鐪熸鐨?builtin 琛屼负涓昏鍦?`src/vm/natives.rs` 鍜?`src/vm/property.rs` 涓€?
**浠诲姟**

- 璁板綍鐪熸鐨勫疄鐜颁綅缃€?- 閬垮厤璇紭鍖栧崰浣嶆ā鍧椼€?- 鎺ㄨ繜缁撴瀯鎬ц縼绉荤洿鍒扮儹鐐瑰伐浣滃畬鎴愬悗锛岄櫎闈炲畠闃诲鎬ц兘宸ヤ綔銆?
### S2. 浣跨敤鍩哄噯鐗瑰畾鐨勪紭鍖栫洰鏍?
**浼樺厛绾?*: P0

**鏉冨▉浼樺寲闆嗗悎**

- `fib` 鈫?璋冪敤璺緞銆侀€掑綊銆佺畻鏈?- `loop` 鈫?dispatch銆佺畻鏈€佸眬閮ㄥ彉閲?- `array` 鈫?dense array 蹇€熻矾寰?- `sieve` 鈫?dense array 璇诲啓 + 寰幆鎴愭湰
- `json` 鈫?宸蹭紭绉€璺緞鐨勫洖褰掍繚鎶?
### S3. 鎵╁睍 benchmark 瑕嗙洊浠ヨ鐩栫己澶辩殑寮曟搸璺緞

**浼樺厛绾?*: P0

**鍘熷洜**

- 褰撳墠 benchmark 闆嗗悎鏈夌敤锛屼絾浠嶇劧瀵瑰嚑涓噸瑕佺殑寮曟搸璺緞瑕嗙洊涓嶈冻銆?- 濡傛灉 benchmark 濂椾欢鍙仛鐒﹀湪 `fib`銆乣loop`銆乣array`銆乣sieve` 鍜?`json` 涓婏紝涓€浜涢珮浠峰€肩殑浼樺寲棰嗗煙灏嗕繚鎸佷笉鍙銆?
**Benchmark 鏂板锛氫富闆嗗悎**

杩欎簺搴旇琚涓轰笅涓€鎵?benchmark 鏂板锛屽洜涓哄畠浠渶鐩存帴鍦版毚闇蹭簡鏈夋剰涔夌殑寮曟搸鐑偣锛?
- `method_chain`
  - 浠ｈ〃鎬у舰鐘讹細`.map().filter().reduce()`
  - 瑕嗙洊锛歚GetField2`銆乣CallMethod`銆佸洖璋冭皟鐢ㄣ€佹暟缁勯摼寮忔搷浣?- `for_of_array`
  - 瑕嗙洊锛歚ForOfStart`銆乣ForOfNext`銆佽凯浠ｅ櫒寰幆鎺у埗
- `deep_property`
  - 浠ｈ〃鎬у舰鐘讹細`a.b.c.d`
  - 瑕嗙洊锛氶噸澶嶇殑 `GetField` 鎴愭湰鍜岄摼寮忓睘鎬ц闂?- `runtime_string_pressure`
  - 瑕嗙洊锛歚create_runtime_string`銆佽繍琛屾椂瀛楃涓插闀裤€佸瓧绗︿覆鍒嗛厤鍘嬪姏

**Benchmark 鏂板锛氭闆嗗悎**

杩欎簺涔熷緢閲嶈锛屼絾鏈€濂戒綔涓烘満鍒剁壒瀹氱殑 benchmark 鑰屼笉鏄涓€娉㈠ご鏉℃€ц兘 benchmark锛?
- `try_catch`
  - 瑕嗙洊锛歚ExceptionHandler`銆乼hrow/catch/finally 鎺у埗娴併€佹爤灞曞紑
- `for_in_object`
  - 瑕嗙洊锛歚ForInStart`銆乣ForInNext`銆佸璞?key 杩唬
- `switch_case`
  - 瑕嗙洊锛氬熀浜?`Dup + StrictEq + IfTrue` 鐨勫鍒嗘敮 dispatch 褰㈢姸

**涓哄綋鍓?no_std 浼樺厛璺緞寤跺悗**

- `regexp_test`
  - 瑕嗙洊锛歚RegExpObject`銆乣test`
  - 淇濈暀涓轰互鍚庣殑 `std` / 鍙€?benchmark 鍊欓€夛紝涓嶄綔涓虹涓€娉?no_std 鐩爣
- `regexp_exec`
  - 瑕嗙洊锛歚RegExpObject`銆乣exec`
  - 淇濈暀涓轰互鍚庣殑 `std` / 鍙€?benchmark 鍊欓€夛紝涓嶄綔涓虹涓€娉?no_std 鐩爣

**寤鸿鎺ㄥ嚭椤哄簭**

1. `method_chain`
2. `runtime_string_pressure`
3. `for_of_array`
4. `deep_property`
5. `try_catch`
6. `switch_case`
7. `for_in_object`

寤跺悗锛?
- `regexp_test`
- `regexp_exec`

**棰勬湡浠峰€?*

- 浣垮熀鍑嗛┍鍔ㄧ殑浼樺寲鏇磋兘浠ｈ〃鐪熷疄鐨?JS 浣跨敤鍦烘櫙
- 鏆撮湶璋冪敤瀵嗛泦銆佽凯浠ｅ櫒瀵嗛泦銆佸璞¤闂瘑闆嗗拰瀛楃涓插帇鍔涘瘑闆嗙殑璺緞
- 璁╁紩鎿庝紭鍖栧伐浣滃湪绠楁湳鍜屽師濮嬪惊鐜箣澶栨湁鏇村ソ鐨勫彲瑙佹€?
**褰撳墠宸插畬鎴?*

- 2026-03-16锛氭坊鍔犱簡绗竴娉?benchmark 鑴氭湰鍜?Criterion 瑕嗙洊锛?  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`
- 2026-03-16锛氭坊鍔犱簡绗簩娉?`switch_case` benchmark 鑴氭湰锛岀敤浜?CLI 椋庢牸鐨?Rust vs C 瀵规瘮銆?- 浣跨敤 `cargo bench --no-run` 楠岃瘉浜?benchmark 鏋勫缓銆?- 2026-03-16锛氶€氳繃浠?`ForOfStart` 涓Щ闄ゅ畬鏁存暟缁勫厠闅嗗苟鏀逛负鎸夌储寮曡凯浠ｆ暟缁勶紝瀹屾垚浜嗙涓€涓?`for_of_array` 浼樺寲杞銆?- 娣诲姞浜嗗洖褰掕鐩栵紝纭鏁扮粍涓婄殑 `for-of` 鍦ㄨ凯浠ｆ湡闂磋兘瑙傚療鍒板厓绱犳洿鏂般€?- Benchmark 缁撴灉锛歚for_of_array 20k` 鍦?Criterion 涓粠绾?`4.22鈥?.47 ms` 鎻愬崌鍒?`2.36鈥?.42 ms`銆?- 2026-03-17锛氫负 `ForOfNext` 鍚庣揣璺?`IfTrue` 鐨勫父瑙佸舰鐘舵坊鍔犱簡鍒嗘敮铻嶅悎蹇矾寰勶紝浣胯凯浠ｅ櫒鐑矾寰勪笉鍐嶄负宸茬煡鍒嗘敮褰㈢姸鐗╁寲涓存椂 `done` 甯冨皵鍊笺€?- 閲嶆柊璺戜簡 `for-of` 鐨勬甯歌凯浠ｃ€乣continue`銆佷互鍙婃暟缁勫厓绱犳洿鏂板彲瑙佹€у洖褰掕鐩栥€?- 鍦ㄥ綋鍓嶆墽琛屾湡 Criterion 鍙ｅ緞涓嬶紝鏈€鏂板畬鏁撮噸璺戣褰?`for_of_array 20k` 涓?`1.80鈥?.96 ms`銆?- 2026-03-16锛氭坊鍔犱簡 `for_in_object` benchmark 瑕嗙洊锛屽苟閫氳繃灏嗘€ュ垏鐨勫畬鏁?key 鍏嬮殕鏇挎崲涓哄璞?鏁扮粍蹇収涓婄殑鍩轰簬绱㈠紩鐨勬噿 key 鐢熸垚锛屽畬鎴愪簡绗竴涓凯浠ｅ櫒璁剧疆浼樺寲杞銆?- 娣诲姞浜嗗洖褰掕鐩栵紝纭瀵硅薄涓婄殑 `for-in` 鍦ㄨ凯浠ｆ湡闂翠粛鐒堕€氳繃闈欐€佸睘鎬ц鍙栬瀵熷埌鏇存柊鐨勫€笺€?- Benchmark 鍩虹嚎宸茶褰曪細`for_in_object 20x2000` 鍦?Criterion 涓负 `3.74鈥?.80 ms`銆?
## 鎺ㄨ崘鎵ц椤哄簭

1. Benchmark 鍩虹嚎閲嶉獙璇佷笌鏂囨。鍚屾
2. 鍩轰簬褰撳墠 head 鐨勮皟鐢ㄨ矾寰勫洖褰掑璁?3. Native/builtin 鍙傛暟鏁寸悊鐨勬敹灏惧伐浣?4. Dense array 蹇€熻矾寰?5. 鍩轰簬褰撳墠閲嶈窇鏁版嵁缁х画鍋氬璞?灞炴€ц闂紭鍖?6. 鍐呭瓨娴嬮噺杞
7. GC 鍩轰簬 root 鐨勬爣璁板伐浣?8. Opcode dispatch 鏀剁揣
9. 娆¤寰紭鍖?
## 瀹屾垚鏍囧噯

褰撴弧瓒充互涓嬫潯浠舵椂锛屾浼樺寲浠诲姟娓呭崟瑙嗕负鍩烘湰瀹屾垚锛?
- benchmark 鍩虹嚎鍙俊涓斿彲澶嶇幇
- `fib`銆乣loop`銆乣array` 鍜?`sieve` 鍚勮嚜鑷冲皯鏈変竴涓粡杩囬獙璇佺殑鐑偣鏀硅繘
- GC 涓嶅啀渚濊禆淇濆畧鐨?`mark_all`
- 鍐呭瓨鍑忓皯宸ヤ綔鍩轰簬娴嬮噺鐨勪富瀵肩被鍒紝鑰岄潪鐚滄祴
- 鏂囨。浠呭弽鏄犳湁鏁堢殑 benchmark 缁撹
## 9.1 涓荤嚎琛ュ厖璇存槑

- 2026-03-20锛歚deep_property` / 鏅€氬璞″睘鎬ч摼宸茬粡閲嶆柊鎴愪负褰撳墠涓荤嚎銆?- 宸插畬鎴愶細
  - 鍦?`GetField` / `GetField2` 涓婅ˉ浜嗕竴灞傛洿绐勭殑鏅€氬璞＄洿杈惧揩璺緞
  - 鏅€氬璞￠摼璇诲彇鐜板湪浼氱洿鎺ヨ蛋 `object_get_property()`锛屼笉鍐嶅厛绌胯繃瀹屾暣鐨?`get_field_value()` 绫诲瀷鍒嗗彂閾?- 褰撳墠鎵ц鏈熼噸璺戠粨鏋滐細
  - `deep_property 200k`锛歚18.706鈥?0.128 ms`
  - `method_chain 5k`锛歚1.106鈥?.214 ms`
  - `runtime_string_pressure 4k`锛歚1.118鈥?.205 ms`
- 褰撳墠瑙ｈ锛?  - 杩欐槸褰撳墠鏈€骞插噣鐨勪竴杞?deep-property 璺熻繘锛屽洜涓哄畠鐩存帴鍛戒腑浜?`root.a.b.c.d` 杩欑杩炵画鏅€氬璞￠摼璇诲彇
  - dense-array 璇讳晶閭ｆ潯寰紭鍖栫嚎褰撳墠宸茬粡杩涘叆鈥滈珮鍥炲綊椋庨櫓銆佷綆淇″彿鈥濈殑鍖哄煙锛屾殏鏃舵敹浣?  - 鍚庣画濡傛灉娌℃湁鏂扮殑 profiling 璇佹嵁锛屼笉鍐嶉噸鍚?dense-array 璇讳晶鐨勫井浼樺寲灏忓垁

## 9.1 涓荤嚎琛ュ厖璇存槑锛堢画锛?
- 2026-03-20锛氬張涓烘渶鐑殑 4 娈甸潤鎬佹櫘閫氬璞″睘鎬ч摼锛坄root.a.b.c.d`锛夎ˉ浜嗕竴鏉″瓧鑺傜爜绾х殑 `GetFieldChain4` 閲嶅啓銆?- 宸插畬鎴愶細
  - deep property chain 鐜板湪鍙互浠?4 娆¤繛缁?`GetField` 鏀舵垚 1 鏉′笓闂ㄧ殑閾惧紡灞炴€ц鍙?opcode
  - 宸茶ˉ compiler 鍥炲綊锛岄攣瀹?`deep_property` 褰㈢姸浼氬彂鍑?`GetFieldChain4`
- 褰撳墠鎵ц鏈熼噸璺戠粨鏋滐細
  - `deep_property 200k`锛歚15.846鈥?6.421 ms`
  - `method_chain 5k`锛歚1.082鈥?.180 ms`
  - `runtime_string_pressure 4k`锛歚1.128鈥?.185 ms`
- 褰撳墠瑙ｈ锛?  - 杩欐槸褰撳墠绗竴鏉℃妸 `deep_property` 鏄庣‘鎷夊洖 `~16 ms` 閲忕骇鐨勮窡杩涙敼鍔?  - 瀹冩瘮缁х画閲嶅惎 dense-array 璇讳晶寰紭鍖栨洿鍊煎緱锛屽洜涓哄畠鍛戒腑鐨勬槸涓€涓潪甯告槑纭€佸彲閲嶅鐨勬櫘閫氬璞￠摼寮忚闂舰鐘?  - 杩欎篃鎰忓懗鐫€褰撳墠涓荤嚎宸茬粡姝ｅ紡鍒囧埌 `deep_property`锛岃€?dense-array 璇讳晶閭ｆ潯寰紭鍖栫嚎缁х画淇濇寔鍐荤粨锛岄潪蹇呰涓嶅啀閲嶅惎
- 2026-03-20锛氱户缁妸 `deep_property` 鍛ㄥ洿鍓╀笅鐨勭儹寰幆灏惧反鏀剁揣浜嗕竴灞傦紝鎵╁睍浜嗙幇鏈夌殑鏈湴鍙橀噺鏇存柊 peephole锛屼娇瀹冧篃鑳界洿鎺ュ悆鎺夋渶鐑殑鏈崟鑾?`PutLoc1` 绱姞鍣ㄥ舰鐘躲€?- 褰撳墠鎵ц鏈熼噸璺戠粨鏋滐細
  - `deep_property 200k`锛歚10.877鈥?1.041 ms`
  - `method_chain 5k`锛歚0.758鈥?.823 ms`
  - `runtime_string_pressure 4k`锛歚0.795鈥?.809 ms`
- 褰撳墠瑙ｈ锛?  - 褰撳墠 `deep_property` 杩欐潯绾垮凡缁忎笉鍙槸鈥滃睘鎬ц鍙栨湰韬洿蹇€濓紝鑰屾槸杩炲懆鍥寸殑绱姞楠ㄦ灦涔熶竴璧疯鍘嬩簡涓嬫潵
  - 杩欒鏄庢垜浠綋鍓嶉€夋嫨鐨勪富绾挎槸瀵圭殑锛岃€屼笖鐜板湪宸茬粡杩涘叆涓€涓柊鐨勬€ц兘妗ｄ綅
  - 鍏ㄩ噺寮曟搸娴嬭瘯銆乣clippy -D warnings` 鍜?`cargo test -p led-runtime` 涔熼兘宸茬粡閲嶆柊楠岃瘉閫氳繃
## 9.1 涓荤嚎琛ュ厖璇存槑锛堜笁锛?
- 2026-03-20锛氱户缁妸 `deep_property` 鍛ㄥ洿鍓╀笅鐨勭疮鍔犲熬宸存敹绱т簡涓€灞傦紝鎵╁睍浜嗙幇鏈夌殑鏈湴鍙橀噺鏇存柊 peephole锛屼娇瀹冧篃鑳界洿鎺ュ悆鎺夋渶鐑殑鏈崟鑾?`PutLoc1` 褰㈢姸銆?- 褰撳墠鎵ц鏈熼噸璺戠粨鏋滐細
  - `deep_property 200k`锛歚10.877鈥?1.041 ms`
  - `method_chain 5k`锛歚0.758鈥?.823 ms`
  - `runtime_string_pressure 4k`锛歚0.795鈥?.809 ms`
- 褰撳墠瑙ｈ锛?  - 鐜板湪鐨?`deep_property` 宸茬粡涓嶅彧鏄€滃睘鎬ч摼鏈韩鏇村揩鈥濓紝鑰屾槸杩炲懆鍥寸疮鍔犻鏋朵篃涓€璧峰帇浜嗕笅鏉?  - dense-array 璇讳晶寰紭鍖栫户缁繚鎸佸喕缁擄紝闄ら潪鍚庣画 profiling 缁欏嚭鍏ㄦ柊鐨勭儹鐐瑰舰鐘?  - 瀛楃涓查偅鏉℃棫浼樺寲绾夸篃缁х画瑙嗕负鍐荤粨鍖猴紝鍚庣画鑻ュ啀鍔紝搴斾綔涓烘柊鐨勫瓧绗︿覆琛ㄧず椤圭洰锛岃€屼笉鏄噸鍚棫寰紭鍖?## 2026-03-20 UTF-8 琛ュ厖锛歫son parse 涓荤嚎

- 杩欒疆閲嶆柊鎺掑簭涔嬪悗锛宍json parse` 琚‘璁や负褰撳墠鏂扮殑姝ｅ紡涓荤嚎銆?- 鏂板浜嗕袱鏉¤瘖鏂?benchmark锛?  - `json parse only 1k`
  - `json parse property read 1k`
- 褰撳墠鍒ゆ柇锛?  - `json parse only` 鍜?`json parse property read` 鍩烘湰钀藉湪鍚屼竴閲忕骇锛岃鏄庡綋鍓嶄富瑕佹垚鏈洿鍍忔槸 `JSON.parse(...)` 璋冪敤鍏ュ彛涓庤В鏋?鍒嗛厤鏈韩锛岃€屼笉鏄悗缁殑 `obj.value` 璇诲彇銆?- 褰撳墠宸蹭繚鐣欑殑绋冲畾浼樺寲锛?  - 缂撳瓨浜?`native_json_parse_idx`
  - 鍦?`CallMethod` 涓粰绮剧‘鐨?`JSON.parse(arg)` builtin 褰㈢姸鍔犱簡绐?fast path锛?    - `this === JSON`
    - native 鐩爣灏辨槸缂撳瓨鐨?`JSON.parse`
    - `argc == 1`
- 褰撳墠鎵ц鏈熼噸璺戠粨鏋滐細
  - `json parse 1k`锛歚1.3660鈥?.4460 ms`
  - `json parse only 1k`锛歚1.3296鈥?.4112 ms`
  - `json parse property read 1k`锛歚1.3313鈥?.4195 ms`
- dump 鐑偣琛ュ厖锛?  - `json_parse_only` 鐨?runtime string 鎬绘暟浠?`2001` 闄嶅埌 `1001`
  - `json_parse_property_read` 鐨?runtime string 鎬绘暟涔熶粠 `2001` 闄嶅埌 `1001`
- 琛ュ厖鐨勫洖褰掕鐩栵細
  - Unicode JSON 瀛楃涓插€?  - 璐熸暟 JSON
  - 灏忔暟 JSON
- 褰撳墠瑙ｈ锛?  - 杩欐槸褰撳墠 head 涓婄涓€鏉″彲浠ョǔ瀹氱暀涓嬫潵鐨?`json parse` 涓荤嚎鏀剁泭锛?  - 瀹冨懡涓殑鏄?`JSON.parse(...)` 鍏ュ彛鎴愭湰锛屾病鏈夊幓閲嶅惎宸茬粡鍐荤粨鐨勬棫璋冪敤涓荤嚎鎴栨棫瀛楃涓蹭富绾匡紱
  - 浣?`json parse` 绂绘洿鏃╅偅缁?`~0.73鈥?.75 ms` 鐨勬渶濂藉尯闂磋繕鏈夋槑鏄捐窛绂伙紝鎵€浠ュ畠鐜板湪浠嶇劧搴旇淇濇寔涓哄綋鍓嶆寮忎富绾裤€?
- 2026-03-20锛氱户缁妸 `json parse` 鐨勫垎閰嶈矾寰勬敹绱т簡涓€灞傦紝鍙仛灏忓閲忛鍒嗛厤锛屼笉鏀硅В鏋愯涔夛細
  - `parse_string()` 鏀规垚 `String::with_capacity(16)`
  - `parse_array()` 鏀规垚 `Vec::with_capacity(4)`
  - `parse_object()` 鏀规垚 `Vec::with_capacity(4)`
  - object key 鐨勪复鏃跺瓧绗︿覆涔熸敼鎴?`String::with_capacity(16)`
- 閲嶆柊璺戜簡瀹氬悜 JSON.parse 鍥炲綊锛?  - 鏁板瓧
  - 璐熸暟
  - 灏忔暟
  - Unicode 瀛楃涓插€?- 褰撳墠鎵ц鏈熼噸璺戠粨鏋滐細
  - `json parse 1k`锛歚1.2098鈥?.3497 ms`
  - `json parse only 1k`锛歚1.3079鈥?.7092 ms`
  - `json parse property read 1k`锛歚1.1194鈥?.1928 ms`
- 褰撳墠瑙ｈ锛?  - 杩欐槸涓€鏉′綆椋庨櫓鐨勫垎閰嶈矾寰勪紭鍖栵紝涓嶆槸鍐嶆鍘绘姞 parser 鎺у埗娴佺粏鑺傦紱
  - 涓ゆ潯 JSON 璇婃柇鍩虹嚎閮芥槑鏄惧線涓嬭蛋锛岃鏄庡綋鍓?`json parse` 涓荤嚎鍦ㄥ垎閰?鐗╁寲璺緞涓婁粛鐒惰繕鏈夌湡瀹炵┖闂达紱
  - `json parse 1k` 杩欎竴鏉″湪杩欐閲嶈窇閲屼粛鐒跺櫔澹板亸澶э紝浣嗕腑蹇冨尯闂翠篃缁х画寰€涓嬬Щ鍔紝鎵€浠ヨ繖鍒€鍊煎緱淇濈暀銆?
- 2026-03-21锛氱户缁妸 `json parse` 涓荤嚎寰€鍓嶆帹鎴愪簡涓€鏉℃洿缁撴瀯鍖栫殑浼樺寲锛?  - 鍦ㄨВ閲婂櫒閲屽姞鍏ヤ簡涓€涓緢绐勭殑鈥滅紪璇戞湡甯搁噺 JSON 妯℃澘缂撳瓨鈥?  - cache key 鍙娇鐢?`(current_string_constants 鎸囬拡, string index)`
  - 鍛戒腑鍚庝笉鍐嶉噸鏂拌В鏋愬悓涓€浠界紪璇戞湡 JSON 鏂囨湰锛岃€屾槸浠庣紦瀛樻ā鏉块噸鏂?materialize 鍑烘柊鐨勫璞?鏁扮粍/瀛楃涓?- 琛ュ厖浜嗗洖褰掕鐩栵細
  - 杩炵画涓ゆ `JSON.parse` 鍛戒腑缂撳瓨鍚庯紝杩斿洖鐨勫璞″拰鏁扮粍浠嶇劧鏄叏鏂扮殑锛屼笉浼氫簰鐩稿埆鍚嶆薄鏌?- 鏂板浜嗕竴涓洿杞婚噺鐨勯獙璇佸伐鍏凤細
  - `json_parse_probe`
  - 杩欐牱鍚庣画楠岃瘉杩欐潯涓荤嚎鏃讹紝涓嶅繀姣忔閮芥媺璧锋暣濂?Criterion
- 褰撳墠 `json_parse_probe` 蹇収锛?  - `json_parse avg_ms=1.522`
  - `json_parse_only avg_ms=1.544`
  - `json_parse_property_read avg_ms=1.468`
- 褰撳墠瑙ｈ锛?  - 杩欐槸 `json parse` 涓荤嚎绗竴鏉℃洿鈥滅粨鏋勫寲鈥濈殑鏀剁泭锛屼笉鍐嶅彧鏄皟鐢ㄥ叆鍙ｆ垨灏忓閲忛鍒嗛厤鐨勫皬淇ˉ锛?  - 瀹冪湡姝ｇ瀯鍑嗕簡 benchmark 閲岀殑閲嶅宸ヤ綔褰㈢姸锛氭瘡杞兘閲嶆柊瑙ｆ瀽鍚屼竴浠界紪璇戞湡 JSON 瀛楃涓诧紱
  - 璇箟淇濇寔姝ｇ‘锛岃€屼笖鐜板湪杩欐潯涓荤嚎宸茬粡杩涘叆鈥滄寔缁嬁鍒扮ǔ瀹氭敹鐩娾€濈殑闃舵锛屼笉鍐嶅彧鏄瘖鏂樁娈点€?- 2026-03-21 鏇存锛?  - 鍚庣画澶氳疆 probe 娌℃湁绋冲畾璇佹槑鈥滅紪璇戞湡甯搁噺 JSON 妯℃澘缂撳瓨鈥濆湪褰撳墠 head 涓婃槸鍑€鏀剁泭锛屾墍浠ヨ繖鏉″疄楠岀嚎鍚庢潵宸茬粡鎾ゅ洖锛屼笉搴旇涓哄凡钀藉湴鎴愭灉銆?  - 褰撳墠绋冲畾淇濈暀鐨?`json parse` 浼樺寲搴旂悊瑙ｄ负锛?    - `native_json_parse_idx` 缂撳瓨
    - 绐勭殑 `JSON.parse(arg)` `CallMethod` fast path
    - built-in / compile-time string 杈撳叆鏃堕伩鍏嶆渶寮€濮嬮偅娆℃暣涓插鍒?    - `parse_string / parse_array / parse_object` 鐨勫皬瀹归噺棰勫垎閰?    - `json_parse_probe` 宸ュ叿淇濈暀锛岀敤浜庤交閲忛獙璇佽繖鏉′富绾?  - `json parse` 浠嶇劧鏄綋鍓嶆寮忎富绾匡紝浣嗗凡鎾ゅ洖鐨勬ā鏉跨紦瀛樺疄楠屼笉搴旂户缁綋浣滃凡瀹屾垚宸ヤ綔寮曠敤銆?- 2026-03-21 闃舵鏀跺彛鍒ゆ柇锛?  - 鍦ㄧǔ瀹氱殑璋冪敤鍏ュ彛浼樺寲鍜屽皬鍒嗛厤浼樺寲涔嬪悗锛屽悗缁杞?`json parse` 璇曢獙锛坧arser 寰揩璺緞銆侀澶栬皟鐢ㄥ舰鐘?opcode銆佹ā鏉跨紦瀛樺彉浣撱€佸瓧绗︿覆鍙跺瓙缂撳瓨鍙樹綋锛夊凡缁忚繘鍏ラ珮鍥為€€銆侀珮鍣０鍖恒€?  - 褰撳墠宸ヤ綔鏍戞瘡娆￠兘宸叉媺鍥炴渶鍚庝竴涓共鍑€绋冲畾鐗堟湰锛屾病鏈夋妸杩欎簺澶辫触鍒嗘敮鐣欏湪浠ｇ爜閲屻€?  - 鍥犳锛屽綋鍓嶈繖涓€杞?`json parse` 寰紭鍖栧彲浠ヨ涓衡€滈樁娈垫€у熀鏈畬鎴愨€濓細
    - 涓嶆槸姘歌繙涓嶅仛
    - 鑰屾槸褰撳墠涓嶅啀缁х画纭姞
  - 鎺ㄨ崘鐨勪笅涓€姝ユ槸锛?    - 鏆傚仠缁х画鍋?`json parse` 寰紭鍖?    - 鍏堥噸鏂版牎鍑?benchmark 鍩虹嚎
    - 鍐嶄粠褰撳墠 head 鐨勫共鍑€蹇収閲嶆柊鎺掑簭涓嬩竴鏉′富绾?- 2026-03-21锛氬張鍦ㄥ綋鍓嶇ǔ瀹氬伐浣滄爲涓婇噸璺戜簡涓€杞富 benchmark锛岀粨鏋滄樉绀鸿繖宸茬粡涓嶆槸鈥滅户缁寫涓嬩竴涓井鐑偣鈥濈殑闂锛岃€屾槸褰撳墠 head 鍜屼箣鍓嶈褰曠殑鍩虹嚎鏁翠綋婕傜Щ鏄庢樉锛?  - `array push 10k`锛歚766.00鈥?46.17 碌s`
  - `string concat 1k`锛歚164.20鈥?05.55 碌s`
  - `json parse 1k`锛歚1.8986鈥?.3272 ms`
  - `sieve 10k`锛歚2.3860鈥?.8523 ms`
  - `method_chain 5k`锛歚1.4008鈥?.7708 ms`
  - `runtime_string_pressure 4k`锛歚1.4943鈥?.8702 ms`
  - `for_of_array 20k`锛歚2.1365鈥?.5288 ms`
  - `deep_property 200k`锛歚19.605鈥?3.419 ms`
- 褰撳墠瑙ｈ锛?  - 杩欒鏄庡綋鍓嶇湡姝ｅ簲璇ラ噸鏂版墦寮€鐨勬槸 benchmark 鍩虹嚎姝ｇ‘鎬э紝鑰屼笉鏄珛鍒诲啀閫変竴涓柊鐨勫井浼樺寲鐩爣锛?  - 鎺ㄨ崘鐨勪笅涓€鏉℃寮忎换鍔″簲鍥炲埌锛?    - `9.1.1 Benchmark baseline 閲嶆牎鍑嗕笌鏂囨。鍚屾`
- 2026-03-21锛氬姞鍏ヤ簡涓€涓粨鏋勬€х殑 `for-in` key 澶嶇敤缂撳瓨锛屼娇閲嶅瀵瑰悓涓€瀵硅薄鍋?`for-in` 鏃朵笉鍐嶈€楀敖 runtime string table銆?- 琛ュ厖浜嗗洖褰掕鐩栵紝纭锛?  - 涔嬪墠閭ｄ釜浼氳€楀敖 runtime string table 鐨勫舰鐘剁幇鍦ㄨ兘澶熸甯歌窇瀹岋紱
  - 閲嶅澶氳疆 `for-in` 鍚屼竴瀵硅薄浠嶇劧寰楀埌姝ｇ‘缁撴灉銆?- 褰撳墠瑙ｈ锛?  - 杩欐潯鏀瑰姩鍊煎緱淇濈暀锛屼絾瀹冨綋鍓嶆洿搴旇琚涓衡€滄纭€?/ 缁撴瀯椴佹鎬т慨澶嶁€濓紝鑰屼笉鏄凡缁忓叧鍗曠殑鎬ц兘鏀剁泭锛?  - 鍥犱负褰撳墠淇″彿鏄贩鍚堢殑锛?    - 鏈湴杩涚▼绾?Rust vs C 瀵规瘮閲岋紝`for_in_object` 鐜板湪鑳介『鍒╄窇瀹岋紝鑰屼笖鏈€鏂颁竴杞粛鐒舵槸 Rust 鐣ュ揩锛坄0.897x`锛夛紱
    - 浣?Criterion 閲岀殑 `for_in_object 20x2000` 鐩墠浠嶇劧鏄庢樉鎱簬鏇存棭璁板綍鐨勯偅缁?`3.74鈥?.80 ms` 鍘嗗彶蹇収锛屾渶鏂颁竴杞湪 `10.898鈥?2.624 ms`銆?  - 鎵€浠ヨ繖鏉＄嚎褰撳墠搴旇涓猴細
    - 姝ｇ‘鎬т慨澶嶅凡钀藉湴
    - 鎬ц兘瑙ｈ浠嶅緟 baseline cleanup 瀹屾垚鍚庡啀鏀跺彛
- 2026-03-21 闃舵鏀跺彛琛ュ厖锛?  - `switch_case` 杩欐潯娆＄骇缁撴瀯绾匡紝鍦?`SwitchCaseI8` 钀藉湴涔嬪悗锛屽彲浠ヨ涓哄綋鍓嶉樁娈靛畬鎴愶紱
  - `for_in_object` 杩欐潯娆＄骇缁撴瀯绾匡紝鍙互瑙嗕负褰撳墠闃舵瀹屾垚浜庘€滅粨鏋?姝ｇ‘鎬т慨澶嶅凡钀藉湴鈥濊繖涓眰闈紝鎬ц兘鍙ｅ緞缁х画鎸傚埌 baseline cleanup 閲岀粺涓€瑙ｉ噴锛?  - `try_catch` 鏈€杩戜竴杞獎瀹為獙娌℃湁鎷垮埌骞插噣鍑€鏀剁泭锛屾墍浠ュ綋鍓嶄笉鍐嶇户缁噸寮€瀹冦€?## 2026-03-21 UTF-8 琛ュ厖锛氬疄鐢ㄦ敹鍙ｈ鍒?
- 杩欎釜浠撳簱閲岀殑浼樺寲鐩爣涓嶆槸鈥滄棤闄愰€艰繎鏋侀檺鈥濄€?- 涓€鏉＄儹鐐逛富绾垮湪褰撳墠闃舵鍙互瑙嗕负瀹屾垚锛屽綋瀹冩弧瓒筹細
  - 宸茬粡鎷垮埌涓€鍒颁袱鏉＄湡瀹炪€佸彲楠岃瘉鐨勭ǔ瀹氭敹鐩婏紱
  - 鍚庣画灏濊瘯涓昏钀藉叆楂樺櫔澹般€佷綆淇″彿鍖猴紱
  - 杩炵画 follow-up patch 宸茬粡寰堥毦鎷垮埌骞插噣鐨勫噣鏀剁泭锛?  - 缁х画鍋氫笅鍘荤殑澶嶆潅搴﹀拰鍥炲綊椋庨櫓锛屽凡缁忚秴杩囧疄闄呮敹鐩娿€?- 褰撳嚭鐜拌繖绉嶆儏鍐碉紝姝ｇ‘鍔ㄤ綔鏄細
  - 鍐荤粨璇ュ尯鍩燂紱
  - 鎶婄ǔ瀹氭敹鐩婅杩涙枃妗ｏ紱
  - 鐒跺悗鍒囧幓涓嬩竴鏉?ROI 鏇撮珮鐨勪富绾匡紝
  - 鑰屼笉鏄户缁负浜嗘渶鍚庡嚑涓櫨鍒嗙偣鍙嶅閲嶅惎鍚屼竴鍧楀尯鍩熴€?## 2026-03-21 UTF-8 琛ュ厖锛歴witch_case 缁撴瀯鏀剁泭

- 涓烘暣鏁?case 鐨?`switch` 鏂板浜?`SwitchCaseI8`銆?- 瀹冩妸鏈€鐑殑
  `Dup + PushConst + StrictEq + IfTrue`
  case-chain 褰㈢姸鏀舵垚涓€鏉♀€滄瘮杈冨父閲忓苟璺宠浆銆佷絾淇濈暀 switch 鍊尖€濈殑 opcode銆?- 宸查噸鏂拌窇杩?switch 璇箟鍥炲綊锛岃鐩栵細
  - 鍩烘湰鏁存暟 switch
  - fallthrough
  - 瀛楃涓?case
  - loop 鍐?break
- 褰撳墠鎵ц鏈熼噸璺戠粨鏋滐細
  - `switch 1k`锛歚223.70鈥?76.85 碌s`
- 鏈湴杩涚▼绾?Rust vs C 瀵规瘮锛?  - `switch_case`: `Rust=0.0723s`, `C=0.0434s`, `1.666x`
  - 褰撳墠浠嶇劧鏄?C 鏇村揩
- 褰撳墠瑙ｈ锛?  - 杩欐槸涓€鏉＄湡瀹炵殑缁撴瀯鏀剁泭锛岃鏄庡綋鍓?head 涓?`switch_case` 鐨勫瓧鑺傜爜褰㈢姸鏈韩宸茬粡鏄庢樉鏇寸煭銆佹洿渚垮疁锛?  - 杩欏垁鍊煎緱淇濈暀锛?  - 浣嗗畠浠嶇劧搴旇鏀惧湪鈥渂aseline 浠嶅湪閲嶆牎鍑嗏€濈殑涓婁笅鏂囬噷瑙ｉ噴锛岃€屼笉鏄褰撲綔鏁翠釜鎺у埗娴佸熀绾垮凡缁忔仮澶嶅仴搴风殑璇佹槑銆?
## 2026-03-21 UTF-8 琛ュ厖锛氫笅涓€鏉℃寮忎富绾?
- 鍦ㄥ綋鍓嶈繖杞?cleanup 涔嬪悗锛宍json parse`銆乣switch_case`銆乣for_in_object` 閮藉簲鍏堝喕缁擄細
  - `json parse`锛氬綋鍓嶉樁娈靛凡鎷垮埌绋冲畾鏀剁泭锛屽悗缁瘯楠屽紑濮嬭繘鍏ラ珮鍣０鍖猴紱
  - `switch_case`锛歚SwitchCaseI8` 宸茬粡缁欏嚭鐪熷疄缁撴瀯鏀剁泭锛?  - `for_in_object`锛氱粨鏋?姝ｇ‘鎬т慨澶嶅凡钀藉湴锛屾€ц兘瑙ｉ噴缁х画鎸傚埌 baseline cleanup銆?- 褰撳墠涓嶅缓璁洜涓?`sieve` 鍙堝彉鎱紝灏辩洿鎺ラ噸鍚?dense-array 璇讳晶寰紭鍖栥€?  - 閭ｆ潯绾夸笂涓€杞凡缁忔槑纭繘鍏ラ珮鍥為€€銆侀珮鍣０鍖猴紱
  - 闈炲嚭鐜板叏鏂扮殑鐑偣褰㈢姸锛屽惁鍒欎笉璇ラ┈涓婇噸寮€銆?- 鎸夊綋鍓?broad rerun 鍜屾渶鏂?Rust vs C 瀵规瘮锛屾洿鍚堢悊鐨勪笅涓€鏉℃寮忎富绾挎槸锛?  - `loop` / `sieve` 鍏变韩鐨勬瘮杈冧笌寰幆楠ㄦ灦
  - 涔熷氨鏄細
    - `GetLoc*`
    - `Lt` / `Lte`
    - `IfFalse`
    - `Goto`
- 鍘熷洜鏄細
  - `loop` 鍜?`sieve` 鍦ㄦ渶鏂版湰鍦?Rust vs C 瀵规瘮閲屼粛鐒堕兘钀藉悗浜?C锛?  - 瀹冧滑鏄?headline benchmark锛屼笉鏄彧褰卞搷涓€鏉℃绾ц瘖鏂剼鏈殑灏忕偣锛?  - 鑰屼笖杩欐潯绾垮彲浠ョ户缁仛锛屼絾涓嶉渶瑕侀噸鏂版墦寮€宸茬粡鍐荤粨鐨?dense-array 璇讳晶寰紭鍖栥€?- 鍥犳锛屽湪 `9.1.1` baseline cleanup 鍩烘湰鏀朵綇涔嬪悗锛屾帹鑽愮殑涓嬩竴鏉℃寮忎富绾挎槸锛?  - `loop/sieve` 鐨?comparison-and-branch skeleton tightening

## 2026-03-21 UTF-8 琛ュ厖锛歠ib / switch_case 瀹氬悜澶嶆煡

- 鍦ㄥ綋鍓嶇ǔ瀹氬伐浣滄爲涓婂張琛ヨ窇浜嗕竴杞洿绐勭殑瀹氬悜 benchmark锛?  - `fib_iter 1k`锛歚5.3292鈥?.2708 ms`
  - `switch 1k`锛歚281.10鈥?45.33 碌s`
- 褰撳墠瑙ｈ锛?  - `switch_case` 杩欐潯绾夸粛鐒跺簲瑙嗕负鈥滅粨鏋勬敹鐩婂凡钀藉湴鈥濓細
    - `SwitchCaseI8` 浠嶇劧鍦ㄥ彂鐮侊紱
    - `switch` 鐨勫瓧鑺傜爜褰㈢姸鏈韩娌℃湁璧颁涪锛?    - 浣嗗畠鐜板湪宸茬粡涓嶆槸鏈€鎬ョ殑鍥炲綊椤广€?  - 鐪熸鏇翠弗閲嶇殑褰撳墠淇″彿鏄?`fib_iter`锛?    - 瀹冪浉姣旀洿鏃╃殑 `2.330鈥?.379 ms` 鍘嗗彶鍖洪棿鍋忕寰楁槑鏄炬洿澶氾紱
    - 鎵€浠ュ綋鍓嶄紭鍏堢骇搴旇鎶?`fib` / 璋冪敤-閫掑綊寮€閿€閲嶆柊鎶埌 `switch_case` 涔嬪墠銆?- 鍥犳锛屽綋鍓嶆洿瀹炵敤鐨勪紭鍏堢骇搴旂悊瑙ｄ负锛?  - 鍏堢湅 `fib` / call-recursion overhead
  - 鍐嶇湅 `loop/sieve` 鐨?comparison-and-branch skeleton
  - `switch_case` 缁х画淇濇寔鍐荤粨锛岄櫎闈炲嚭鐜版柊鐨?switch 涓撳睘鐑偣褰㈢姸

## 2026-03-21 UTF-8 琛ュ厖锛歠ib_iter / switch_case 璇婃柇宸ュ叿鎺ョ嚎

- 鐜板湪宸茬粡鎶?`fib_iter` 鍜?`switch_case` 閮芥帴杩涗簡鏈湴璇婃柇宸ュ叿閾撅細
  - `benches/workloads/fib_iter.js`
  - `src/bin/dump_bytecode.rs`
  - `src/bin/profile_hotspots.rs`
- 褰撳墠绋冲畾鏍戜笂鐨?dump-mode 鐑偣缁撹鏄細
  - `fib_iter`
    - 椤跺眰澶栧惊鐜牳蹇冧粛鐒舵槸 `Lt + IfFalse + Call + Add + IncLoc`
    - 鍐呭眰杩唬 `fib` 鏈綋褰撳墠涓昏鐢卞眬閮ㄦЫ浣嶆祦閲忓拰灏忓惊鐜鏋朵富瀵硷紝灏ゅ叾鏄細
      - `GetLoc3`
      - `Drop`
      - `Dup`
      - `GetLoc0`
      - `Lte`
      - `GetLoc2`
      - `PutLoc2`
      - `Goto`
      - `Add`
      - `GetLoc4`
      - `PutLoc3`
      - `GetLoc8`
      - `PutLoc8`
      - `IncLoc4Drop`
  - `switch_case`
    - `SwitchCaseI8` 鏄庣‘杩樺湪宸ヤ綔锛堟渶鏂?dump 蹇収閲屾墽琛屼簡 `108000` 娆★級
    - 杩欒鏄?`switch_case` 褰撳墠鍓╀綑鎴愭湰鏇村儚鏄?switch 澶栧洿鐨?loop/add/update 楠ㄦ灦锛岃€屼笉鏄棫鐨勬暣鏁?case 姣旇緝閾炬湰韬?- 褰撳墠瑙ｈ锛?  - 杩欒繘涓€姝ュ己鍖栦簡鍓嶉潰鐨勪紭鍏堢骇璋冩暣锛?    - `fib_iter` 鎵嶆槸鐜板湪鏇存槑纭殑涓嬩竴鏉＄粨鏋勪富绾匡紱
    - `switch_case` 缁х画鍐荤粨鏇村悎鐞嗭紝鍥犱负瀹冪殑涓撻棬 opcode 鐩墠琛ㄧ幇绗﹀悎棰勬湡銆?
## 2026-03-21 UTF-8 琛ュ厖锛歠ib_iter 绗竴鏉＄ǔ瀹氭敹鐩?
- 杩欒疆 `fib_iter` 鐨勭涓€鏉＄ǔ瀹氭敹鐩婂凡缁忚惤鍦帮細
  - 鑷姩 GC trigger 璁拌处涓嶅啀鎸傚湪姣忎竴娆￠€氱敤 JS `Call` / `CallMethod` / `CallConstructor` 涓婏紱
  - 鐜板湪鏀规垚鍙湪鐪熸鐨?GC-managed allocation path 涓婅璐︼紝涔熷氨鏄細
    - closure
    - var cell
    - array
    - object
    - iterator
    - error object
    - regex object
    - typed array
    - array buffer
- 杩欐牱鏀圭殑鍘熷洜寰堢洿鎺ワ細
  - 涔嬪墠閭ｇ鈥滄瘡娆¤皟鐢ㄩ兘璁颁竴娆♀€濈殑妯″瀷锛屼細璁?`fib_iter` 杩欑楂樿皟鐢ㄣ€佷綆鍒嗛厤 workload 鐧界櫧鏀粯 GC bookkeeping 鎴愭湰锛?  - 杩欓潪甯稿儚鏄?`fib_iter` 鍓嶉潰澶у洖閫€鐨勬牴鍥犱箣涓€銆?- 褰撳墠楠岃瘉缁撴灉锛?  - 瀹氬悜 Criterion锛?    - `fib_iter 1k`锛歚3.5469鈥?.1842 ms`
  - 鍚庣画澶嶈窇锛?    - `fib_iter 1k`锛歚3.5909鈥?.2369 ms`
  - GC 鑷姩瑙﹀彂鍥炲綊浠嶇劧閫氳繃锛?    - `test_gc_auto_triggers_during_js_function_workload`
  - 鍏ㄩ噺 `cargo test -p mquickjs-rs` 涓?`clippy -D warnings` 閮介€氳繃銆?- 褰撳墠瑙ｈ锛?  - 杩欐槸褰撳墠閲嶆柊鎺掑簭涔嬪悗锛宍fib_iter` 杩欐潯绾跨殑绗竴鏉＄ǔ瀹氭敹鐩婏紱
  - 瀹冧笉鍙槸 benchmark 鏇村揩浜嗭紝涔熻褰撳墠 GC trigger 妯″瀷鏇寸鍚堢湡瀹?allocation pressure銆?
## 2026-03-21 UTF-8 琛ュ厖锛歠ib_iter 闃舵鏀跺彛

- 鍦ㄥ彧淇濈暀绗竴鏉＄ǔ瀹氭敹鐩婄殑褰撳墠宸ヤ綔鏍戜笂锛屽張琛ヨ窇浜嗕竴杞畾鍚?benchmark锛?  - `fib_iter 1k`锛歚3.0507鈥?.6993 ms`
  - `loop 10k`锛歚690.21鈥?46.31 碌s`
  - `sieve 10k`锛歚2.9538鈥?.5064 ms`
- 褰撳墠瑙ｈ锛?  - 涔嬪墠淇濈暀涓嬫潵鐨?GC trigger 璁拌处璺緞璋冩暣锛屼粛鐒舵妸 `fib_iter` 绋冲畾缁存寔鍦ㄦ瘮 `5.3292鈥?.2708 ms` 鍥為€€妗ｆ槑鏄炬洿鍋ュ悍鐨勫尯闂达紱
  - 鍚庣画閭ｆ潯鈥滄湰鍦版嫹璐濆熬宸存敹绱р€濆疄楠屽洜涓轰細鎷栨參 `loop`锛屽凡缁忓畬鏁存挙鍥烇紱
  - 杩欒鏄庡綋鏃剁殑 `fib_iter` 涓撻」寰皟宸茬粡杩涘叆鎴戜滑鑷繁瀹氫箟鐨勨€滃疄鐢ㄦ敹鍙ｅ尯鈥濓細
    - 宸茬粡鎷垮埌涓€鏉＄湡瀹炵ǔ瀹氭敹鐩婏紱
    - 鍐嶅線涓嬪仛寮€濮嬭繘鍏ラ珮鍣０銆侀珮璇激鍖恒€?- 鍥犳锛屾寜褰撳墠浠撳簱鐨?stop rule锛岃繖涓€闃舵鐨?`fib_iter` 鍙互瑙嗕负闃舵鎬у熀鏈畬鎴愶紱
  - 杩欏苟涓嶆剰鍛崇潃鍚庣画鍏变韩缁撴瀯涓荤嚎涓嶄細缁х画椤哄甫鏀瑰杽瀹冿紝鍙槸琛ㄧず涓嶅啀鍗曠嫭閲嶅紑 `fib_iter` 寰皟绾裤€?- 鎺ㄨ崘鐨勪笅涓€鏉℃寮忎富绾块噸鏂板洖鍒帮細
  - `loop/sieve` 鐨?comparison-and-branch skeleton tightening
- 2026-03-21锛氬洖鍒?`loop/sieve` 涓荤嚎涔嬪悗锛屽凡缁忔嬁鍒扮涓€鏉℃槑纭殑鍏变韩缁撴瀯鏀剁泭锛?  - 闈炲瓧绗︿覆 `Add / Mul` 璺緞鐜板湪浼氱洿鎺ユ秷璐硅鍙ョ骇鏈湴瀛樺偍锛?    - `PutLoc0..4`
    - `PutLoc8 <idx>`
  - 涔熷氨鏄儚锛?    - `c = a + b;`
    - `j = i * i;`
    杩欑璇彞锛屼笉鍐嶅厛鎶婄粨鏋滃帇鏍堝啀绔嬪埢琚?`PutLoc*` 寮规帀銆?- 瀹冩濂藉懡涓細
  - `fib_iter` 鍐呭眰鏈€鐑殑閭ｆ潯鐗╁寲褰㈢姸锛歚GetLoc2; GetLoc3; Add; PutLoc8 5`
  - `sieve` 閲岀殑鐑舰鐘讹細`GetLoc3; GetLoc3; Mul; PutLoc4`
- 琛ヤ簡鍥炲綊瑕嗙洊锛?  - `test_statement_local_add_store_updates_captured_target`
  - `test_statement_local_mul_store_updates_captured_target`
- 褰撳墠楠岃瘉缁撴灉锛?  - `fib_iter 1k`锛歚2.2286鈥?.6849 ms`
  - `loop 10k`锛歚455.86鈥?59.99 碌s`
  - `sieve 10k`锛歚1.8323鈥?.1708 ms`
  - 鍏ㄩ噺 `cargo test -p mquickjs-rs` 涓?`clippy -D warnings` 閮介€氳繃銆?- 褰撳墠瑙ｈ锛?  - 杩欐槸閲嶆柊鍥炲埌 `loop/sieve` 涓荤嚎涔嬪悗鐨勭涓€鏉″叡浜ǔ瀹氭敹鐩婏紱
  - 瀹冧篃椤哄甫鎶?`fib_iter` 鍙堟媺涓嬩簡涓€涓彴闃讹紝浣嗚繖娆″簲璁版垚鈥滃叡浜湰鍦扮畻鏈瓨鍌ㄤ紭鍖栤€濓紝鑰屼笉鏄噸鏂版墦寮€ `fib_iter` 涓撻」寰皟銆?
