// for-of benchmark - tests iterator setup and iteration over arrays
var arr = [];
for (var i = 0; i < 20000; i = i + 1) {
    arr.push(i);
}

var sum = 0;
for (var value of arr) {
    sum = sum + value;
}
