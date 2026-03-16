# 寮曟搸浼樺寲浠诲姟娓呭崟

鏈枃妗ｆ槸 `mquickjs-rs` **浠呴潰鍚戝紩鎿?*鐨勪紭鍖栧緟鍔炴竻鍗曘€?
瀹冪洿鎺ユ簮鑷?`IMPLEMENTATION_PLAN.md` 涓皻鏈畬鎴愮殑绗?9 闃舵锛?- `9.1 Profile and optimize hot paths`
- `9.2 Optimize GC performance`
- `9.3 Reduce memory usage`

鏈枃妗?涓嶅寘鍚? `led-runtime` 浜у搧灞傚伐浣溿€?
鐩稿叧 benchmark 鍒嗘瀽锛?- `docs/BENCHMARK_ANALYSIS.md`
- `docs/BENCHMARK_ANALYSIS.zh.md`

## 閫傜敤鑼冨洿

鏈枃妗ｅ彧瑕嗙洊锛?- `mquickjs-rs` 鐨?parser / compiler / VM / runtime
- benchmark 鐨勬纭€т笌鎬ц兘鍒嗘瀽
- 寮曟搸鑷韩鐨?GC 涓庡唴瀛樿涓?
鏈枃妗ｄ笉瑕嗙洊锛?- `led-runtime` 涓绘満 API 浜轰綋宸ュ
- effect 鑴氭湰/浜у搧璇箟
- GUI / demo / 浜у搧灞傞泦鎴?
## 褰撳墠浼樺寲涓婚

