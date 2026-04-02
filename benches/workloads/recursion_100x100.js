function sum(n) {
    if (n <= 0) return 0;
    return n + sum(n - 1);
}

var total = 0;
for (var i = 0; i < 100; i = i + 1) {
    total = total + sum(100);
}
