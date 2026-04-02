// String benchmark - tests string operations
var s = "";
for (var i = 0; i < 1000; i = i + 1) {
    s = s + "x";
}

var count = 0;
for (var i = 0; i < 1000; i = i + 1) {
    if (s.indexOf("x") >= 0) {
        count = count + 1;
    }
}
