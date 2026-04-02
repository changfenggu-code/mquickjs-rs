// Method-chain benchmark - tests callback-heavy array method dispatch
var arr = [];
for (var i = 0; i < 5000; i = i + 1) {
    arr.push(i);
}

function double(x) { return x * 2; }
function divByThree(x) { return x % 3 === 0; }
function add(acc, x) { return acc + x; }

arr.map(double).filter(divByThree).reduce(add, 0);
