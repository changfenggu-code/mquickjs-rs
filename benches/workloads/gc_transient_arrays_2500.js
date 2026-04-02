function makeArray() {
    var a = [1, 2, 3, 4, 5, 6, 7, 8];
    return a.length;
}

var sum = 0;
for (var i = 0; i < 2500; i = i + 1) {
    sum = sum + makeArray();
}

