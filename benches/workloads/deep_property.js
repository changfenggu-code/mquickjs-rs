// Deep property benchmark - tests chained property access
var root = { a: { b: { c: { d: 1 } } } };
var sum = 0;

for (var i = 0; i < 200000; i = i + 1) {
    sum = sum + root.a.b.c.d;
}
