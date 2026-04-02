function Point(x, y) {
    this.x = x;
    this.y = y;
}

var points = [];
for (var i = 0; i < 1000; i = i + 1) {
    points.push(new Point(i, i * 2));
}
