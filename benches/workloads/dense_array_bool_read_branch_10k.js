var arr = [];
for (var i = 0; i <= 10000; i = i + 1) {
    arr.push(true);
}
arr[0] = false;
arr[1] = false;

var count = 0;
for (var i = 0; i <= 10000; i = i + 1) {
    if (arr[i]) count = count + 1;
}

