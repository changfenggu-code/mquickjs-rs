// JSON parse benchmark - parse plus one hot property read
var data = '{"name": "test", "value": 42, "items": [1, 2, 3]}';

var sum = 0;
for (var i = 0; i < 1000; i = i + 1) {
    var obj = JSON.parse(data);
    sum = sum + obj.value;
}
