function fib(n) {
    if (n <= 1) return n;
    var a = 0;
    var b = 1;
    for (var i = 2; i <= n; i = i + 1) {
        var c = a + b;
        a = b;
        b = c;
    }
    return b;
}

var sum = 0;
for (var i = 0; i < 1000; i = i + 1) {
    sum = sum + fib(30);
}
