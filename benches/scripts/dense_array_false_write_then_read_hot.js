var arr = [];
for (var i = 0; i <= 10000; i = i + 1) {
    arr.push(true);
}

for (var i = 0; i <= 10000; i = i + 1) {
    arr[i] = false;
}

function countTrue(a) {
    var count = 0;
    for (var i = 0; i <= 10000; i = i + 1) {
        if (a[i]) count = count + 1;
    }
    return count;
}

var total = 0;
for (var r = 0; r < 200; r = r + 1) {
    total = total + countTrue(arr);
}

return total;
