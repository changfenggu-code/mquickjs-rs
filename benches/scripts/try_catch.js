// try/catch benchmark - tests exception handler setup and unwind path
var sum = 0;
for (var i = 0; i < 5000; i = i + 1) {
    try {
        throw i;
    } catch (e) {
        sum = sum + e;
    }
}
print("try_catch sum = " + sum);
