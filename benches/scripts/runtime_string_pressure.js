// Runtime string pressure benchmark - tests repeated runtime string creation
var parts = [];
for (var i = 0; i < 4000; i = i + 1) {
    parts.push("item-" + i + "-" + (i % 17));
}

var total = 0;
for (var i = 0; i < parts.length; i = i + 1) {
    total = total + parts[i].length;
}

print("runtime_string_pressure total = " + total);
