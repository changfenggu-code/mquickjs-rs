// JSON parse benchmark - parse only, without post-parse property reads
var data = '{"name": "test", "value": 42, "items": [1, 2, 3]}';

var count = 0;
for (var i = 0; i < 1000; i = i + 1) {
    JSON.parse(data);
    count = count + 1;
}
print("json_parse_only count = " + count);
