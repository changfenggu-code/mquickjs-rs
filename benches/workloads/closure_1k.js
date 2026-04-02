function makeAdder(x) {
    function adder(y) {
        return x + y;
    }
    return adder;
}

var sum = 0;
for (var i = 0; i < 1000; i = i + 1) {
    var add = makeAdder(i);
    sum = sum + add(i);
}