缁撳悎褰撳墠浠ｇ爜鍜?benchmark 褰㈢姸锛屾渶鍊煎緱鍏虫敞鐨勫紩鎿庣儹鐐规槸锛?- `src/vm/interpreter.rs` 涓殑璋冪敤涓庢柟娉曞垎鍙?- `src/vm/interpreter.rs` 鍜?`src/vm/natives.rs` 涓殑 native / builtin 鍙傛暟鏁寸悊
- `src/vm/interpreter.rs` 鍜?`src/vm/property.rs` 涓殑 dense array 璁块棶
- `src/vm/interpreter.rs` 涓殑 opcode dispatch 寮€閿€
- `src/gc/collector.rs` 涓殑 GC 瀹炵幇璐ㄩ噺
- `src/vm/types.rs`銆乣src/context.rs` 鍜?`src/runtime/*` 涓殑杩愯鏃跺垎閰嶄笌瀹瑰櫒甯冨眬

## 浼樺厛绾ф€荤粨

### P0

- benchmark 鐪熷疄鍩虹嚎娓呯悊
- 璋冪敤璺緞鐑偣浼樺寲
- Native/builtin 璋冪敤鍙傛暟浼犻€掍紭鍖?- Dense array 蹇€熻矾寰?
### P1

- 鏈€鐑偣 opcode 鐨?dispatch 绠€鍖?- GC锛氫粠淇濆畧鐨?`mark_all` 琛屼负杩佺Щ鍒扮湡姝ｇ殑 root-based marking
- 杩愯鏃跺垎閰嶄笌鍐呭瓨鍗犵敤璇勪及

### P2

- Builtin/runtime 杈圭晫缁撴瀯娓呯悊
- 鏂?benchmark 楠岃瘉鍚庣殑绗簩杞井浼樺寲

## 璇︾粏浠诲姟娓呭崟

## 9.1 鍒嗘瀽骞朵紭鍖栫儹鐐硅矾寰?
### 9.1.1 Benchmark 鍩虹嚎娓呯悊

**浼樺厛绾?*: P0

**鍘熷洜**

- 鍙湁 benchmark 鏁版嵁鍙俊锛屼紭鍖栧伐浣滄墠鏈夋剰涔夈€?- 鏈湴鑴氭湰鍜?CI workflow 涔嬪墠瀛樺湪涓嶄竴鑷淬€?- 涓€浜涘巻鍙?benchmark 缁撹鍩轰簬閿欒鐨勫姣旂洰鏍囥€?
**浠诲姟**

- 淇濈暀涓€涓彲淇＄殑鏈湴 benchmark 娴佺▼鐢ㄤ簬楠岃瘉銆?- 淇濇寔 CI benchmark 琛屼负涓庢湰鍦?benchmark 琛屼负涓€鑷淬€?- 鍖哄垎锛氳繘绋嬪惎鍔ㄥ紑閿€涓庤剼鏈噣鎵ц鏃堕棿銆?- 缁存姢涓€涓粺涓€鐨勫熀绾胯〃锛岃嚦灏戣鐩栵細
  - `fib`
  - `loop`
  - `array`
  - `sieve`
  - `json`

**楠岃瘉鏂瑰紡**

- Benchmark 澶氭杩愯缁撴灉鍙鐜般€?- `docs/BENCHMARK_ANALYSIS.md` 鍐呴儴涓€鑷淬€?
**褰撳墠宸插畬鎴?*

- 2026-03-16锛氬凡瀹氫箟褰撳墠瑙勮寖鐨?benchmark 闆嗗悎銆?- 2026-03-16锛氬凡鍒嗙骞惰褰曟湰鍦?Criterion銆佹湰鍦?Rust-vs-C 瀵规瘮銆丆I Summary 涓夎€呯殑鑱岃矗銆?- 2026-03-16锛歚.github/workflows/bench.yml` 鐜板湪浼氬悓鏃惰緭鍑?Rust-vs-C 瀵规瘮琛ㄥ拰 Rust-only 鐨?Criterion 琛ㄣ€?- 2026-03-16锛歚docs/BENCHMARK_ANALYSIS.md` 宸查噸鍐欎负褰撳墠鍩虹嚎鍙傝€冩枃妗ｃ€?- 鐘舵€侊細瀵逛簬褰撳墠寮曟搸浼樺寲闃舵锛屾浠诲姟鍙涓哄凡瀹屾垚銆?
### 9.1.2 璋冪敤璺緞鐑偣浼樺寲

**浼樺厛绾?*: P0

**鐑偣鏂囦欢**

- `src/vm/interpreter.rs`
- `src/vm/stack.rs`

**鍘熷洜**

- `fib` 鍜?`loop` 寮虹儓琛ㄦ槑璋冪敤寮€閿€涓庨珮棰?dispatch 寮€閿€浠嶆槸涓昏鎴愭湰銆?- 褰撳墠 `Call` 璺緞宸叉湁鏀硅繘锛屼絾 `remove_at_offset()` 浠嶄細璋冪敤 `Vec::remove()`锛屽鑷村厓绱犺縼绉汇€?
**浠诲姟**

- 閲嶆柊璁捐璋冪敤鏍堝竷灞€锛岄伩鍏嶅湪鐑皟鐢ㄨ矾寰勪笂浣跨敤 `Vec::remove()`銆?- 鍒嗗埆閽堝 `Call`銆乣CallMethod`銆乣CallConstructor` 鍋氫笓闂ㄤ紭鍖栥€?- 鍑忓皯鏅€?JS 鍑芥暟璋冪敤涓殑涓存椂鍙傛暟閲嶆帓銆?- 閲嶆柊璇勪及璋冪敤璺緞涓殑瀛楃涓叉彁鍗囨垚鏈€?
**棰勬湡鏀剁泭**

- 涓昏鏀瑰杽鐩爣锛歚fib`
- 娆¤鏀瑰杽鐩爣锛歚loop`

**褰撳墠宸插畬鎴?*

- 2026-03-16锛氱涓€杞?`method_chain` 鐩稿叧浼樺寲宸插畬鎴愶紝鍦ㄦ暟缁勯珮闃舵柟娉曚腑鍘婚櫎浜嗘瘡涓厓绱犲洖璋冩椂鐨勪复鏃?`Vec<Value>` 鍙傛暟鍒嗛厤銆?- 宸叉坊鍔犻摼寮?`map().filter().reduce()` 琛屼负鍥炲綊娴嬭瘯銆?- Benchmark 缁撴灉锛欳riterion 涓?`method_chain 5k` 浠庣害 `1.88-1.54 ms` 鎻愬崌鍒?`0.80-0.82 ms`銆?
### 9.1.3 Native/builtin 璋冪敤鍙傛暟浼犻€掍紭鍖?
**浼樺厛绾?*: P0

**鐑偣鏂囦欢**

- `src/vm/interpreter.rs`
- `src/vm/natives.rs`

**鍘熷洜**

- Native 鍜?builtin 璋冪敤浠嶅湪鏋勫缓涓存椂 `Vec<Value>` 缂撳啿鍖哄苟鍋?reverse銆?- 姝よ矾寰勫奖鍝?`Math.*`銆乣JSON.*`銆佹暟缁勬柟娉曞拰鍏朵粬鍐呯疆鍑芥暟銆?
**浠诲姟**

- 涓?0/1/2 鍙傛暟鐨?native 璋冪敤澧炲姞涓撻棬鐨勫揩閫熻矾寰勩€?- 閬垮厤涓虹煭鍙傛暟鍒楄〃杩涜鍫嗗垎閰嶃€?- 鍑忓皯鎴栨秷闄?native/builtin 璋冪敤鍑嗗闃舵鐨?`reverse()`銆?- 鍦ㄥ畨鍏ㄥ墠鎻愪笅鑰冭檻浣跨敤鍩轰簬鏍堢殑鍙傛暟鍒囩墖浼犻€掋€?
**棰勬湡鏀剁泭**

- 鏀瑰杽鍐呯疆鍑芥暟瀵嗛泦鍨嬭剼鏈?- 甯姪 `array`銆乣json` 鍜屾暟瀛﹀瘑闆嗗瀷宸ヤ綔璐熻浇

**褰撳墠宸插畬鎴?*

- 2026-03-16锛氫负 `CallMethod` 鐨?native 璺緞娣诲姞浜嗗皬鍙傛暟鏁伴噺鐨勫揩閫熻矾寰勶紝鍦?`argc <= 2` 鏃跺幓闄や复鏃跺弬鏁?`Vec` 鍒嗛厤銆?- 宸叉坊鍔犲鍙傛暟 `Array.prototype.push` 鍙傛暟椤哄簭鍥炲綊娴嬭瘯銆?- Benchmark 缁撴灉锛欳riterion 涓?`array push 10k` 浠庣害 `0.897-0.911 ms` 鎻愬崌鍒?`0.672-0.691 ms`銆?- Benchmark 缁撴灉锛欳riterion 涓?`method_chain 5k` 杩涗竴姝ヤ粠绾?`0.986-1.182 ms` 鎻愬崌鍒?`0.720-0.763 ms`銆?- 2026-03-16锛氬湪 `CallMethod` 涓负 `Array.prototype.push` 娣诲姞浜嗗師鐢熷揩閫熻矾寰勶紝骞堕拡瀵?`argc == 1` 鍦烘櫙澧炲姞浜嗕笓鐢ㄥ揩鎹锋柟寮忥紝浠庣儹鏁扮粍鍒濆鍖栬矾寰勪腑娑堥櫎浜嗛€氱敤 native-call 寮€閿€銆?- 閲嶇敤鐜版湁 `Array.prototype.push` 鍥炲綊娴嬭瘯楠岃瘉璇箟銆?- Benchmark 缁撴灉锛欳riterion 涓?`sieve 10k` 浠庣害 `2.038-2.078 ms` 鎻愬崌鍒?`2.014-2.074 ms`銆?
### 9.1.4 Dense array 蹇€熻矾寰?
**浼樺厛绾?*: P0

**鐑偣鏂囦欢**

- `src/vm/interpreter.rs`
- `src/vm/property.rs`
- `src/runtime/array.rs`

**鍘熷洜**

- `array` 鍜?`sieve` 鏄吀鍨嬬殑 dense-array benchmark銆?- 褰撳墠璁块棶浠嶇粡杩囪嫢骞查€氱敤灞傘€?
**浠诲姟**

- 缂╃煭 `GetArrayEl`銆乣GetArrayEl2` 鍜?`PutArrayEl` 璺緞銆?- 涓?dense integer-index 璁块棶鍋氫笓闂ㄥ鐞嗐€?- 瀵规槑鏄剧殑鏁扮粍鎿嶄綔閬垮厤閫氱敤 property lookup銆?- 鍒嗗埆瀹℃煡 `push`銆佺储寮曡鍙栧拰绱㈠紩鍐欏叆璺緞銆?
**棰勬湡鏀剁泭**

- 涓昏鏀瑰杽鐩爣锛歚array`
- 瀵?`sieve` 涔熸湁鏄庢樉鏀瑰杽

**褰撳墠宸插畬鎴?*

- 2026-03-16锛氬畬鎴愮涓€涓繁搴﹀睘鎬т紭鍖栵紝涓哄父瑙勫璞″睘鎬ф煡鎵炬坊鍔犱簡灏忓璞″揩閫熻矾寰勶紝骞剁粺涓€浜?`GetField` / `GetField2` 鐨勫睘鎬у垎鍙戣矾寰勩€?- 宸叉坊鍔犳繁搴﹀睘鎬ч摼璁块棶鍥炲綊娴嬭瘯銆?- Benchmark 缁撴灉锛欳riterion 涓?`deep_property 200k` 浠庣害 `28-29 ms` 鎻愬崌鍒?`15.7-17.0 ms`銆?
### 9.1.5 Opcode dispatch 绮剧畝

**浼樺厛绾?*: P1

**鐑偣鏂囦欢**

- `src/vm/interpreter.rs`

**鍘熷洜**

- `loop` 浠嶈〃鏄庢湁鎰忎箟鐨勬寚浠?dispatch 寮€閿€銆?- 澶у瀷鍩轰簬 match 鐨?dispatch 姝ｇ‘涓斿彲缁存姢锛屼絾鍦ㄦ渶鐑矾寰勪笂浠嶆湁鏁堢泭鎴愭湰銆?
**浠诲姟**

- 閫氳繃 benchmark 椹卞姩鍒嗘瀽鎵惧嚭鏈€鐑殑 10-20 涓?opcode銆?- 缂╃煭 dispatch 寰幆涓瘡娆¤凯浠ｇ殑宸ヤ綔銆?- 鍑忓皯鐑寚浠や腑鐨勯噸澶?decode / branch / error-path 寮€閿€銆?- 瀵圭畻鏈€佸眬閮ㄥ彉閲忋€佽烦杞拰璋冪敤鎸囦护浼樺厛鍋氭湰鍦板揩閫熻矾寰勩€?
**棰勬湡鏀剁泭**

- `loop` 鐨勬渶浣虫瑕佺洰鏍?- 骞挎硾鎯犲強澶氫釜 benchmark

**褰撳墠宸插畬鎴?*

- 2026-03-16锛氭坊鍔犱簡 `try_catch` benchmark锛岃鐩栭噸澶?throw/catch 鎺у埗娴併€?- 2026-03-16锛氶€氳繃缁熶竴寮傚父鍒嗗彂鍜岀敤鍩轰簬 `truncate` / `drop_n` 鐨?unwind 鏇夸唬閲嶅鐨?pop unwind锛岄檷浣庝簡寮傚父璺敱寮€閿€銆?- 宸叉坊鍔?寰幆鍐呴噸澶?throw/catch"鍥炲綊娴嬭瘯銆?- Benchmark 缁撴灉锛欳riterion 涓?`try_catch 5k` 鍩虹嚎璁板綍涓?`340-349 渭s`銆?- 2026-03-16锛氬湪 `dump` feature 涓嬫坊鍔犱簡杩愯鏃?opcode 璁℃暟鍣紝骞堕€氳繃 `Context` 鏆撮湶缁?profiling 宸ヤ綔浣跨敤銆?- 宸叉坊鍔?`dump` 妯″紡鍥炲綊娴嬭瘯锛岀‘淇?opcode 璁℃暟璁板綍鐪熷疄鎵ц銆?- 杩愯鏃剁儹鐐瑰彂鐜帮細
  - `loop` 涓昏鐢?`GetLoc1`銆乣Goto`銆乣Add`銆乣Dup`銆乣Drop`銆乣GetLoc0`銆乣PutLoc0`銆乣PutLoc1`銆乣Lt`銆乣IfFalse` 涓诲銆?  - `sieve` 涓昏鐢?`Goto`銆乣Drop`銆乣IfFalse`銆乣GetLoc3`銆乣Add`銆乣Dup`銆乣GetLoc0`銆乣Lte`銆乣GetLoc2`銆乣PutArrayEl`銆乣PutLoc3`銆乣GetArrayEl`銆乣CallMethod` 涓诲銆?- 褰撳墠鍒ゆ柇锛氫笅涓€涓熀浜庤瘉鎹殑浼樺寲鐩爣鏇村彲鑳芥槸 `Dup/Drop` + 灞€閮ㄥ瓨鍌ㄤ娇鐢ㄦā寮忔垨鍒嗘敮/鎺у埗娴佹垚鏈紝鑰屼笉鏄户缁紭鍖栧崟涓畻鏈?helper銆?- 2026-03-16锛氬畬鎴愪簡 `Dup + PutLocX + Drop` peephole 蹇€熻矾寰勶紝鐢ㄤ簬 `i = i + 1;` 杩欑被甯歌璇彞鏇存柊妯″紡銆?- 宸叉坊鍔犲眬閮ㄨ祴鍊艰鍙ユ洿鏂板洖褰掓祴璇曪紝鍚屾椂淇濈暀璧嬪€艰〃杈惧紡琛屼负銆?- Benchmark 缁撴灉锛欳riterion 涓?`loop 10k` 浠庣害 `0.513-0.525 ms` 鎻愬崌鍒?`0.486-0.492 ms`銆?- Benchmark 缁撴灉锛欳riterion 涓?`sieve 10k` 浠庣害 `2.257-2.310 ms` 鎻愬崌鍒?`2.152-2.191 ms`銆?- 2026-03-16锛氶€氳繃鐢ㄧ洿鎺ュ揩閫熻矾寰勬爤鎿嶄綔鏇挎崲閫氱敤 checked helper锛屼紭鍖栦簡鐑?`Dup` / `Drop` opcode 澶勭悊鍣ㄦ湰韬€?- 閲嶇敤鐩稿悓鐨勫眬閮ㄨ祴鍊煎拰璧嬪€艰〃杈惧紡鍥炲綊娴嬭瘯鏉ラ獙璇佹洿鏀广€?- 褰撳墠鍩虹嚎浠?`docs/BENCHMARK_ANALYSIS.md` 涓哄噯銆?- 2026-03-16锛氫负绱ц窡 `IfFalse` / `IfTrue` 鐨?`Lt/Lte` 娣诲姞浜嗗垎鏀瀺鍚堝揩閫熻矾寰勶紝浣挎瘮杈冪粨鏋滃彲浠ョ洿鎺ヨ烦杞紝鑰屾棤闇€鍦ㄦ爤涓婂疄渚嬪寲涓存椂甯冨皵鍊笺€?- 閲嶇敤鐜版湁 `while`銆乣switch` 鍜?`try_catch` 鎺у埗娴佸洖褰掓祴璇曟潵楠岃瘉璇箟銆?- Benchmark 缁撴灉锛欳riterion 涓?`loop 10k` 浠庣害 `0.502-0.514 ms` 鎻愬崌鍒?`0.484-0.499 ms`銆?- Benchmark 缁撴灉锛欳riterion 涓?`sieve 10k` 浠庣害 `2.164-2.207 ms` 鎻愬崌鍒?`2.038-2.078 ms`銆?
### 9.1.6 绠楁湳/姣旇緝寰紭鍖?
**浼樺厛绾?*: P1

**鐑偣鏂囦欢**

- `src/vm/ops.rs`

**鍘熷洜**

- 鏍稿績绠楁湳鍜屾瘮杈?helper 宸茬粡閮ㄥ垎 inlines銆?- 杩欏潡浠嶆湁鎰忎箟锛屼絾鍏堕鏈熸敹鐩婁綆浜?call/array/native 鐑矾寰勩€?
**浠诲姟**

- 瀹¤鍓╀綑鐑偣 `op_*` helper 鏄惁鐪熸鍊煎緱 inline銆?- 鍑忓皯甯歌 int/int 鍜?int/float 璺緞涓婄殑閲嶅鏁板€煎己鍒惰浆鎹€?- 鍦?benchmark 鍩虹嚎绋冲畾鍚庨噸鏂拌瘎浼扮浉绛夊拰姣旇緝蹇€熻矾寰勩€?
**棰勬湡鏀剁泭**

- 灏忎絾骞挎硾鐨勬敼鍠?
**褰撳墠宸插畬鎴?*

- 2026-03-16锛氶€氳繃鍦ㄥ崟涓緭鍑虹紦鍐插尯涓瀯寤烘渶缁堣繍琛屾椂瀛楃涓诧紝鑰屼笉鏄厛瀹炰緥鍖栦袱涓搷浣滄暟涓轰复鏃舵嫢鏈夌殑 `String` 鍊硷紝鏀硅繘浜嗗瓧绗︿覆鎷兼帴鐑矾寰勩€?- 宸叉坊鍔犳贩鍚堝瓧绗︿覆/鏁板瓧閾惧紡鎷兼帴褰㈢姸鍥炲綊娴嬭瘯銆?- Benchmark 缁撴灉锛欳riterion 涓?`runtime_string_pressure 4k` 浠庣害 `2.89-3.38 ms` 鎻愬崌鍒?`1.53-1.55 ms`銆?- 2026-03-16锛氶€氳繃鍦ㄥ悓涓€鍊笺€佹暣鏁板拰甯冨皵姣旇緝娣诲姞鐩存帴蹇€熻矾寰勶紝鐒跺悗鍥為€€鍒拌緝鎱㈢殑閫氱敤澶勭悊锛屾敼杩涗簡 `StrictEq` / `StrictNeq` 鐑?opcode 澶勭悊銆?- 閲嶆柊杩愯浜嗙幇鏈?switch 璇箟鍥炲綊娴嬭瘯銆?- Benchmark 缁撴灉锛欳riterion 涓?`switch 1k` 浠庣害 `145-149 渭s` 绫绘€ц兘鎻愬崌鍒?`132-136 渭s`銆?
## 9.2 浼樺寲 GC 鎬ц兘

### 9.2.1 鏇挎崲淇濆畧鐨?`mark_all` 琛屼负

**浼樺厛绾?*: P1

**鐑偣鏂囦欢**

- `src/gc/collector.rs`
- `src/context.rs`

**鍘熷洜**

- 褰撳墠 collector 浠嶅寘鍚繚瀹堢殑涓存椂鏂规锛屾爣璁版墍鏈夊璞°€?- 杩欓樆纰嶄簡鏈夋剰涔夌殑 GC 鎬ц兘宸ヤ綔銆?
**浠诲姟**

- 鐢ㄧ湡姝ｇ殑 root 鍙戠幇鏇挎崲 `mark_all()`銆?- 鏄庣‘骞堕亶鍘嗙湡姝ｇ殑 roots锛?  - stack
  - globals
  - closures
  - active frames
  - runtime-owned containers
- 楠岃瘉 compaction 鍚庢寚閽堟洿鏂颁粛鐒舵纭€?
**棰勬湡鏀剁泭**

- 闄嶄綆 GC pause 鎴愭湰
- 鏀瑰杽瀵硅薄瀵嗛泦鍨嬭剼鏈殑鍙墿灞曟€?
### 9.2.2 娴嬮噺 GC 瑙﹀彂琛屼负

**浼樺厛绾?*: P1

**鍘熷洜**

- GC 鎴愭湰涓嶄粎鍙栧喅浜?collector 瀹炵幇锛岃繕鍙栧喅浜庤Е鍙戦鐜囥€?
**浠诲姟**

- 娴嬮噺 benchmark 宸ヤ綔璐熻浇鏈熼棿鐨?GC 棰戠巼銆?- 璁板綍浠ｈ〃鎬ц剼鏈殑瀵硅薄/鏁扮粍/瀛楃涓插闀裤€?- 鍙湪鏀堕泦鍒扮湡瀹炴暟鎹悗璋冩暣瑙﹀彂鍚彂寮忋€?
### 9.2.3 闄嶄綆寮曟搸鑷湁瀹瑰櫒鐨勬壂鎻忔垚鏈?
**浼樺厛绾?*: P2

**鐑偣鏂囦欢**

- `src/vm/types.rs`
- `src/context.rs`

**浠诲姟**

- 瀹℃煡杩欎簺杩愯鏃跺悜閲忕殑鎵弿鎴愭湰锛?  - `objects`
  - `closures`
  - `runtime_strings`
  - `typed_arrays`
  - `array_buffers`
- 鍦ㄦ湁鐢ㄦ椂灏嗙儹 live data 涓庨暱鐢熷懡鍛ㄦ湡 metadata 鍒嗙銆?
## 9.3 鍑忓皯鍐呭瓨浣跨敤

### 9.3.1 鍏堝仛濂芥祴閲?
**浼樺厛绾?*: P0

**鐑偣鏂囦欢**

- `src/context.rs`
- `src/vm/types.rs`

**鍘熷洜**

- `MemoryStats` 宸茬粡鍙敤锛屼絾鍐呭瓨浼樺寲蹇呴』鍩轰簬鐪熷疄鐨勪富瑕佹潵婧愩€?
**浠诲姟**

- 浠?`MemoryStats` 浣滀负鍩虹嚎娴嬮噺鏉ユ簮銆?- 璁板綍 benchmark 鑴氭湰涓殑 object/string/closure/typed-array 鏁伴噺鍙樺寲銆?- 鍦ㄧ‘璁ゆ渶澶у唴瀛樼被鍒箣鍓嶏紝涓嶅仛婵€杩涚殑甯冨眬閲嶆瀯銆?
**褰撳墠宸插畬鎴?*

- 2026-03-16锛氬皢 `MemoryStats` / `InterpreterStats` 浠?鍙瀵硅薄鏁伴噺"鎵╁睍涓烘洿缁嗙矑搴︾殑缁熻锛屽寘鎷細
  - `runtime_string_bytes`
  - `array_elements`
  - `object_properties`
  - `typed_array_bytes`
  - `array_buffers`
  - `array_buffer_bytes`
- 宸插悓姝ユ洿鏂?CLI 鐨勫唴瀛樿浆鍌ㄨ緭鍑恒€?- 宸叉坊鍔犲洖褰掓祴璇曪紝瑕嗙洊锛?  - 鏁扮粍/瀵硅薄褰㈢姸缁熻
  - 杩愯鏃跺瓧绗︿覆瀛楄妭缁熻
- 鐘舵€侊細鐜板凡鍏峰缁х画鎺ㄨ繘 `9.3` 鐨勬祴閲忓熀纭€銆?
### 9.3.2 鍑忓皯鐑墽琛岃矾寰勪腑鐨勪复鏃跺垎閰?
**浼樺厛绾?*: P0

**鍘熷洜**

- 涓存椂 Vec 鍜岀灛鎬侀噸鎺掍細澧炲姞 CPU 鍜屽唴瀛?churn銆?
**浠诲姟**

- 鍘婚櫎鐑皟鐢ㄨ矾寰勪腑鍓╀綑鐨勪复鏃?`Vec<Value>` 鍒嗛厤銆?- 瀹℃煡鏁扮粍/builtin 瀵嗛泦鍨嬫墽琛屼腑鐨勭煭鐢熷懡鍛ㄦ湡鍒嗛厤妯″紡銆?- 鍦ㄥ畨鍏ㄥ墠鎻愪笅浼樺厛浣跨敤淇濈暀鏍堢殑甯冨眬鍜屽€熺敤鐨勬暟鎹€?
### 9.3.3 瀹℃煡杩愯鏃跺瓧绗︿覆澧為暱

**浼樺厛绾?*: P1

**鐑偣鏂囦欢**

- `src/vm/interpreter.rs`
- `src/context.rs`

**鍘熷洜**

- `runtime_strings` 鍦?`MemoryStats` 涓鍗曠嫭璁℃暟锛屽彲鑳芥倓鎮勫闀裤€?
**浠诲姟**

- 娴嬮噺 benchmark 涓?`runtime_strings` 鐨勫闀挎洸绾裤€?- 妫€鏌ュ瓧绗︿覆鎻愬崌鍦ㄧ儹璺緞涓槸鍚﹁繃浜庢縺杩涖€?- 鎵惧嚭閲嶅瀛楃涓插垱寤虹殑鏈轰細銆?
### 9.3.4 瀹℃煡 object 鍜?array 甯冨眬寮€閿€

**浼樺厛绾?*: P1

**鐑偣鏂囦欢**

- `src/runtime/object.rs`
- `src/runtime/array.rs`
- `src/vm/types.rs`

**浠诲姟**

- 姣旇緝 dense array 涓庨€氱敤 object-backed 璁块棶鐨勫唴瀛樻垚鏈€?- 妫€鏌ラ绻佸垱寤虹殑杩愯鏃剁粨鏋勬槸鍚﹀彲浠ユ洿灏忋€?- 鍙湪娴嬮噺鏀拺涓嬪仛鏈夐拡瀵规€х殑甯冨眬鏀瑰姩銆?
## 鏀寔鎬у紩鎿庝换鍔?
### S1. 淇濇寔 builtin/runtime 杈圭晫鐪熷疄

**浼樺厛绾?*: P2

**鍘熷洜**

- `src/builtins/` 褰撳墠鍩烘湰涓婅繕鏄粨鏋勫崰浣嶇浠ｇ爜銆?- 鐪熸鐨?builtin 琛屼负涓昏鍦?`src/vm/natives.rs` 鍜?`src/vm/property.rs` 涓€?
**浠诲姟**

- 璁板綍鐪熷疄瀹炵幇鎵€鍦ㄤ綅缃€?- 閬垮厤浼樺寲鏃惰鎶婂崰浣嶇妯″潡褰撲綔鐑偣銆?- 闄ら潪闃荤鎬ц兘宸ヤ綔锛屽惁鍒欐帹杩熺粨鏋勮縼绉诲埌鐑偣宸ヤ綔涔嬪悗銆?
### S2. 鐢?benchmark 椹卞姩浼樺寲鐩爣

**浼樺厛绾?*: P0

**鏍囧噯浼樺寲闆嗗悎**

- `fib` -> 璋冪敤璺緞銆侀€掑綊銆佺畻鏈?- `loop` -> dispatch銆佺畻鏈€佸眬閮ㄥ彉閲?- `array` -> dense array 蹇€熻矾寰?- `sieve` -> dense array 璇诲啓 + 寰幆鎴愭湰
- `json` -> 浣滀负宸叉湁濂借矾寰勭殑鍥炲綊淇濇姢

### S3. 鎵╁睍缂哄け鐨?benchmark 瑕嗙洊

**浼樺厛绾?*: P0

**鍘熷洜**

- 褰撳墠 benchmark 闆嗗悎宸叉湁浠峰€硷紝浣嗗鏇村閲嶈寮曟搸璺緞瑕嗙洊浠嶄笉澶熴€?- 濡傛灉 benchmark 浠嶅彧鑱氱劍 `fib`銆乣loop`銆乣array`銆乣sieve`銆乣json`锛屼竴浜涢珮浠峰€间紭鍖栨柟鍚戝皢闀挎湡涓嶅彲瑙併€?
**寤鸿鏂板锛氫富闆嗗悎**

杩欎簺 benchmark 鏈€鍊煎緱浼樺厛鍔犲叆锛屽洜涓哄畠浠渶鐩存帴鏆撮湶鏈夋剰涔夌殑寮曟搸鐑偣锛?
- `method_chain`
  - 浠ｈ〃褰㈢姸锛歚.map().filter().reduce()`
  - 瑕嗙洊锛歚GetField2`銆乣CallMethod`銆佸洖璋冭皟鐢ㄣ€佹暟缁勯摼寮忓鐞?- `for_of_array`
  - 瑕嗙洊锛歚ForOfStart`銆乣ForOfNext`銆佽凯浠ｅ櫒寰幆鎺у埗
- `deep_property`
  - 浠ｈ〃褰㈢姸锛歚a.b.c.d`
  - 瑕嗙洊锛氶噸澶?`GetField` 鎴愭湰涓庨摼寮忓睘鎬ц闂?- `runtime_string_pressure`
  - 瑕嗙洊锛歚create_runtime_string`銆佽繍琛屾椂瀛楃涓插闀裤€佸瓧绗︿覆鍒嗛厤鍘嬪姏

**寤鸿鏂板锛氭闆嗗悎**

杩欎簺鍚屾牱閲嶈锛屼絾鏇撮€傚悎浣滀负鏈哄埗鐗瑰畾鐨?benchmark锛岃€屼笉鏄涓€娉富瑕佹€ц兘 benchmark锛?
- `try_catch`
  - 瑕嗙洊锛歚ExceptionHandler`銆乼hrow/catch/finally 鎺у埗娴併€佹爤灞曞紑
- `for_in_object`
  - 瑕嗙洊锛歚ForInStart`銆乣ForInNext`銆佸璞￠敭杩唬
- `switch_case`
  - 瑕嗙洊锛氬熀浜?`Dup + StrictEq + IfTrue` 鐨勫鍒嗘敮鍒嗗彂缁撴瀯

**涓哄綋鍓?no_std 璺緞寤跺悗**

- `regexp_test`
  - 瑕嗙洊锛歚RegExpObject`銆乣test`
  - 淇濈暀涓哄悗缃?`std` / 鍙€?benchmark 鍊欓€夛紝涓嶄綔涓虹涓€娉?no_std 鐩爣
- `regexp_exec`
  - 瑕嗙洊锛歚RegExpObject`銆乣exec`
  - 淇濈暀涓哄悗缃?`std` / 鍙€?benchmark 鍊欓€夛紝涓嶄綔涓虹涓€娉?no_std 鐩爣

**寤鸿钀藉湴椤哄簭**

1. `method_chain`
2. `runtime_string_pressure`
3. `for_of_array`
4. `deep_property`
5. `try_catch`
6. `switch_case`
7. `for_in_object`

寤跺悗锛?- `regexp_test`
- `regexp_exec`

**棰勬湡浠峰€?*

- 璁?benchmark 椹卞姩鐨勪紭鍖栨洿浠ｈ〃鐪熷疄 JS 浣跨敤鏂瑰紡
- 鏆撮湶璋冪敤瀵嗛泦鍨嬨€佽凯浠ｅ櫒瀵嗛泦鍨嬨€佸璞¤闂瘑闆嗗瀷鍜屽瓧绗︿覆鍘嬪姏瀵嗛泦鍨嬭矾寰?- 璁╁紩鎿庝紭鍖栧伐浣滀笉鍐嶅彧鐪嬬畻鏈拰鍘熷寰幆

**褰撳墠宸插畬鎴?*

- 2026-03-16锛氭坊鍔犱簡绗竴娉?benchmark 鑴氭湰鍜?Criterion 瑕嗙洊锛?  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`
- 2026-03-16锛氭坊鍔犱簡绗簩娉?`switch_case` benchmark 鑴氭湰锛岀敤浜?CLI 椋庢牸鐨?Rust-vs-C 瀵规瘮銆?- 宸查€氳繃 `cargo bench --no-run` 楠岃瘉 benchmark 鍙紪璇戙€?- 2026-03-16锛氬畬鎴愪簡绗竴杞?`for_of_array` 浼樺寲锛宍ForOfStart` 涓嶅啀鏁存暟缁勫鍒讹紝鑰屾槸鏀逛负鍩轰簬鏁扮粍绱㈠紩鐩存帴杩唬銆?- 宸叉坊鍔犲洖褰掓祴璇曪紝纭 `for-of` 鍦ㄦ暟缁勮凯浠ｈ繃绋嬩腑鑳藉瑙傚療鍒板悗缁厓绱犳洿鏀广€?- Benchmark 缁撴灉锛欳riterion 涓?`for_of_array 20k` 浠庣害 `4.22-4.47 ms` 鎻愬崌鍒?`2.36-2.42 ms`銆?- 2026-03-16锛氭坊鍔犱簡 `for_in_object` benchmark 瑕嗙洊锛屽苟瀹屾垚浜嗙涓€杞凯浠ｅ櫒鍒濆鍖栦紭鍖栵紝灏?棰勫厛鐢熸垚鍏ㄩ儴 key"鏀逛负"鍩轰簬蹇収鎸夐渶鐢熸垚 key"銆?- 宸叉坊鍔犲洖褰掓祴璇曪紝纭 `for-in` 鍦ㄥ璞¤凯浠ｈ繃绋嬩腑浠嶈兘閫氳繃闈欐€佸睘鎬ц鍙栬瀵熷埌鏇存柊鍚庣殑鍊笺€?- Benchmark 鍩虹嚎璁板綍锛欳riterion 涓?`for_in_object 20x2000` 涓?`3.74-3.80 ms`銆?
## 鎺ㄨ崘鎵ц椤哄簭

1. Benchmark 鍩虹嚎娓呯悊
2. Benchmark 瑕嗙洊鎵╁睍锛堜紭鍏堝姞鍏?`method_chain`銆乣runtime_string_pressure`銆乣for_of_array`銆乣deep_property`锛?3. 璋冪敤璺緞浼樺寲
4. Native/builtin 鍙傛暟浼犻€掍紭鍖?5. Dense array 蹇€熻矾寰?6. 鍐呭瓨娴嬮噺璇勪及
7. GC root-based marking 宸ヤ綔
8. Opcode dispatch 绮剧畝
9. 绗簩杞井浼樺寲

## 瀹屾垚鏍囧噯

褰撴弧瓒充互涓嬫潯浠舵椂锛岃繖浠戒紭鍖栦换鍔℃竻鍗曞彲瑙嗕负"鍩烘湰瀹屾垚"锛?
- benchmark 鍩虹嚎鍙俊銆佸彲澶嶇幇
- `fib`銆乣loop`銆乣array`銆乣sieve` 鍚勮嚜鑷冲皯鏈変竴涓粡杩囬獙璇佺殑鐑偣鏀瑰杽
- GC 涓嶅啀渚濊禆淇濆畧鐨?`mark_all`
- 鍐呭瓨鍑忓皯宸ヤ綔鍩轰簬娴嬮噺鐨勪富瑕佺被鍒紝鑰岄潪鐚滄祴
- 鏂囨。鍙褰曟湁鏁堢殑 benchmark 缁撹

