var arr = [];
for (var i = 0; i <= 10000; i = i + 1) {
    arr.push(true);
}

for (var i = 0; i <= 10000; i = i + 1) {
    arr[i] = false;
}

var sum = 0;
for (var i = 0; i <= 10000; i = i + 1) {
    if (arr[i]) sum = sum + 1;
}

return sum;
