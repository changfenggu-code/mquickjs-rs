var arr = [];
for (var i = 0; i <= 10000; i = i + 1) {
    arr.push(true);
}
arr[0] = false;
arr[1] = false;

function scan(a) {
    for (var i = 0; i <= 10000; i = i + 1) {
        if (a[i]) { }
    }
}

for (var r = 0; r < 200; r = r + 1) {
    scan(arr);
}

return 0;
